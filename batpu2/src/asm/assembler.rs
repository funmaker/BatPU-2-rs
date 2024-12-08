use std::collections::HashMap;
use arrayvec::ArrayVec;

use crate::asm::{AsmError, Token, Line};
use crate::isa::{Instruction, InstructionError, MAX_CODE_LEN, MAX_ARGS, Mnemonic};
use crate::utils::Char;

const MAX_ERRORS: usize = 100;

pub fn assemble<'l, 'c>(lines: &'l [Line<'c>]) -> impl 'l + Iterator<Item=Result<Instruction, AsmError<'c>>> {
	Assembler::new(lines)
}

struct Assembler<'l, 'c> {
	line: usize,
	lines: &'l [Line<'c>],
	pc: i16,
	pc_overflow: bool,
	errors: usize,
	symbols: HashMap<&'c str, i16>,
	symbols_done: bool,
}

impl<'l, 'c> Assembler<'l, 'c> {
	fn new(lines: &'l [Line<'c>]) -> Self {
		Self {
			line: 0,
			lines,
			pc: 0,
			pc_overflow: false,
			errors: 0,
			symbols: HashMap::new(),
			symbols_done: false,
		}
	}
	
	fn define_symbols(&mut self, line: &'l Line<'c>) -> Result<(), AsmError<'c>> {
		if let Some(label) = line.label {
			self.check_pc_overflow(line.line_number, label)?;
			
			self.symbols.insert(label.span, self.pc);
		}
		
		if Some("define") == line.mnemonic.as_deref() {
			let [key, value] = line.args.as_ref()
			                       .try_into()
			                       .map_err(|_| AsmError::WrongOperandCount {
				                       line_number: line.line_number,
				                       expected: 2..=2,
				                       args: line.args.clone(),
				                       mnemonic: line.mnemonic.unwrap()
			                       })?;
			
			let value = value.parse()
			                 .map_err(|source| AsmError::IntParseError {
				                 line_number: line.line_number,
				                 token: value,
				                 source,
			                 })?;
			
			self.symbols.insert(key.span, value);
		} else if let Some(mnemonic) = line.mnemonic {
			self.check_pc_overflow(line.line_number, mnemonic)?;
			
			self.pc += 1;
		}
		
		Ok(())
	}
	
	fn check_pc_overflow(&mut self, line_number: usize, token: Token<'c>) -> Result<(), AsmError<'c>> {
		if self.pc as usize >= MAX_CODE_LEN {
			self.pc_overflow = true;
			Err(AsmError::TooManyInstructions { line_number, token })
		} else {
			Ok(())
		}
	}
	
	fn resolve_token(&self, line: &Line, token: Token<'c>, literal: bool) -> Result<i16, AsmError<'c>> {
		if literal {
			if let Some(char) = match token.as_bytes() {
				&[b'"' | b'\'', ref inner @ .., b'"' | b'\''] if !inner.is_empty() && inner.trim_ascii().is_empty() => Char::try_from(' ').ok(),
				&[b'\'', inner, b'\''] |
				&[b'"', inner, b'"'] => Char::try_from(inner as char).ok(),
				_ => None,
			} {
				return Ok(char.as_u8() as i16);
			}
			
			if let Ok(int) = token.span.parse() {
				return Ok(int);
			}
		}
		
		self.symbols
		    .get(token.span)
		    .copied()
		    .or_else(|| default_symbols(token.span))
		    .ok_or(AsmError::UnknownSymbol {
			    line_number: line.line_number,
			    token,
			    literal,
		    })
	}
}

impl<'l, 'c> Iterator for Assembler<'l, 'c> {
	type Item = Result<Instruction, AsmError<'c>>;
	
	fn next(&mut self) -> Option<Self::Item> {
		if self.pc_overflow || self.errors > MAX_ERRORS {
			return None
		}
		
		while !self.symbols_done {
			if let Some(line) = self.lines.get(self.line) {
				self.line += 1;
				if let Err(err) = self.define_symbols(line) {
					self.errors += 1;
					return Some(Err(err))
				}
			} else {
				self.symbols_done = true;
				self.line = 0;
			}
		}
		
		while let Some(line) = self.lines.get(self.line) {
			self.line += 1;
			
			if let Some(mnemonic_token) = line.mnemonic {
				let mnemonic = match self.resolve_token(line, mnemonic_token, false).ok()
				                         .and_then(|opcode| opcode.try_into().ok())
				                         .and_then(|opcode: i16| Mnemonic::try_from(opcode).ok()) {
					Some(mnemonic) => mnemonic,
					None => return Some(Err(AsmError::UnknownMnemonic { line_number: line.line_number, token: mnemonic_token })),
				};
				
				let args = match line.args.iter()
				                          .map(|&token| self.resolve_token(line, token, true))
				                          .collect::<Result<ArrayVec<_, MAX_ARGS>, _>>() {
					Ok(args) => args,
					Err(err) => return Some(Err(err)),
				};
				
				let instruction = match Instruction::new(mnemonic, args.clone()) {
					Ok(instruction) => instruction,
					Err(InstructionError::WrongOperandCount { expected, .. }) => return Some(Err(AsmError::WrongOperandCount {
						line_number: line.line_number,
						expected,
						mnemonic: mnemonic_token,
						args: line.args.clone(),
					})),
					Err(InstructionError::OperandOutOfRange { operand, name, min, max, got }) => return Some(Err(AsmError::OperandOutOfRange {
						line_number: line.line_number,
						mnemonic: mnemonic_token,
						token: line.args[operand],
						operand, name, min, max, got,
					})),
				};
				
				return Some(Ok(instruction));
			}
		}
		
		None
	}
	
	fn size_hint(&self) -> (usize, Option<usize>) {
		if self.symbols_done {
			(0, Some(self.lines.len() - self.line))
		} else {
			(0, Some(self.lines.len() * 2 - self.line))
		}
	}
}


fn default_symbols(symbol: &str) -> Option<i16> {
	Some(match symbol {
		"pixel_x"             => 240,
		"pixel_y"             => 241,
		"draw_pixel"          => 242,
		"clear_pixel"         => 243,
		"load_pixel"          => 244,
		"buffer_screen"       => 245,
		"clear_screen_buffer" => 246,
		"write_char"          => 247,
		"buffer_chars"        => 248,
		"clear_chars_buffer"  => 249,
		"show_number"         => 250,
		"clear_number"        => 251,
		"signed_mode"         => 252,
		"unsigned_mode"       => 253,
		"rng"                 => 254,
		"controller_input"    => 255,
		
		"NOP" => 0x0,
		"HLT" => 0x1,
		"ADD" => 0x2,
		"SUB" => 0x3,
		"NOR" => 0x4,
		"AND" => 0x5,
		"XOR" => 0x6,
		"RSH" => 0x7,
		"LDI" => 0x8,
		"ADI" => 0x9,
		"JMP" => 0xA,
		"BRH" => 0xB,
		"CAL" => 0xC,
		"RET" => 0xD,
		"LOD" => 0xE,
		"STR" => 0xF,
		
		"r0"  => 0,
		"r1"  => 1,
		"r2"  => 2,
		"r3"  => 3,
		"r4"  => 4,
		"r5"  => 5,
		"r6"  => 6,
		"r7"  => 7,
		"r8"  => 8,
		"r9"  => 9,
		"r10" => 10,
		"r11" => 11,
		"r12" => 12,
		"r13" => 13,
		"r14" => 14,
		"r15" => 15,
		
		"eq" | "="  | "z"  | "zero"     => 0,
		"ne" | "!=" | "nz" | "notzero"  => 1,
		"ge" | ">=" | "c"  | "carry"    => 2,
		"lt" | "<"  | "nc" | "notcarry" => 3,
		
		_ => return None,
	})
}

