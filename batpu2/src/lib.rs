#![feature(integer_sign_cast)]
#![feature(debug_closure_helpers)]

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
	
	#[test]
	#[cfg(feature = "embedded_io")]
	fn hello_world() {
		let code: Vec<u16> = vec![
			0x8ff9, 0xff00, 0x8ff7, 0x8e08, 0xffe0, 0x8e05, 0xffe0, 0x8e0c, 0xffe0, 0x8e0c, 0xffe0, 0x8e0f, 0xffe0,
			0x8e17, 0xffe0, 0x8e0f, 0xffe0, 0x8e12, 0xffe0, 0x8e0c, 0xffe0, 0x8e04, 0xffe0, 0x8ff8, 0xff00, 0x1000
		];
		
		let mut vm = BatPU2::new(code);
		assert_eq!(vm.step_multiple(30), 26);
		assert_eq!(vm.io.char_display.to_string(), String::from("HELLOWORLD"));
	}
}
