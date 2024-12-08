
macro_rules! operand_type {
	( opcode ) => { u8   };
	( a      ) => { u8   };
	( b      ) => { u8   };
	( c      ) => { u8   };
	( imm    ) => { i8   };
	( addr   ) => { u16  };
	( cond   ) => { Cond };
	( offset ) => { i8   };
}

pub(crate) use operand_type;

macro_rules! operand_mask {
	( opcode ) => { 0b_1111_0000_0000_0000_u16 };
	( a      ) => { 0b_0000_1111_0000_0000_u16 };
	( b      ) => { 0b_0000_0000_1111_0000_u16 };
	( c      ) => { 0b_0000_0000_0000_1111_u16 };
	( imm    ) => { 0b_0000_0000_1111_1111_u16 };
	( addr   ) => { 0b_0000_0011_1111_1111_u16 };
	( cond   ) => { 0b_0000_1100_0000_0000_u16 };
	( offset ) => { 0b_0000_0000_0000_1111_u16 };
}

pub(crate) use operand_mask;

macro_rules! operand_signed {
	( opcode ) => { false };
	( a      ) => { false };
	( b      ) => { false };
	( c      ) => { false };
	( imm    ) => { true  };
	( addr   ) => { false };
	( cond   ) => { false };
	( offset ) => { true  };
}

pub(crate) use operand_signed;

macro_rules! operand_from {
	( cond, $val:expr ) => {
		match $val {
			0b_00 => Cond::Zero,
			0b_01 => Cond::NotZero,
			0b_10 => Cond::Carry,
			0b_11 => Cond::NotCarry,
			_ => unreachable!(),
		}
	};
	( $operand:tt, $val:expr ) => { $val as operand_type!($operand) };
}

pub(crate) use operand_from;

macro_rules! count {
    () => { 0usize };
    ($_head:tt $($tail:tt)*) => { 1usize + count!($($tail)*) };
}

pub(crate) use count;

macro_rules! isa {
	(
		$vis:vis instructions {
			$(
				$mnemonic:ident $( ( $( $operand:tt ),* $(,)? ) )? = $opcode:expr
			),* $(,)?
		}
		$(
			$_:vis aliases {
				$(
					$alias:ident ( $( $alias_op:tt ),* $(,)? ) => $target:ident ( $( $target_op:expr ),* $(,)? )
				),* $(,)?
			}
		)?
	) => {
		mod generated {
			#![allow(unused_assignments)]
			
			use std::fmt::{Display, Formatter};
			use $crate::isa::macros::*;
			use $crate::isa::common::*;
			
			pub const MAX_ARGS: usize = {
				let mut max = 0;
				$($( if count!($($operand)*) > max { max = count!($($operand)*); } )?)*
				$($( if count!($($alias_op)*) > max { max = count!($($alias_op)*); } )*)?
				max
			};
			pub const MAX_CODE_LEN: usize = 1 << operand_mask!( addr ).count_ones();
			
			#[derive(Debug, Copy, Clone, Eq, PartialEq)]
			pub enum Mnemonic {
				$( $mnemonic, )*
				$($( $alias, )*)?
			}
			
			impl Mnemonic {
				const fn operand_count(self) -> usize {
					match self {
						$( Self::$mnemonic => count!($($( $operand )*)?), )*
						$($( Self::$alias => count!($( $alias_op )*), )*)?
					}
				}
			}
			
			impl Display for Mnemonic {
				fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
					match self {
						$( Self::$mnemonic => stringify!($mnemonic).fmt(f), )*
						$($( Self::$alias => stringify!($alias).fmt(f), )*)?
					}
				}
			}
			
			impl TryFrom<&str> for Mnemonic {
				type Error = UnknownMnemonicError;
				
				fn try_from(name: &str) -> Result<Self, UnknownMnemonicError> {
					match name {
						$(
							stringify!($mnemonic) => Ok(Mnemonic::$mnemonic),
						)*
						$($(
							stringify!($alias) => Ok(Mnemonic::$alias),
						)*)?
						_ => Err(UnknownMnemonicError),
					}
				}
			}
			
			impl TryFrom<u8> for Mnemonic {
				type Error = UnknownOpcodeError;
				
				fn try_from(opcode: u8) -> Result<Self, UnknownOpcodeError> {
					match opcode {
						$(
							$opcode => Ok(Mnemonic::$mnemonic),
						)*
						_ => Err(UnknownOpcodeError(opcode)),
					}
				}
			}
			
			#[derive(Debug, Copy, Clone, Eq, PartialEq)]
			pub enum Instruction {
				$( $mnemonic $({$( $operand: operand_type!($operand), )*})?, )*
			}
			
			impl Instruction {
				pub fn new<Ops>(mnemonic: Mnemonic, operands: Ops) -> Result<Self, InstructionError>
				where Ops: IntoIterator,
				      Ops::IntoIter: Iterator<Item = i16> + ExactSizeIterator {
					let mut operands = operands.into_iter();
					
					if operands.len() != mnemonic.operand_count() {
						return Err(InstructionError::WrongOperandCount { expected: mnemonic.operand_count(), got: operands.len() });
					}
					
					match mnemonic {
						$(
							Mnemonic::$mnemonic => {
								$(
									let mut operand = 0;
									$(
										let $operand = operands.next().unwrap();
										check_range($operand, operand_mask!($operand), operand_signed!($operand), operand, stringify!($operand))?;
										let $operand = operand_from!($operand, $operand);
										operand += 1;
									)*
								)?
								Ok(Instruction::$mnemonic $({$( $operand ),*})?)
							},
						)*
						$($(
							Mnemonic::$alias => {
								let mut operand = 0;
								$(
									let $alias_op = operands.next().unwrap();
									check_range($alias_op, operand_mask!($alias_op), operand_signed!($alias_op), operand, stringify!($alias_op))?;
									operand += 1;
								)*
								Instruction::new(Mnemonic::$target, [$( $target_op ),*])
							},
						)*)?
					}
				}
				
				pub fn as_u16(self) -> u16 {
					self.into()
				}
			}
			
			impl From<Instruction> for u16 {
				fn from(val: Instruction) -> u16 {
					match val {
						$(
							Instruction::$mnemonic $({$( $operand ),*})? => write_masked($opcode, operand_mask!(opcode)) $($( | write_masked($operand as i16, operand_mask!($operand)) )*)?,
						)*
					}
				}
			}
			
			impl From<u16> for Instruction {
				fn from(value: u16) -> Self {
					let mnemonic = Mnemonic::try_from(read_masked(value, operand_mask!(opcode), operand_signed!(opcode)) as operand_type!(opcode)).unwrap();
					
					match mnemonic {
						$(
							Mnemonic::$mnemonic => Instruction::$mnemonic $({$(
							   $operand: operand_from!($operand, read_masked(value, operand_mask!($operand), operand_signed!($operand))),
							)*})?,
						)*
						_ => unreachable!(),
					}
				}
			}
		}
		
		$vis use generated::*;
	};
}

pub(crate) use isa;
