use flags::Flags;
use io::IO;

#[cfg(feature = "embedded_io")]
use io::EmbeddedIO;

pub mod flags;
pub mod io;

pub struct BatPU2<T, I> {
	pub flags: Flags,
	pub io: I,
	pub pc: u16,
	pub registers: [u8; 15],
	pub call_stack: [u16; 16],
	pub memory: [u8; 240],
	pub code: T,
}

#[cfg(feature = "embedded_io")]
impl<T> BatPU2<T, EmbeddedIO>
where T: AsRef<[u16]> {
	pub fn new(code: T) -> Self {
		Self {
			flags: Flags::default(),
			io: EmbeddedIO::new(),
			pc: 0,
			registers: [0; 15],
			call_stack: [0; 16],
			memory: [0; 240],
			code,
		}
	}
}

impl<T, I> BatPU2<T, I>
where T: AsRef<[u16]>,
      I: IO {
	pub fn with_io(code: T, io: I) -> Self {
		Self {
			flags: Flags::default(),
			io,
			pc: 0,
			registers: [0; 15],
			call_stack: [0; 16],
			memory: [0; 240],
			code,
		}
	}
	
	pub fn step() {
		
	}
}
