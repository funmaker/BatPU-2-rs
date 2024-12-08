use std::fmt::{self, Write, Display, Formatter};
use std::ops::RangeInclusive;
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
		write!(output, "{:b}\n", instruction.as_word()).unwrap();
	}
	
	output
}

pub(crate) struct PrettyRange<'a>(pub &'a RangeInclusive<usize>);

impl Display for PrettyRange<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		if *self.0.start() == 0 && *self.0.end() == 0 {
			write!(f, "no")
		} else if self.0.start() == self.0.end() {
			write!(f, "{}", *self.0.start())
		} else {
			write!(f, "from {} to {}", *self.0.start(), *self.0.end())
		}
	}
}

#[derive(Error, Debug)]
#[error("Invalid character \"{char}\"")]
pub struct InvalidCharacter { char: char }

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
	type Error = InvalidCharacter; // todo
	
	fn try_from(value: char) -> Result<Self, Self::Error> {
		for (id, char) in Self::TABLE.into_iter().enumerate() {
			if value == char {
				return Ok(Char(id as u8))
			}
		}
		
		Err(InvalidCharacter { char: value })
	}
}

impl Display for Char {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		if let Some(c) = self.to_char() {
			c.fmt(f)
		} else {
			write!(f, "<{}>", self.0)
		}
	}
}

