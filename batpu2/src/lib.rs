#![feature(integer_sign_cast)]

mod vm;
mod char;

pub use vm::BatPU2;
pub use vm::flags::Flags;
pub use vm::io::{IO, RawIO};
pub use char::Char;

#[cfg(feature = "embedded_io")]
pub use vm::io::embedded;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "embedded_io")]
    fn it_works() {
        BatPU2::new([0, 0, 1, 2, 3]);
    }
}
