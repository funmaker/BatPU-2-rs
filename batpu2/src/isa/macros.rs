
macro_rules! count {
    () => { 0usize };
    ($_head:tt $($tail:tt)*) => { 1usize + count!($($tail)*) };
}

pub(crate) use count;

macro_rules! map_kind {
    ( unsigned ) => { $crate::isa::common::OperandKind::Unsigned };
    ( signed ) => { $crate::isa::common::OperandKind::Signed };
    ( any ) => { $crate::isa::common::OperandKind::Any };
    ( address ) => { $crate::isa::common::OperandKind::Address };
}

pub(crate) use map_kind;

macro_rules! isa {
	(
		$word_vis:vis word = $word_ty:ty;
		$val_vis:vis operand = $val_ty:ty;
		
		$op_vis:vis operands {
			$(
				$op_name:ident : $op_type:ty = $op_kind:tt $op_mask:expr
			),* $(,)?
		}
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
			
			
			$word_vis type Word = $word_ty;
			$val_vis type Operand = $val_ty;
			
			$(
				pub mod $op_name {
					use super::super::*;
					use super::*;
					
					pub type Type = $op_type;
					pub const NAME: &'static str = stringify!($op_name);
					pub const MASK: $word_ty = $op_mask;
					pub const KIND: OperandKind = map_kind!($op_kind);
				}
			)*
			
			pub const MAX_ARGS: usize = {
				let mut max = 0;
				$($( if count!($($operand)*) > max { max = count!($($operand)*); } )?)*
				$($( if count!($($alias_op)*) > max { max = count!($($alias_op)*); } )*)?
				max
			};
			pub const MAX_CODE_LEN: usize = 1 << addr::MASK.count_ones();
			
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
			
			impl TryFrom<Operand> for Mnemonic {
				type Error = UnknownOpcodeError;
				
				fn try_from(opcode: Operand) -> Result<Self, UnknownOpcodeError> {
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
				$( $mnemonic $({$( $operand: $operand::Type, )*})?, )*
			}
			
			impl Instruction {
				pub fn new<Ops>(mnemonic: Mnemonic, operands: Ops) -> Result<Self, InstructionError>
				where Ops: IntoIterator,
				      Ops::IntoIter: Iterator<Item = Operand> + ExactSizeIterator {
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
										check_range($operand, $operand::MASK, $operand::KIND, operand, $operand::NAME)?;
										let $operand = $operand.try_into().unwrap();
										operand += 1;
									)*
								)?
								Ok(Instruction::$mnemonic $({$( $operand ),*})?)
							}
						)*
						$($(
							Mnemonic::$alias => {
								let mut operand = 0;
								$(
									let $alias_op = operands.next().unwrap();
									check_range($alias_op, $alias_op::MASK, $alias_op::KIND, operand, $alias_op::NAME)?;
									operand += 1;
								)*
								Instruction::new(Mnemonic::$target, [$( $target_op ),*])
							}
						)*)?
					}
				}
				
				pub fn as_word(self) -> Word {
					self.into()
				}
			}
			
			impl From<Instruction> for Word {
				fn from(val: Instruction) -> Word {
					match val {
						$(
							Instruction::$mnemonic $({$( $operand ),*})? => write_masked($opcode, opcode::MASK) $($( | write_masked($operand.try_into().unwrap(), $operand::MASK) )*)?,
						)*
					}
				}
			}
			
			impl From<Word> for Instruction {
				fn from(value: Word) -> Self {
					let mnemonic = Mnemonic::try_from(read_masked(value, opcode::MASK, opcode::KIND)).unwrap();
					
					match mnemonic {
						$(
							Mnemonic::$mnemonic => Instruction::$mnemonic $({$(
							   $operand: read_masked(value, $operand::MASK, $operand::KIND).try_into().unwrap(),
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
