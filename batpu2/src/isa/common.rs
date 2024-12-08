use std::ops::RangeInclusive;
use thiserror::Error;

use crate::utils::PrettyRange;
use super::{Operand, Word};

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Cond {
	Zero     = 0b_00,
	NotZero  = 0b_01,
	Carry    = 0b_10,
	NotCarry = 0b_11,
}

impl From<Operand> for Cond {
	fn from(value: Operand) -> Self {
		match value {
			x if x == Cond::Zero as Operand => Cond::Zero,
			x if x == Cond::NotZero as Operand => Cond::NotZero,
			x if x == Cond::Carry as Operand => Cond::Carry,
			x if x == Cond::NotCarry as Operand => Cond::NotCarry,
			_ => panic!("Value {value} is out range for Cond"),
		}
	}
}

impl Into<Operand> for Cond {
	fn into(self) -> Operand {
		self as Operand
	}
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum OperandKind {
	Unsigned,
	Signed,
	Any,
}

pub(crate) fn check_range(value: Operand, mask: Word, kind: OperandKind, operand: usize, name: &'static str) -> Result<(), InstructionError> {
	debug_assert!(mask != 0);
	debug_assert!(mask != !0);
	
	let umax = 1 << mask.count_ones();
	let range = match kind {
		OperandKind::Unsigned => 0..umax,
		OperandKind::Signed => (umax/-2)..(umax/2),
		OperandKind::Any => (umax/-2)..umax,
	};
	
	if !range.contains(&value) {
		return Err(InstructionError::OperandOutOfRange {
			operand,
			name,
			min: range.start,
			max: range.end,
			got: value,
		});
	}
	
	Ok(())
}

pub(crate) fn write_masked(mut value: Operand, mask: Word) -> Word {
	debug_assert!(mask != 0);
	debug_assert!(mask != !0);
	
	if value < 0 {
		let umax = 1 << mask.count_ones();
		
		value = umax + value;
	}
	
	((value as Word) << mask.trailing_zeros()) & mask
}

pub(crate) fn read_masked(value: Word, mask: Word, kind: OperandKind) -> Operand {
	debug_assert!(mask != 0);
	debug_assert!(mask != !0);
	
	let value = (value & mask) >> mask.trailing_zeros();
	
	if kind == OperandKind::Signed {
		let imax = 1 << (mask.count_ones() - 1);
		
		if value >= imax { (imax as Operand * -2) + value as Operand }
		else { value as Operand }
	} else {
		value as Operand
	}
}

#[derive(Error, Debug)]
#[error("Unknown Mnemonic")]
pub struct UnknownMnemonicError;

#[derive(Error, Debug)]
#[error("Unknown Opcode({0})")]
pub struct UnknownOpcodeError(pub Operand);

#[derive(Error, Debug)]
pub enum InstructionError {
	#[error("Invalid argument count. Expected {} operands (got {got})", PrettyRange(expected))]
	WrongOperandCount { expected: RangeInclusive<usize>, got: usize },
	#[error("{}. operand {name} is out of range. value out of range (min {min}, max {}, got {got})", operand + 1, max - 1)]
	OperandOutOfRange { operand: usize, name: &'static str, min: Operand, max: Operand, got: Operand },
}