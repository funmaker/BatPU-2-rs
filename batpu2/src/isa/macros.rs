
macro_rules! operand_type {
	( a ) => { u8 };
	( b ) => { u8 };
	( c ) => { u8 };
	( imm ) => { u8 };
	( addr ) => { u16 };
	( cond ) => { u8 };
	( offset ) => { u8 };
}

pub(crate) use operand_type;

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
			use $crate::asm::ast;
			use $crate::isa::macros::*;
			
			// #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
			// pub(super) enum Cond {
			// 	Zero,
			// 	NotZero,
			// 	Carry,
			// 	NotCarry,
			// }
			
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
			
			#[derive(Debug, Copy, Clone, Eq, PartialEq)]
			pub enum Instruction {
				$( $mnemonic $( { $( $operand: operand_type!($operand), )* } )?, )*
			}
			
			impl TryFrom<&ast::ResolvedInstruction> for Instruction {
				type Error = String;
				
				fn try_from(resolved: &ast::ResolvedInstruction) -> Result<Self, String> {
					let mut operands = resolved.operands.iter();
					
					match &*resolved.mnemonic {
						$(
							stringify!($mnemonic) => {
								if resolved.operands.len() != Mnemonic::$mnemonic.operand_count() {
									return Err(format!(concat!("Invalid number of operands for ", stringify!($mnemonic), ", expected {}, got {}"), Mnemonic::$mnemonic.operand_count(), resolved.operands.len()));
								}
								
								Ok(
									Instruction::$mnemonic $( { $(
									   $operand: operands.next().copied().unwrap() as operand_type!($operand),
									)* } )?
								)
							}
						)*
						$($(
							stringify!($alias) => {
								let _ = Mnemonic::$alias.operand_count();
								unimplemented!()
							},
						)*)?
						_ => {
							return Err(format!("Unknown mnemonic \"{}\"", resolved.mnemonic));
						}
					}
				}
			}
		}
		
		$vis use generated::*;
	};
}

pub(crate) use isa;
