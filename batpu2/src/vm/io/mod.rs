use std::error::Error as StdError;

#[cfg(feature = "embedded_io")]
pub mod embedded;
pub mod char;

pub trait RawIO {
	type Error: StdError + 'static;
	
	fn read_addr(&mut self, addr: u8) -> Result<u8, Self::Error>;
	fn write_addr(&mut self, addr: u8, value: u8) -> Result<(), Self::Error>;
}

pub trait IO {
	type Error: StdError + 'static;
	                                                                 // Address
	fn set_pixel_x(&mut self, value: u8) -> Result<(), Self::Error>; // 240
	fn set_pixel_y(&mut self, value: u8) -> Result<(), Self::Error>; // 241
	fn draw_pixel(&mut self)             -> Result<(), Self::Error>; // 242
	fn clear_pixel(&mut self)            -> Result<(), Self::Error>; // 243
	fn load_pixel(&mut self)             -> Result<u8, Self::Error>; // 244
	fn show_screen_buffer(&mut self)     -> Result<(), Self::Error>; // 245
	fn clear_screen_buffer(&mut self)    -> Result<(), Self::Error>; // 246
	fn write_char(&mut self, value: u8)  -> Result<(), Self::Error>; // 247
	fn show_char_buffer(&mut self)       -> Result<(), Self::Error>; // 248
	fn clear_char_buffer(&mut self)      -> Result<(), Self::Error>; // 249
	fn show_number(&mut self, value: u8) -> Result<(), Self::Error>; // 250
	fn clear_number(&mut self)           -> Result<(), Self::Error>; // 251
	fn set_number_signed(&mut self)      -> Result<(), Self::Error>; // 252
	fn set_number_unsigned(&mut self)    -> Result<(), Self::Error>; // 253
	fn get_rng(&mut self)                -> Result<u8, Self::Error>; // 254
	fn get_controller(&mut self)         -> Result<u8, Self::Error>; // 255
}

impl<T: IO> RawIO for T {
	type Error = <T as IO>::Error;
	
	fn read_addr(&mut self, addr: u8) -> Result<u8, Self::Error> {
		match addr {
			244 => self.load_pixel(),
			254 => self.get_rng(),
			255 => self.get_controller(),
			_ => Ok(0),
		}
	}
	
	fn write_addr(&mut self, addr: u8, value: u8) -> Result<(), Self::Error> {
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
			_ => Ok(()),
		}
	}
}
