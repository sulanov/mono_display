extern crate rppal;

use self::rppal::i2c;
use super::*;


impl From<i2c::Error> for Error {
    fn from(e: i2c::Error) -> Error {
        Error::from_string(format!("i2c error: {}", e))
    }
}

#[derive(Eq, PartialEq)]
pub enum Ssd1306Type {
	S128x32,
	S128x64
}

pub struct Ssd1306 {
	type_: Ssd1306Type,
	bus: i2c::I2c,
}

const COMMAND_DISPLAY_OFF: u8 = 0xAE;
const COMMAND_DISPLAY_ON: u8 = 0xAF;


impl Ssd1306 {
	pub fn new(type_: Ssd1306Type, flip: bool) -> Result<Ssd1306> {
		let mut bus = i2c::I2c::new()?;
		bus.set_slave_address(0x3c)?;
		let mut result = Ssd1306 {
			type_,
			bus
		};
		result.initialize(flip)?;

		Ok(result)
	}

	fn initialize(&mut self, flip: bool) -> Result<()> {
		self.send_command(&[COMMAND_DISPLAY_OFF, ])?;

		// Set multiplex ratio.
		let mux_ratio = if self.type_ == Ssd1306Type::S128x32 { 0x1F } else { 0x3F };
		self.send_command(&[0xA8, mux_ratio, ])?;

		// Charge pump settings.
		self.send_command(&[0x8D, 0x14, ])?;
		
		// Memory mode.
		self.send_command(&[0x20, 0x00, ])?;

		// Clock div.
		self.send_command(&[0xB3, 0x80, ])?;

		// Clock offset.
		self.send_command(&[0xD3, 0x00, ])?;

		// Set start line to 0.
		self.send_command(&[0x40, ])?;

		// Set segment re-map.
		self.send_command(&[if flip { 0xA1 } else { 0xA0 }, ])?;
		// Set output scan direction .
		self.send_command(&[if flip { 0xC8 } else { 0xC0 }, ])?;

		// Set COM pin config.
		let pin_config = if self.type_ == Ssd1306Type::S128x32 { 0x02 } else { 0x12 };
		self.send_command(&[0xDA, pin_config, ])?;

		// Pre-charge period.
		self.send_command(&[0xD9, 0xF1, ])?;

		// Set VCOMH Deselect Level.
		self.send_command(&[0xDB, 0x40, ])?;

		// Entire Display ON.
		self.send_command(&[0xA4, ])?;

		// Set Normal Display.
		self.send_command(&[0xA6, ])?;

		// Set column address.
		self.send_command(&[0x21, 0, 127, ])?;

		// Set page address.
		self.send_command(&[0x22, 0, 7, ])?;

		// Set contrast level.
		self.send_command(&[0x81, 0x8F, ])?;

		self.send_command(&[COMMAND_DISPLAY_ON])?;              

		Ok(())
	}

	fn i2c_send(&mut self, mode: u8, content: &[u8]) -> Result<()> {
		let mut data = Vec::with_capacity(content.len() + 1);
		data.push(mode);
		data.extend_from_slice(content);
		assert!(self.bus.write(&data[..])? == data.len());
		Ok(())
	}

	fn send_command(&mut self, cmd: &[u8]) -> Result<()> {
		self.i2c_send(0x00, cmd)
	}

	fn send_data(&mut self, data: &[u8]) -> Result<()> {
		self.i2c_send(0x40, data)
	}

	fn show_frame_may_fail(&mut self, frame: &gfx::Frame) -> Result<()> {
		assert!(frame.size() == self.size());

		// TODO: update this to send only updated parts of the frame.

		// Set lower column start address.
	    self.send_command(&[0x00 | 0x0, ])?;

		// Set higher column start address.
	    self.send_command(&[0x10 | 0x0, ])?;

	    // Set start line.
	    self.send_command(&[0x40 | 0x0, ])?;

	    // Send the frame.
	    for chunk in frame.data().chunks(16) {
		    self.send_data(&chunk[..])?;	    	
	    }

	    Ok(())
	}
}

impl super::DisplayDriver for Ssd1306 {
	fn size(&self) -> gfx::Size {
		match self.type_ {
			Ssd1306Type::S128x32 => gfx::Size { width: 128, height: 32 },
			Ssd1306Type::S128x64 => gfx::Size { width: 128, height: 64 },
		} 
	}

	fn show_frame(&mut self, frame: &gfx::Frame) {
		if let Err(e) = self.show_frame_may_fail(frame) {
			println!("Ssd1306: Failed to push a frame: {}", e.msg);
		}
	}
}