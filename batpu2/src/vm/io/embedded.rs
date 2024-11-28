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
		self.screen.buffer[self.screen.y as usize] |= 1 << self.screen.x;
	}
	
	fn clear_pixel(&mut self) {
		self.screen.buffer[self.screen.y as usize] &= !(1 << self.screen.x);
	}
	
	fn load_pixel(&mut self) -> u8 {
		if self.screen.buffer[self.screen.y as usize] & (1 << self.screen.x) == 0 {
			0
		} else {
			1
		}
	}
	
	fn show_screen_buffer(&mut self) {
		self.screen.output = self.screen.buffer;
	}
	
	fn clear_screen_buffer(&mut self) {
		self.screen.buffer = [0; 32];
	}
	
	fn write_char(&mut self, value: u8) {
		self.char_display.buffer.rotate_right(1);
		self.char_display.buffer[0] = Char::new(value);
	}
	
	fn show_char_buffer(&mut self) {
		self.char_display.output = self.char_display.buffer;
	}
	
	fn clear_char_buffer(&mut self) {
		self.char_display.buffer = [Char::SPACE; 10];
	}
	
	fn show_number(&mut self, value: u8) {
		self.number_display.value = Some(value);
	}
	
	fn clear_number(&mut self) {
		self.number_display.value = None;
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

#[derive(Debug, Default, Clone)]
pub struct CharDisplay {
	pub buffer: [Char; 10],
	pub output: [Char; 10],
}

#[derive(Debug, Default, Clone)]
pub struct NumberDisplay {
	pub value: Option<u8>,
	pub signed: bool,
}

#[derive(Debug, Default, Clone)]
pub struct Controller {
	pub state: u8,
}
