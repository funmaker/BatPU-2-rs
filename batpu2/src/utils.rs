//! Utility functions for loading and assembling program files

use std::fmt::{self, Write, Display, Formatter};
use std::ops::RangeInclusive;
use std::num::ParseIntError;
use thiserror::Error;

use crate::asm::{self, AsmError};
use crate::isa::Instruction;

/// Parses and assembles a program from a source code written in BatPU2 assembly
///
/// # Example
///
/// ```
/// use batpu2::isa::Instruction;
///
/// let code = "
/// JMP .label
/// .label ADD r1 r2 r3
/// ";
///
/// let program = batpu2::utils::from_asm(code).unwrap();
///
/// assert_eq!(&program, &[
///     Instruction::JMP { addr: 1 },
///     Instruction::ADD{ a: 1, b: 2, c: 3 },
/// ]);
/// ```
pub fn from_asm(code: &str) -> Result<Vec<Instruction>, AsmError> {
	let lines = asm::parse_lines(code).collect::<Result<Vec<_>, _>>()?;
	let instructions = asm::assemble(&lines).collect::<Result<Vec<_>, _>>()?;
	
	Ok(instructions)
}

/// Loads a compiled program from a compiled code in .mc format
///
/// # Example
///
/// ```
/// use batpu2::isa::Instruction;
///
/// let code = "
/// 1010000000000001
/// 0010000100100011
/// ";
///
/// let program = batpu2::utils::from_mc(code).unwrap();
///
/// assert_eq!(&program, &[
///     Instruction::JMP { addr: 1 },
///     Instruction::ADD{ a: 1, b: 2, c: 3 },
/// ]);
/// ```
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

/// Serializes a program into a string in .mc format
///
/// ```
/// use batpu2::isa::Instruction;
///
/// let program = [
///     Instruction::JMP { addr: 1 },
///     Instruction::ADD{ a: 1, b: 2, c: 3 },
/// ];
///
/// let code = batpu2::utils::into_mc(&program);
///
/// assert_eq!(&code, "1010000000000001\n0010000100100011\n");
/// ```
pub fn into_mc(instructions: &[Instruction]) -> String {
	let mut output = String::with_capacity(instructions.len() * 17);
	
	for instruction in instructions.iter() {
		write!(output, "{:016b}\n", instruction.as_word()).unwrap();
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

/// Single character encoded using BatPU-2 character encoding:
///
/// ```text
/// 0 - SPACE
/// 1 - 26 - A - Z
/// 27 - .
/// 28 - !
/// 29 - ?
/// ```
///
/// ### Invalid Characters
///
/// `Char` can be constructed using an u8 that does not correspond to any character in the character
/// table lays outside. Such invalid characters are not printed by redstone text screen(needs checking)
/// but can still be generated by a program.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct Char(u8);

impl Char {
	/// BatPU-2 character table(uppercase).
	pub const TABLE: [char; 30] = [' ', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '.', '!', '?'];
	pub const SPACE: Char = Char(0);
	
	/// Constructs new character from its code.
	pub fn new(value: u8) -> Self {
		Self(value)
	}
	
	/// Converts character to its `char` representation. Returns `None` if the character code lays outside the character table.
	pub fn to_char(self) -> Option<char> {
		Char::TABLE.get(self.as_u8() as usize)
		           .copied()
	}
	
	/// Returns character code
	pub fn as_u8(self) -> u8 {
		self.0
	}
	
	/// Checks if the character lays within the character table
	pub fn is_valid(self) -> bool {
		(self.as_u8() as usize) < Char::TABLE.len()
	}
}

impl TryFrom<char> for Char {
	type Error = InvalidCharacter;
	
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

/// An error which can be returned when loading .mc code
#[derive(Error, Debug)]
#[error("{line_number}: Cannot parse \"{line}\"")]
pub struct FromMcError {
	line_number: usize,
	line: String,
	#[source] source: ParseIntError,
}

impl FromMcError {
	pub fn line_number(&self) -> usize {
		self.line_number
	}
	
	pub fn line(&self) -> &str {
		self.line.as_str()
	}
}

/// An error which can be returned when converting [`char`] to [`Char`]
#[derive(Error, Debug)]
#[error("Invalid character \"{char}\"")]
pub struct InvalidCharacter { char: char }

impl InvalidCharacter {
	pub fn char(&self) -> char {
		self.char
	}
}
