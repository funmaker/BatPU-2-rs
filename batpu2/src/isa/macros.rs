
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
        mod isa {
            use $crate::isa::macros::*;
            
            #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
            pub(super) enum Cond {
                Zero,
                NotZero,
                Carry,
                NotCarry,
            }
            
            #[derive(Debug, Copy, Clone, Eq, PartialEq)]
            pub(super) enum Mnemonic {
                $( $mnemonic, )*
                $($( $alias, )*)?
            }
            
            #[derive(Debug, Copy, Clone, Eq, PartialEq)]
            pub(super) enum Instruction {
                $( $mnemonic $( { $( $operand: operand_type!($operand), )* } )?, )*
            }
        }
        
        $vis use isa::*;
    };
}

pub(crate) use isa;
