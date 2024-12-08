#![allow(dead_code)]
#![feature(integer_sign_cast)]
#![feature(debug_closure_helpers)]
#![feature(never_type)]
#![cfg_attr(feature = "doc_cfg", feature(doc_auto_cfg))]

pub mod vm;
pub mod asm;
pub mod isa;
pub mod utils;

pub use vm::BatPU2;

#[cfg(test)]
#[cfg(feature = "embedded_io")]
mod tests {
	use crate::BatPU2;
	
	#[test]
	fn it_works() {
		BatPU2::new([0, 0, 1, 2, 3]);
		BatPU2::new(&[0, 0, 1, 2, 3]);
		BatPU2::new(&[0, 0, 1, 2, 3][..]);
		BatPU2::new(vec![0, 0, 1, 2, 3]);
		BatPU2::new(std::sync::Arc::new([0, 0, 1, 2, 3]));
	}
	
	#[test]
	fn hello_world() {
		let code = [
			0x8ff9, 0xff00, 0x8ff7, 0x8e08, 0xffe0, 0x8e05, 0xffe0, 0x8e0c, 0xffe0, 0x8e0c, 0xffe0, 0x8e0f, 0xffe0,
			0x8e17, 0xffe0, 0x8e0f, 0xffe0, 0x8e12, 0xffe0, 0x8e0c, 0xffe0, 0x8e04, 0xffe0, 0x8ff8, 0xff00, 0x1000
		];
		
		let mut vm = BatPU2::new(code);
		assert_eq!(vm.step_multiple(30), 26);
		assert_eq!(vm.io.char_display.to_string(), "HELLOWORLD");
	}
	
	#[test]
	fn hello_asm() {
		let vm = BatPU2::from_asm(r"
			LDI r2 write_char
			LDI r3 buffer_chars
			
			LDI r1 'H'
			STR r2 r1 0
			LDI r1 'E'
			STR r2 r1 0
			LDI r1 'L'
			STR r2 r1 0
			STR r2 r1 0
			LDI r1 'O'
			STR r2 r1 0
			LDI r1 ' '
			STR r2 r1 0
			LDI r1 'A'
			STR r2 r1 0
			LDI r1 'S'
			STR r2 r1 0
			LDI r1 'M'
			STR r2 r1 0
			
			STR r3 r1 0
		").unwrap();
		
		assert_eq!(vm.io.char_display.to_string(), "HELLO ASM ")
	}

	#[test]
	fn dvd() {
		let code = [
			0x8ff9, 0xff00, 0x8ff7, 0x8104, 0xff10, 0x8116, 0xff10, 0x8104, 0xff10, 0x8100, 0xff10, 0x8100, 0xff10,
			0x8100, 0xff10, 0x8100, 0xff10, 0x8100, 0xff10, 0x8100, 0xff10, 0x8100, 0xff10, 0x8ff8, 0xff00, 0x8100,
			0x824f, 0xf120, 0x9101, 0x82c9, 0xf120, 0x9101, 0x82e6, 0xf120, 0x9101, 0x82e0, 0xf120, 0x9101, 0x82e7,
			0xf120, 0x9101, 0x82a8, 0xf120, 0x9101, 0x82e7, 0xf120, 0x9101, 0x82e0, 0xf120, 0x9101, 0x82ef, 0xf120,
			0x9101, 0x8269, 0xf120, 0x9101, 0x8246, 0xf120, 0x9101, 0x8100, 0x8200, 0x8301, 0x8401, 0x8cf0, 0x8df1,
			0x8ef2, 0x8f14, 0x8bf6, 0xfb00, 0xc04d, 0x8bf5, 0xfb00, 0xc063, 0xc066, 0x3f1f, 0xb843, 0x1000, 0x8800,
			0x890b, 0x8a01, 0xe870, 0x8608, 0x2818, 0xfc80, 0x3818, 0x96ff, 0x57a0, 0xb05c, 0x2626, 0xfd60, 0x3626,
			0xfe00, 0x7707, 0x2606, 0xb455, 0x9801, 0x3890, 0xb450, 0xd000, 0x2131, 0x2242, 0xd000, 0x8515, 0x8618,
			0x3150, 0xb071, 0x3100, 0xb071, 0x3260, 0xb073, 0x3200, 0xb073, 0xd000, 0x3033, 0xa06c, 0x3044, 0xd000,
		];

		let mut vm = BatPU2::new(code);

		let done_steps = vm.step_multiple(10000);
		assert_eq!(done_steps, 5102);
		assert_eq!(vm.io.char_display.to_string(), "DVD       ");
		assert_eq!(vm.io.number_display.value, None);
		assert_eq!(vm.io.number_display.signed, false);
		assert_eq!(vm.io.screen.x, 15);
		assert_eq!(vm.io.screen.y, 6);
		assert_eq!(vm.io.screen.output, [0, 0, 0, 0, 0, 16320, 64480, 32640, 0, 25696, 43680, 43680, 27232, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
	}
}
