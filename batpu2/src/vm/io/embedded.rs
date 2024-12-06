use std::fmt::{Debug, Display, Formatter};
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};

use super::char::Char;
use super::IO;

#[derive(Clone)]
pub struct EmbeddedIO {
	pub screen: Screen,
	pub char_display: CharDisplay,
	pub number_display: NumberDisplay,
	pub rng: SmallRng,
	pub controller: Controller,
}

impl EmbeddedIO {
	pub fn new() -> Self {
		Self {
			screen: Screen::default(),
			char_display: CharDisplay::default(),
			number_display: NumberDisplay::default(),
			rng: SmallRng::from_entropy(),
			controller: Controller::default(),
		}
	}
}

impl Debug for EmbeddedIO {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("EmbeddedIO")
			.field("char_display", &self.char_display)
			.field("number_display", &self.number_display)
			.field("screen", &self.screen)
			.field_with("controller", |f| f.write_str(&format!("{:?}", self.controller)))
			.field_with("rng (next value)", |f| {
				let next_value: u8 = self.rng.clone().gen();
				f.write_str(&format!("{} / 0x{:2X}", next_value, next_value))
			})
			.finish()
	}
}

impl IO for EmbeddedIO {
	type Error = !;
	
	fn set_pixel_x(&mut self, value: u8) -> Result<(), Self::Error> {
		self.screen.x = value & 0b11111;
		Ok(())
	}
	
	fn set_pixel_y(&mut self, value: u8) -> Result<(), Self::Error> {
		self.screen.y = value & 0b11111;
		Ok(())
	}
	
	fn draw_pixel(&mut self) -> Result<(), Self::Error> {
		self.screen.set_buffer_pixel(self.screen.x, self.screen.y, true);
		Ok(())
	}
	
	fn clear_pixel(&mut self) -> Result<(), Self::Error> {
		self.screen.set_buffer_pixel(self.screen.x, self.screen.y, false);
		Ok(())
	}
	
	fn load_pixel(&mut self) -> Result<u8, Self::Error> {
		if self.screen.get_buffer_pixel(self.screen.x, self.screen.y) {
			Ok(1)
		} else {
			Ok(0)
		}
	}
	
	fn show_screen_buffer(&mut self) -> Result<(), Self::Error> {
		self.screen.show_buffer();
		Ok(())
	}
	
	fn clear_screen_buffer(&mut self) -> Result<(), Self::Error> {
		self.screen.clear_buffer();
		Ok(())
	}
	
	fn write_char(&mut self, value: u8) -> Result<(), Self::Error> {
		self.char_display.write(Char::new(value));
		Ok(())
	}
	
	fn show_char_buffer(&mut self) -> Result<(), Self::Error> {
		self.char_display.show_buffer();
		Ok(())
	}
	
	fn clear_char_buffer(&mut self) -> Result<(), Self::Error> {
		self.char_display.clear_buffer();
		Ok(())
	}
	
	fn show_number(&mut self, value: u8) -> Result<(), Self::Error> {
		self.number_display.set(value);
		Ok(())
	}
	
	fn clear_number(&mut self) -> Result<(), Self::Error> {
		self.number_display.clear();
		Ok(())
	}
	
	fn set_number_signed(&mut self) -> Result<(), Self::Error> {
		self.number_display.signed = true;
		Ok(())
	}
	
	fn set_number_unsigned(&mut self) -> Result<(), Self::Error> {
		self.number_display.signed = false;
		Ok(())
	}
	
	fn get_rng(&mut self) -> Result<u8, Self::Error> {
		Ok(self.rng.gen())
	}
	
	fn get_controller(&mut self) -> Result<u8, Self::Error> {
		Ok(self.controller.state)
	}
}

#[derive(Default, Clone)]
pub struct Screen {
	pub x: u8,
	pub y: u8,
	pub buffer: [u32; 32],
	pub output: [u32; 32],
}

impl Screen {
	pub fn get_pixel(&self, mut x: u8, mut y: u8) -> bool {
		x &= 0b11111;
		y &= 0b11111;
		
		self.output[y as usize] & (1 << x) != 0
	}
	
	pub fn set_pixel(&mut self, mut x: u8, mut y: u8, val: bool) {
		x &= 0b11111;
		y &= 0b11111;
		
		if val {
			self.output[y as usize] |= 1 << x
		} else {
			self.output[y as usize] &= !(1 << x);
		}
	}
	
	pub fn get_buffer_pixel(&self, mut x: u8, mut y: u8) -> bool {
		x &= 0b11111;
		y &= 0b11111;
		
		self.buffer[y as usize] & (1 << x) != 0
	}
	
	pub fn set_buffer_pixel(&mut self, mut x: u8, mut y: u8, val: bool) {
		x &= 0b11111;
		y &= 0b11111;
		
		if val {
			self.buffer[y as usize] |= 1 << x
		} else {
			self.buffer[y as usize] &= !(1 << x);
		}
	}
	
	pub fn show_buffer(&mut self) {
		self.output = self.buffer;
	}
	
	pub fn clear_buffer(&mut self) {
		self.buffer = [0; 32];
	}
	
	pub fn clear_output(&mut self) {
		self.output = [0; 32];
	}
}

impl Debug for Screen {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Screen")
			.field_with("coords", |f| f.write_str(&format!("{:?}", vec![self.x, self.y])))
			.field_with("buffer", |f| f.write_str(&format!("{:?}", self.buffer)))
			.field_with("output", |f| f.write_str(&format!("{:?}", self.output)))
			.finish()
	}
}

#[derive(Default, Clone)]
pub struct CharDisplay {
	pub buffer: [Char; 10],
	pub output: [Char; 10],
	pub head: usize,
}

impl CharDisplay {
	pub fn write(&mut self, char: Char) {
		self.head = self.head % 10;
		self.buffer[self.head] = char;
		self.head += 1;
	}
	
	pub fn show_buffer(&mut self) {
		self.output = self.buffer;
	}
	
	pub fn clear_buffer(&mut self) {
		self.buffer = [Char::SPACE; 10];
		self.head = 0;
	}
	
	pub fn clear_output(&mut self) {
		self.output = [Char::SPACE; 10];
	}
}

impl Display for CharDisplay {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for chr in self.output.iter() {
			std::fmt::Display::fmt(&chr, f)?;
		}
		Ok(())
	}
}

impl Debug for CharDisplay {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let buffer_str: String = self.buffer.iter().map(ToString::to_string).collect();
		let output_str: String = self.output.iter().map(ToString::to_string).collect();
		f.debug_struct("CharDisplay")
			.field("buffer", &buffer_str)
			.field("output", &output_str)
			.finish()
	}
}

#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub struct NumberDisplay {
	pub value: Option<u8>,
	pub signed: bool,
}

impl NumberDisplay {
	pub fn set(&mut self, value: u8) {
		self.value = Some(value);
	}
	
	pub fn set_unsigned(&mut self, value: u8) {
		self.set(value);
		self.signed = false;
	}
	
	pub fn set_signed(&mut self, value: i8) {
		self.set(value.cast_unsigned());
		self.signed = true;
	}
	
	pub fn clear(&mut self) {
		self.value = None;
	}
}

impl Display for NumberDisplay {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match &self.value {
			None => { Ok(()) }
			Some(value) => {
				if self.signed {
					let signed = value.cast_signed();
					Display::fmt(&signed, f)
				}else{
					Display::fmt(&value, f)
				}
			}
		}
	}
}
impl Debug for NumberDisplay {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match &self.value {
			None => {
				write!(f, "None ({})", if self.signed { "signed" } else { "unsigned" })
			}
			Some(value) => {
				write!(f, "{} ({} 0x{:2X})", &self, if self.signed { "signed" } else { "unsigned" }, value)
			}
		}
	}
}

#[derive(Default, Clone)]
pub struct Controller {
	pub state: u8,
}

impl Controller {
	pub const B_LEFT: u8 = 0x01;
	pub const B_DOWN: u8 = 0x02;
	pub const B_RIGHT: u8 = 0x04;
	pub const B_UP: u8 = 0x08;
	pub const B_B: u8 = 0x10;
	pub const B_A: u8 = 0x20;
	pub const B_SELECT: u8 = 0x40;
	pub const B_START: u8 = 0x80;
	pub const BUTTON_NAMES: [&'static str; 8] = [ "LEFT", "DOWN", "RIGHT", "UP", "B", "A", "SELECT", "START" ];
	
	pub fn get_button(&self, button: u8) -> bool {
		self.state & button != 0
	}
	
	pub fn set_button(&mut self, button: u8) {
		self.state |= button;
	}
	
	pub fn clear_button(&mut self, button: u8) {
		self.state &= !button;
	}
}

impl Debug for Controller {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "0x{:02X} ", self.state)?;
		let mut set = f.debug_set();
		for i in 0..Self::BUTTON_NAMES.len() {
			if self.get_button((1 << i) as u8) {
				set.entry_with(|f| f.write_str(Self::BUTTON_NAMES[i]));
			}
		}
		set.finish()?;
		Ok(())
	}
}