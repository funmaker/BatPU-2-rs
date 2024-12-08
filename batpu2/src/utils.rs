use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use thiserror::Error;

use crate::asm::{self, AsmError};
use crate::isa::Instruction;

pub fn from_asm(code: &str) -> Result<Vec<Instruction>, AsmError> {
	let lines = asm::parse_lines(code).collect::<Result<Vec<_>, _>>()?;
	let instructions = asm::assemble(&lines).collect::<Result<Vec<_>, _>>()?;
	
	Ok(instructions)
}

pub fn from_mc(code: &str) -> Result<Vec<Instruction>, FromMcError> {
	code.lines()
		.enumerate()
		.filter(|(_, line)| !line.is_empty())
	    .map(|(line_number, line)|
		    u16::from_str_radix(line, 2)
		        .map(Into::into)
		        .map_err(|source| FromMcError { line_number, line: line.to_owned(), source }))
	    .collect()
}

#[derive(Error, Debug)]
#[error("{line_number}: Cannot parse \"{line}\"")]
pub struct FromMcError {
	line_number: usize,
	line: String,
	#[source] source: ParseIntError,
}

pub fn into_mc(instructions: &[Instruction]) -> String {
	let mut output = String::with_capacity(instructions.len() * 17);
	
	for instruction in instructions.iter() {
		output.push_str(&format!("{:b}\n", instruction.as_word()))
	}
	
	output
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct Char(u8);

impl Char {
	pub const TABLE: [char; 30] = [' ', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '.', '!', '?'];
	pub const SPACE: Char = Char(0);
	
	pub fn new(value: u8) -> Self {
		Self(value)
	}
	
	pub fn to_char(self) -> Option<char> {
		Char::TABLE.get(self.as_u8() as usize)
		           .copied()
	}
	
	pub fn as_u8(self) -> u8 {
		self.0
	}
	
	pub fn is_valid(self) -> bool {
		(self.as_u8() as usize) < Char::TABLE.len()
	}
}

impl TryFrom<char> for Char {
	type Error = String; // todo
	
	fn try_from(value: char) -> Result<Self, Self::Error> {
		if value == ' ' { Ok(Char::SPACE) }
		else if value == '.' { Ok(Char::new(27)) }
		else if value == '!' { Ok(Char::new(28)) }
		else if value == '?' { Ok(Char::new(29)) }
		else if value.is_ascii_alphabetic() {
			Ok(Char::new(value.to_ascii_uppercase() as u8 - 'A' as u8 + 1))
		}else{
			Err(format!("invalid char: {}", value))
		}
	}
}

impl Display for Char {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		if let Some(c) = self.to_char() {
			c.fmt(f)
		} else {
			write!(f, "<{}>", self.0)
		}
	}
}

