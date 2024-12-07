use std::fmt::{Display, Formatter};
use std::ops::Deref;
use arrayvec::ArrayVec;

use crate::isa::MAX_ARGS;

#[derive(Debug, Clone)]
pub struct Line<'a> {
	pub line_number: usize,
	pub label: Option<Token<'a>>,
	pub mnemonic: Option<Token<'a>>,
	pub args: ArrayVec<Token<'a>, MAX_ARGS>,
	pub comment: Option<Token<'a>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Token<'a> {
	pub char_number: usize,
	pub span: &'a str,
}

impl<'a> Token<'a> {
	pub fn new(char_number: usize, span: &'a str) -> Token<'a> {
		Self {
			char_number,
			span,
		}
	}
}

impl Deref for Token<'_> {
	type Target = str;
	
	fn deref(&self) -> &Self::Target {
		self.span
	}
}

impl PartialEq<str> for Token<'_> {
	fn eq(&self, other: &str) -> bool {
		self.span == other
	}
}

impl Display for Token<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.span.fmt(f)
	}
}

pub struct ResolvedInstruction {
	pub mnemonic: String,
	pub operands: ArrayVec<u16, MAX_ARGS>,
}
