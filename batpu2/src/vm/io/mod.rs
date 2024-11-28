
#[cfg(feature = "embedded_io")]
pub mod embedded;
#[cfg(feature = "embedded_io")]
pub use embedded::EmbeddedIO;

pub trait RawIO {
	fn read_addr(&mut self, addr: u8) -> u8;
	fn write_addr(&mut self, addr: u8, value: u8) -> ();
}

pub trait IO {                            // Address
	fn set_pixel_x(&mut self, value: u8); // 240
	fn set_pixel_y(&mut self, value: u8); // 241
	fn draw_pixel(&mut self);             // 242
	fn clear_pixel(&mut self);            // 243
	fn load_pixel(&mut self) -> u8;       // 244
	fn show_screen_buffer(&mut self);     // 245
	fn clear_screen_buffer(&mut self);    // 246
	fn write_char(&mut self, value: u8);  // 247
	fn show_char_buffer(&mut self);       // 248
	fn clear_char_buffer(&mut self);      // 249
	fn show_number(&mut self, value: u8); // 250
	fn clear_number(&mut self);           // 251
	fn set_number_signed(&mut self);      // 252
	fn set_number_unsigned(&mut self);    // 253
	fn get_rng(&mut self) -> u8;          // 254
	fn get_controller(&mut self) -> u8;   // 255
}

impl<T: IO> RawIO for T {
	fn read_addr(&mut self, addr: u8) -> u8 {
		match addr {
			244 => self.load_pixel(),
			254 => self.get_rng(),
			255 => self.get_controller(),
			_ => 0,
		}
	}
	
	fn write_addr(&mut self, addr: u8, value: u8) -> () {
		match addr {
			240 => self.set_pixel_x(value),
			241 => self.set_pixel_y(value),
			242 => self.draw_pixel(),
			243 => self.clear_pixel(),
			245 => self.show_screen_buffer(),
			246 => self.clear_screen_buffer(),
			247 => self.write_char(value),
			248 => self.show_char_buffer(),
			249 => self.clear_char_buffer(),
			250 => self.show_number(value),
			251 => self.clear_number(),
			252 => self.set_number_signed(),
			253 => self.set_number_unsigned(),
			_ => {},
		}
	}
}
