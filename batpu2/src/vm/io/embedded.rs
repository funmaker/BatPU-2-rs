use std::any::type_name;
use std::fmt::{Debug, Display, Formatter};
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};

use crate::{IO, Char};

#[derive(Debug, Clone)]
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

impl IO for EmbeddedIO {
	fn set_pixel_x(&mut self, value: u8) {
		self.screen.x = value & 0b11111;
	}
	
	fn set_pixel_y(&mut self, value: u8) {
		self.screen.y = value & 0b11111;
	}
	
	fn draw_pixel(&mut self) {
		self.screen.set_buffer_pixel(self.screen.x, self.screen.y, true);
	}
	
	fn clear_pixel(&mut self) {
		self.screen.set_buffer_pixel(self.screen.x, self.screen.y, false);
	}
	
	fn load_pixel(&mut self) -> u8 {
		if self.screen.get_buffer_pixel(self.screen.x, self.screen.y) {
			1
		} else {
			0
		}
	}
	
	fn show_screen_buffer(&mut self) {
		self.screen.show_buffer();
	}
	
	fn clear_screen_buffer(&mut self) {
		self.screen.clear_buffer();
	}
	
	fn write_char(&mut self, value: u8) {
		self.char_display.write(Char::new(value));
	}
	
	fn show_char_buffer(&mut self) {
		self.char_display.show_buffer();
	}
	
	fn clear_char_buffer(&mut self) {
		self.char_display.clear_buffer();
	}
	
	fn show_number(&mut self, value: u8) {
		self.number_display.set(value);
	}
	
	fn clear_number(&mut self) {
		self.number_display.clear();
	}
	
	fn set_number_signed(&mut self) {
		self.number_display.signed = true;
	}
	
	fn set_number_unsigned(&mut self) {
		self.number_display.signed = false;
	}
	
	fn get_rng(&mut self) -> u8 {
		self.rng.gen()
	}
	
	fn get_controller(&mut self) -> u8 {
		self.controller.state
	}
}

#[derive(Debug, Default, Clone)]
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

#[derive(Default, Clone)]
pub struct CharDisplay {
	pub buffer: [Char; 10],
	pub output: [Char; 10],
}

impl CharDisplay {
	pub fn write(&mut self, char: Char) {
		self.buffer.rotate_left(1);
		self.buffer[self.buffer.len()-1] = char;
	}
	
	pub fn show_buffer(&mut self) {
		self.output = self.buffer;
	}
	
	pub fn clear_buffer(&mut self) {
		self.buffer = [Char::SPACE; 10];
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
		let output_str: String = self.buffer.iter().map(ToString::to_string).collect();
		f.debug_struct("CharDisplay")
			.field("buffer", &buffer_str)
			.field("output", &output_str)
			.finish()
	}
}

#[derive(Debug, Default, Clone)]
pub struct NumberDisplay {
	pub value: Option<u8>,
	pub signed: bool,
}

pub enum OptSign {
	Unsigned(u8),
	Signed(i8),
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

#[derive(Debug, Default, Clone)]
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
	
	pub fn get_button(&self, button: u8) -> bool {
		self.state & button != 0
	}
	
	pub fn set_button(&mut self, button: u8) {
		self.state |= button;
	}
}
