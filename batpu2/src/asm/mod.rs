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

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AsmError<'a> {
	#[error("{line_number}:{}: Unexpected token `{token}`, too many arguments (max {MAX_ARGS})", token.char_number)]
	TooManyTokens {
		line_number: usize,
		token: Token<'a>,
	},
	#[error("{line_number}:{}: Instruction `{mnemonic}` expects {expected} arguments (got {})", mnemonic.char_number, args.len())]
	WrongArgumentsCount {
		line_number: usize,
		expected: usize,
		mnemonic: Token<'a>,
		args: ArrayVec<Token<'a>, MAX_ARGS>,
	},
	#[error("{line_number}:{}: Unexpected token `{token}`, too many instructions (max {MAX_CODE_LEN}).", token.char_number)]
	TooManyInstructions {
		line_number: usize,
		token: Token<'a>,
	},
	#[error("{line_number}:{}: Unexpected token `{token}`, expected a mnemonic or `define`", token.char_number)]
	UnknownMnemonic {
		line_number: usize,
		token: Token<'a>,
	},
	#[error("{line_number}:{}: Unexpected token `{token}`, expected an integer", token.char_number)]
	IntParseError {
		line_number: usize,
		token: Token<'a>,
		#[source] source: ParseIntError,
	},
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
		
		let ast: Vec<_> = parse_lines(code).collect();
		println!("{:#?}", ast);
	}
	
	#[test]
	fn assembler_assembles() {
		let code = r"
		.start
		  MOV 1 5 # comment text
	    .second
		  LDI r1 ' '
		  BRH < .start";
		
		let ast: Vec<_> = parse_lines(code).collect();
		println!("{:#?}", ast);
	}
	
	#[test]
	fn assembler_assembles_2() {
		let code = r"
		ldi r15 buffer_chars
		define write -1
		LDI r4 'D'
		STR r15 r4 write";
		
		let ast: Vec<_> = parse_lines(code).collect();
		println!("{:#?}", ast);
	}
}

