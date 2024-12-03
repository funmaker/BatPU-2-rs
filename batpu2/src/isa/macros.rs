
macro_rules! operand_type {
	( a ) => { u8 };
	( b ) => { u8 };
	( c ) => { u8 };
	( imm ) => { u8 };
	( addr ) => { u16 };
	( cond ) => { Cond };
	( offset ) => { u8 };
}

pub(crate) use operand_type;

macro_rules! operand_from {
	( a, $val:expr ) => { $val as u8 };
	( b, $val:expr ) => { $val as u8 };
	( c, $val:expr ) => { $val as u8 };
	( imm, $val:expr ) => { $val as u8 };
	( addr, $val:expr ) => { $val as u16 };
	( cond, $val:expr ) => {
		match $val {
			0b_00 => Cond::Zero,
			0b_01 => Cond::NotZero,
			0b_10 => Cond::Carry,
			0b_11 => Cond::NotCarry,
			_ => unreachable!(),
		}
	};
	( offset, $val:expr ) => { $val as u8 };
}

pub(crate) use operand_from;

macro_rules! operand_mask {
	( a ) =>      { 0b_0000_1111_0000_0000_u16 };
	( b ) =>      { 0b_0000_0000_1111_0000_u16 };
	( c ) =>      { 0b_0000_0000_0000_1111_u16 };
	( imm ) =>    { 0b_0000_0000_1111_1111_u16 };
	( addr ) =>   { 0b_0000_0011_1111_1111_u16 };
	( cond ) =>   { 0b_0000_1100_0000_0000_u16 };
	( offset ) => { 0b_0000_0000_0000_1111_u16 };
}

pub(crate) use operand_mask;

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
			use std::fmt::{Display, Formatter};
			use $crate::isa::macros::*;
			
			#[repr(u8)]
			#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
			pub enum Cond {
				Zero     = 0b_00,
				NotZero  = 0b_01,
				Carry    = 0b_10,
				NotCarry = 0b_11,
			}
			
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
				type Error = String;
				
				fn try_from(name: &str) -> Result<Self, String> {
					match name {
						$(
							stringify!($mnemonic) => Ok(Mnemonic::$mnemonic),
						)*
						$($(
							stringify!($alias) => Ok(Mnemonic::$alias),
						)*)?
						name => Err(format!("Unknown mnemonic \"{name}\"")),
					}
				}
			}
			
			impl TryFrom<u8> for Mnemonic {
				type Error = String;
				
				fn try_from(opcode: u8) -> Result<Self, String> {
					match opcode {
						$(
							$opcode => Ok(Mnemonic::$mnemonic),
						)*
						_ => Err(format!("Unknown opcode {opcode}")),
					}
				}
			}
			
			#[derive(Debug, Copy, Clone, Eq, PartialEq)]
			pub enum Instruction {
				$( $mnemonic $({$( $operand: operand_type!($operand), )*})?, )*
			}
			
			impl Instruction {
				pub fn new<Ops>(mnemonic: Mnemonic, operands: Ops) -> Result<Self, String>
				where Ops: IntoIterator,
				      Ops::IntoIter: Iterator<Item = u16> + ExactSizeIterator {
					let mut operands = operands.into_iter();
					
					if operands.len() != mnemonic.operand_count() {
						return Err(format!("Invalid number of operands for {mnemonic}, expected {}, got {}", mnemonic.operand_count(), operands.len()));
					}
					
					match mnemonic {
						$(
							Mnemonic::$mnemonic => Ok(Instruction::$mnemonic $({$(
							   $operand: operand_from!($operand, operands.next().unwrap()),
							)*})?),
						)*
						$($(
							Mnemonic::$alias => {
								$( let $alias_op = operands.next().unwrap(); )*
								Instruction::new(Mnemonic::$target, [$( $target_op ),*])
							},
						)*)?
					}
				}
				
				pub fn as_u16(self) -> u16 {
					self.into()
				}
			}
			
			impl Into<u16> for Instruction {
				fn into(self) -> u16 {
					match self {
						$(
							Instruction::$mnemonic $({$( $operand ),*})? => $opcode << 12 $($(
								| ($operand as u16) << operand_mask!($operand).trailing_zeros() & operand_mask!($operand)
							)*)?,
						)*
					}
				}
			}
			
			impl From<u16> for Instruction {
				fn from(value: u16) -> Self {
					let mnemonic = Mnemonic::try_from((value >> 12) as u8).unwrap();
					
					match mnemonic {
						$(
							Mnemonic::$mnemonic => Instruction::$mnemonic $({$(
							   $operand: operand_from!($operand, (value & operand_mask!($operand)) >> operand_mask!($operand).trailing_zeros()),
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
