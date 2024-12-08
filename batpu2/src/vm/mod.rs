use std::fmt::{Debug, Formatter};
use std::error::Error as StdError;
use thiserror::Error;

pub mod io;
pub mod code;

use crate::isa::{Cond, Instruction};
use code::Code;
use io::RawIO;
#[cfg(feature = "embedded_io")]
use io::embedded;

#[derive(Debug, Eq, PartialEq, Default)]
pub struct Flags {
	pub zero: bool,
	pub carry: bool,
}

#[cfg(feature = "embedded_io")]
type DefaultIO = embedded::EmbeddedIO;
#[cfg(not(feature = "embedded_io"))]
type DefaultIO = ();

pub struct BatPU2<C: ?Sized, I = DefaultIO> {
	pub flags: Flags,
	pub io: I,
	pub pc: u16,
	pub registers: [u8; 15],
	pub call_stack: [u16; 16],
	pub memory: [u8; 240],
	pub halted: bool,
	pub code: C,
}

#[cfg(feature = "embedded_io")]
impl<C> BatPU2<C, embedded::EmbeddedIO>
where C: Code {
	pub fn new(code: C) -> Self {
		Self {
			flags: Flags::default(),
			io: embedded::EmbeddedIO::new(),
			pc: 0,
			registers: [0; 15],
			call_stack: [0; 16],
			memory: [0; 240],
			halted: false,
			code,
		}
	}
}

#[cfg(feature = "embedded_io")]
impl BatPU2<Vec<Instruction>, embedded::EmbeddedIO> {
	pub fn from_asm(code: &str) -> Result<Self, crate::asm::AsmError> {
		Ok(Self::new(crate::utils::from_asm(code)?))
	}
	
	pub fn from_mc(code: &str) -> Result<Self, crate::utils::FromMcError> {
		Ok(Self::new(crate::utils::from_mc(code)?))
	}
}

impl<C, I> BatPU2<C, I>
where C: Code,
      I: RawIO {
	pub fn with_io(code: C, io: I) -> Self {
		Self {
			flags: Flags::default(),
			io,
			pc: 0,
			registers: [0; 15],
			call_stack: [0; 16],
			memory: [0; 240],
			halted: false,
			code,
		}
	}
	
	pub fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), RunError<I::Error, C::Error>> {
		match instruction {
			Instruction::NOP => {}
			Instruction::HLT => { self.halted = true }
			Instruction::ADD{ a, b, c } => {
				let (result, overflow) = self.register(a).overflowing_add(self.register(b));
				self.write_register_and_flags(c, result, overflow);
			}
			Instruction::SUB{ a, b, c } => {
				let (result, overflow) = self.register(a).overflowing_sub(self.register(b));
				self.write_register_and_flags(c, result, !overflow);
			}
			Instruction::NOR{ a, b, c } => { self.write_register_and_flags(c, !(self.register(a) | self.register(b)), false) }
			Instruction::AND{ a, b, c } => { self.write_register_and_flags(c, self.register(a) & self.register(b), false) }
			Instruction::XOR{ a, b, c } => { self.write_register_and_flags(c, self.register(a) ^ self.register(b), false) }
			Instruction::RSH{ a, c } => { self.write_register(c, self.register(a) >> 1) }
			Instruction::LDI{ a, imm } => { self.write_register(a, imm.cast_unsigned()) }
			Instruction::ADI{ a, imm } => {
				let (result, overflow) = imm.cast_unsigned().overflowing_add(self.register(a));
				self.write_register_and_flags(a, result, overflow);
			}
			Instruction::JMP{ addr } => { self.pc = addr }
			Instruction::BRH{ cond, addr } => {
				if match cond {
					Cond::Zero => self.flags.zero,
					Cond::NotZero => !self.flags.zero,
					Cond::Carry => self.flags.carry,
					Cond::NotCarry => !self.flags.carry,
				} { self.pc = addr; }
			}
			Instruction::CAL{ addr } => {
				self.call_stack.rotate_right(1);
				self.call_stack[0] = self.pc;
				self.pc = addr;
			}
			Instruction::RET => {
				self.pc = self.call_stack[0];
				self.call_stack[0] = 0;
				self.call_stack.rotate_left(1);
			}
			Instruction::LOD{ a, b, offset } => {
				let data = self.read_memory(self.resolve_offset(a, offset))?;
				self.write_register(b, data);
			}
			Instruction::STR{ a, b, offset } => { self.write_memory(self.resolve_offset(a, offset), self.register(b))? }
		}
		
		Ok(())
	}
	
	pub fn try_step(&mut self) -> Result<(), RunError<I::Error, C::Error>> {
		self.pc %= 1 << 10;
		let instruction = self.code.instruction(self.pc)
		                           .map_err(RunError::CodeError)?
		                           .unwrap_or(Instruction::NOP);
		self.pc += 1;
		
		self.execute_instruction(instruction)?;
		
		Ok(())
	}
	
	pub fn try_step_multiple(&mut self, limit: usize) -> Result<usize, RunError<I::Error, C::Error>> {
		for count in 0..limit {
			if self.halted { return Ok(count) }
			self.try_step()?;
		}
		Ok(limit)
	}
	
	fn register(&self, reg: u8) -> u8 {
		match reg {
			0 => 0,
			1..=15 => self.registers[reg as usize - 1],
			_ => unreachable!(),
		}
	}
	
	fn write_register_and_flags(&mut self, reg: u8, value: u8, carry: bool) {
		self.flags.carry = carry;
		self.flags.zero = value == 0;
		self.write_register(reg, value);
	}
	
	fn write_register(&mut self, reg: u8, val: u8) {
		if reg != 0 {
			self.registers[reg as usize - 1] = val;
		}
	}
	
	fn write_memory(&mut self, addr: u8, value: u8) -> Result<(), RunError<I::Error, C::Error>> {
		if addr < 240 {
			Ok(self.memory[addr as usize] = value)
		} else {
			self.io.write_addr(addr, value)
			       .map_err(RunError::IOError)
		}
	}
	
	fn read_memory(&mut self, addr: u8) -> Result<u8, RunError<I::Error, C::Error>> {
		if addr < 240 {
			Ok(self.memory[addr as usize])
		} else {
			self.io.read_addr(addr)
			       .map_err(RunError::IOError)
		}
	}
	
	fn resolve_offset(&self, reg: u8, offset: i8) -> u8 {
		self.register(reg)
			.wrapping_add(offset.cast_unsigned())
	}
}

impl <T, I> BatPU2<T, I>
where T: Code<Error=!>,
      I: RawIO<Error=!> {
	pub fn step(&mut self) {
		self.try_step().unwrap()
	}
	
	pub fn step_multiple(&mut self, limit: usize) -> usize {
		self.try_step_multiple(limit).unwrap()
	}
}

impl<T, I> Debug for BatPU2<T, I>
where T: Code,
      I: Debug {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("BatPU2")
		 .field("halted", &self.halted)
		 .field("rom size", &self.code.len())
		 .field("pc", &self.pc)
		 .field_with("flags", |f| f.write_str(&format!("{:?}", self.flags)))
		 .field_with("registers", |f| f.write_str(&format!("{:?}", self.registers)))
		 .field_with("memory", |f| f.write_str(&format!("{:?}", self.memory)))
		 .field_with("call_stack", |f| f.write_str(&format!("{:?}", self.call_stack)))
		 .field("io", &self.io)
		 .finish()
	}
}

#[derive(Error, Debug)]
pub enum RunError<IOError: StdError + 'static, CodeError: StdError + 'static> {
	#[error("Input/Output error: {}", .0)]
	IOError(#[source] IOError),
	#[error("Code error: {}", .0)]
	CodeError(#[source] CodeError),
}
