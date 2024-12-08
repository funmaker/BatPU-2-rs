use thiserror::Error;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Cond {
	Zero     = 0b_00,
	NotZero  = 0b_01,
	Carry    = 0b_10,
	NotCarry = 0b_11,
}

pub(crate) fn check_range(value: i16, mask: u16, signed: bool, operand: usize, name: &'static str) -> Result<(), InstructionError> {
	debug_assert!(mask != 0);
	debug_assert!(mask != !0);
	
	// TODO: BetterBool
	let range = if signed {
		-( 1 << (mask.count_ones() - 1))..(1 << mask.count_ones())
	} else {
		0..(1 << mask.count_ones())
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

pub(crate) fn write_masked(mut value: i16, mask: u16) -> u16 {
	debug_assert!(mask != 0);
	debug_assert!(mask != !0);
	
	if value < 0 {
		let umax = 1 << mask.count_ones();
		
		value = umax + value;
	}
	
	(value.cast_unsigned() << mask.trailing_zeros()) & mask
}

pub(crate) fn read_masked(value: u16, mask: u16, signed: bool) -> i16 {
	debug_assert!(mask != 0);
	debug_assert!(mask != !0);
	
	let value = (value & mask) >> mask.trailing_zeros();
	
	if signed {
		let imax = 1 << (mask.count_ones() - 1);
		
		if value >= imax { (imax as i16 * -2) + value as i16 }
		else { value as i16 }
	} else {
		value.cast_signed()
	}
}

#[derive(Error, Debug)]
#[error("Unknown Mnemonic")]
pub struct UnknownMnemonicError;

#[derive(Error, Debug)]
#[error("Unknown Opcode({0})")]
pub struct UnknownOpcodeError(pub u8);

#[derive(Error, Debug)]
pub enum InstructionError {
	#[error("Invalid argument count. Expected {expected} operands (got {got})")]
	WrongOperandCount { expected: usize, got: usize },
	#[error("{}. operand {name} is out of range. value out of range (min {min}, max {}, got {got})", operand + 1, max - 1)]
	OperandOutOfRange { operand: usize, name: &'static str, min: i16, max: i16, got: i16 },
}