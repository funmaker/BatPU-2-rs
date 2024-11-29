use std::ops::Rem;
use flags::Flags;
use io::IO;
use num_enum::TryFromPrimitive;

#[cfg(feature = "embedded_io")]
use io::EmbeddedIO;
use crate::RawIO;

pub mod flags;
pub mod io;

pub struct BatPU2<T, I> {
	pub flags: Flags,
	pub io: I,
	pub pc: u16,
	pub registers: [u8; 16],
	pub call_stack: [u16; 16],
	pub memory: [u8; 240],
	pub code: T,
	pub halted: bool,
}

#[cfg(feature = "embedded_io")]
impl<T> BatPU2<T, EmbeddedIO>
where T: AsRef<[u16]> {
	pub fn new(code: T) -> Self {
		Self {
			flags: Flags::default(),
			io: EmbeddedIO::new(),
			pc: 0,
			registers: [0; 16],
			call_stack: [0; 16],
			memory: [0; 240],
			code,
			halted: false,
		}
	}
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum Opcode {
	NOP, HLT, ADD, SUB,
	NOR, AND, XOR, RSH,
	LDI, ADI, JMP, BRH,
	CAL, RET, LOD, STR
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum ConditionalFlags {
	Z, NZ, C, NC
}

impl<T, I> BatPU2<T, I>
where T: AsRef<[u16]>,
      I: IO {
	pub fn with_io(code: T, io: I) -> Self {
		Self {
			flags: Flags::default(),
			io,
			pc: 0,
			registers: [0; 16],
			call_stack: [0; 16],
			memory: [0; 240],
			code,
			halted: false,
		}
	}

	fn match_flags(&self, flags: ConditionalFlags) -> bool {
		match flags {
			ConditionalFlags::Z => { self.flags.zero }
			ConditionalFlags::NZ => { !self.flags.zero }
			ConditionalFlags::C => { self.flags.carry }
			ConditionalFlags::NC => { !self.flags.carry }
		}
	}

	fn write_register_update_flags(&mut self, reg: usize, value: u8, carry: bool) {
		self.flags.carry = carry;
		self.flags.zero = value == 0;
		self.write_register(reg, value);
	}

	fn write_register(&mut self, reg: usize, val: u8) {
		if reg != 0 {
			self.registers[reg] = val;
		}
	}

	fn write_memory(&mut self, addr: usize, value: u8) {
		let addr = addr & 0xff;
		if addr < 240 {
			self.memory[addr] = value;
		}else{
			self.io.write_addr(addr as u8, value);
		}
	}

	fn read_memory(&mut self, addr: usize) -> u8 {
		let addr = addr & 0xff;
		if addr < 240 {
			self.memory[addr]
		}else{
			self.io.read_addr(addr as u8)
		}
	}

	fn get_offset_address(&self, reg: usize, offset: usize) -> usize {
		let offset = if offset & 0b1000 != 0 {
			(offset | !0b1111).cast_signed()
		} else {
			offset as isize
		};
		(self.registers[reg] as isize + offset) as usize
	}

	pub fn execute_instruction(&mut self, instruction: u16) {
		let opcode = Opcode::try_from((instruction >> 12) as u8).unwrap();

		let operand_r_a = ((instruction & 0x0F00) >> 8) as usize;
		let operand_r_b = ((instruction & 0x00F0) >> 4) as usize;
		let operand_r_c = (instruction & 0x000F) as usize;
		let operand_m = instruction & 0x03FF;
		let operand_imm = (instruction & 0x00FF) as u8;

		match opcode {
			Opcode::NOP => {}
			Opcode::HLT => { self.halted = true }
			Opcode::ADD => {
				let (result, overflow) = self.registers[operand_r_a].overflowing_add(self.registers[operand_r_b]);
				self.write_register_update_flags(operand_r_c, result, overflow);
			}
			Opcode::SUB => {
				let (result, overflow) = self.registers[operand_r_a].overflowing_sub(self.registers[operand_r_b]);
				self.write_register_update_flags(operand_r_c, result, !overflow); // todo check cf
			}
			Opcode::NOR => { self.write_register_update_flags(operand_r_c, !(self.registers[operand_r_a] | self.registers[operand_r_b]), false) }
			Opcode::AND => { self.write_register_update_flags(operand_r_c, self.registers[operand_r_a] & self.registers[operand_r_b], false) }
			Opcode::XOR => { self.write_register_update_flags(operand_r_c, self.registers[operand_r_a] ^ self.registers[operand_r_b], false) }
			Opcode::RSH => { self.write_register(operand_r_c, self.registers[operand_r_a] >> 1) }
			Opcode::LDI => { self.write_register(operand_r_a, operand_imm) }
			Opcode::ADI => {
				let (result, overflow) = operand_imm.overflowing_add(self.registers[operand_r_b]);
				self.write_register_update_flags(operand_r_a, result, overflow);
			}
			Opcode::JMP => { self.pc = operand_m }
			Opcode::BRH => {
				if self.match_flags(ConditionalFlags::try_from(((instruction & 0x0300) >> 8) as u8).unwrap()) {
					self.pc = operand_m;
				}
			}
			Opcode::CAL => {
				self.call_stack.rotate_left(1);
				self.call_stack[0] = self.pc;
				self.pc = operand_m;
			}
			Opcode::RET => {
				self.pc = self.call_stack[0];
				self.call_stack[0] = 0;
				self.call_stack.rotate_right(1);
			}
			Opcode::LOD => {
				let data = self.read_memory(self.get_offset_address(operand_r_a, operand_r_c));
				self.write_register(operand_r_b, data);
			}
			Opcode::STR => { self.write_memory(self.get_offset_address(operand_r_a, operand_r_c), self.registers[operand_r_b]) }
		}
	}

	pub fn step(&mut self) {
		let instruction = self.code.as_ref().get(self.pc as usize).unwrap_or(&0).clone();
		self.pc = self.pc.wrapping_add(1).rem(self.code.as_ref().len() as u16);
		self.execute_instruction(instruction);
	}

	pub fn step_multiple(&mut self, limit: usize) -> usize {
		for count in 0..limit {
			if self.halted { return count }
			self.step();
		}
		limit
	}
}
