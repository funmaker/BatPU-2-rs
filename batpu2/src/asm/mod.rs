use std::ops::RangeInclusive;
use std::num::ParseIntError;
use arrayvec::ArrayVec;
use thiserror::Error;

mod ast;
mod parser;
mod assembler;

pub use ast::*;
pub use parser::*;
pub use assembler::*;

use crate::isa::{MAX_ARGS, MAX_CODE_LEN};
use crate::utils::PrettyRange;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AsmError<'a> {
	#[error("Unexpected token `{token}`, too many operands (max {MAX_ARGS})")]
	TooManyTokens {
		line_number: usize,
		token: Token<'a>,
	},
	#[error("Instruction `{mnemonic}` expects {} operands (got {})", PrettyRange(expected), args.len())]
	WrongOperandCount {
		line_number: usize,
		expected: RangeInclusive<usize>,
		mnemonic: Token<'a>,
		args: ArrayVec<Token<'a>, MAX_ARGS>,
	},
	#[error("{mnemonic}'s {}. operand {name} value out of range (min {min}, max {}, got {got})", operand + 1, max - 1)]
	OperandOutOfRange {
		line_number: usize,
		operand: usize,
		mnemonic: Token<'a>,
		name: &'static str,
		min: i16,
		max: i16,
		got: i16,
		token: Token<'a>,
	},
	#[error("Unexpected token `{token}`, too many instructions (max {MAX_CODE_LEN}).")]
	TooManyInstructions {
		line_number: usize,
		token: Token<'a>,
	},
	#[error("Unexpected token `{token}`, expected a mnemonic or `define`")]
	UnknownMnemonic {
		line_number: usize,
		token: Token<'a>,
	},
	#[error("Unexpected token `{token}`, expected {}", if *literal { "a known symbol or an integer literal" } else { "a known symbol" })]
	UnknownSymbol {
		line_number: usize,
		token: Token<'a>,
		literal: bool,
	},
	#[error("Unexpected token `{token}`, expected an integer literal")]
	IntParseError {
		line_number: usize,
		token: Token<'a>,
		#[source] source: ParseIntError,
	},
}

impl AsmError<'_> {
	pub fn line_num(&self) -> usize {
		match self {
			&AsmError::TooManyTokens { line_number, .. } => line_number,
			&AsmError::WrongOperandCount { line_number, .. } => line_number,
			&AsmError::OperandOutOfRange { line_number, .. } => line_number,
			&AsmError::TooManyInstructions { line_number, .. } => line_number,
			&AsmError::UnknownMnemonic { line_number, .. } => line_number,
			&AsmError::UnknownSymbol { line_number, .. } => line_number,
			&AsmError::IntParseError { line_number, .. } => line_number,
		}
	}
	
	pub fn col_num(&self) -> usize {
		self.token().char_number
	}
	
	pub fn token(&self) -> Token {
		match self {
			&AsmError::TooManyTokens { token, .. } => token,
			&AsmError::WrongOperandCount { mnemonic, .. } => mnemonic,
			&AsmError::OperandOutOfRange { token, .. } => token,
			&AsmError::TooManyInstructions { token, .. } => token,
			&AsmError::UnknownMnemonic { token, .. } => token,
			&AsmError::UnknownSymbol { token, .. } => token,
			&AsmError::IntParseError { token, .. } => token,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn ast_parses() {
		let code = r"
		.start
		  MOV 1 2 3  # comment text
		  LDI r1 ' '
		  TEST < .start";
		
		let _ast: Vec<_> = parse_lines(code).collect();
	}
	
	#[test]
	fn assembler_assembles() {
		let code = r"
		.start
		  MOV 1 5 # comment text
	    .second
		  LDI r1 ' '
		  BRH < .start";
		
		let _ast: Vec<_> = parse_lines(code).collect();
	}
	
	#[test]
	fn assembler_assembles_2() {
		let code = r"
		ldi r15 buffer_chars
		define write -1
		LDI r4 'D'
		STR r15 r4 write";
		
		let _ast: Vec<_> = parse_lines(code).collect();
	}
}

