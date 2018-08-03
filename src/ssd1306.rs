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
    S128x64,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum AddressMode {
    Page,
    Horizontal,
    Vertical,
}

pub struct Ssd1306 {
    type_: Ssd1306Type,
    bus: i2c::I2c,
    address_mode: AddressMode,
    cur_frame: Option<gfx::Frame>,
    old_frame: Option<gfx::Frame>,
}

const COMMAND_DISPLAY_OFF: u8 = 0xAE;
const COMMAND_DISPLAY_ON: u8 = 0xAF;

// Helper for show_frame_diff() used to find spans that a different
// between |a| and |b|. Two spans that are less than 5 pixels apart are
// considered part of the same span to account for the costs of sending
// page address.
fn find_span_end(a: &[u8], b: &[u8]) -> usize {
    let mut last_diff = 0;
    for i in 1..a.len() {
        if a[i] != b[i] {
            last_diff = i;
        }
        if i - last_diff > 4 {
            return last_diff + 1;
        }
    }
    last_diff + 1
}

impl Ssd1306 {
    pub fn new(type_: Ssd1306Type, flip: bool) -> Result<Ssd1306> {
        let mut bus = i2c::I2c::new()?;
        bus.set_slave_address(0x3c)?;
        let mut result = Ssd1306 {
            type_,
            bus,
            address_mode: AddressMode::Page,
            cur_frame: None,
            old_frame: None,
        };
        result.initialize(flip)?;

        Ok(result)
    }

    fn initialize(&mut self, flip: bool) -> Result<()> {
        self.send_command(&[COMMAND_DISPLAY_OFF])?;

        // Set multiplex ratio.
        let mux_ratio = if self.type_ == Ssd1306Type::S128x32 {
            0x1F
        } else {
            0x3F
        };
        self.send_command(&[0xA8, mux_ratio])?;

        // Charge pump settings.
        self.send_command(&[0x8D, 0x14])?;

        // Clock div.
        self.send_command(&[0xB3, 0x80])?;

        // Clock offset.
        self.send_command(&[0xD3, 0x00])?;

        // Set start line to 0.
        self.send_command(&[0x40])?;

        // Set segment re-map.
        self.send_command(&[if flip { 0xA1 } else { 0xA0 }])?;
        // Set output scan direction.
        self.send_command(&[if flip { 0xC8 } else { 0xC0 }])?;

        // Set COM pin config.
        let pin_config = if self.type_ == Ssd1306Type::S128x32 {
            0x02
        } else {
            0x12
        };
        self.send_command(&[0xDA, pin_config])?;

        // Pre-charge period.
        self.send_command(&[0xD9, 0xF1])?;

        // Set VCOMH Deselect Level.
        self.send_command(&[0xDB, 0x40])?;

        // Entire Display ON.
        self.send_command(&[0xA4])?;

        // Set Normal Display.
        self.send_command(&[0xA6])?;

        // Set contrast level.
        self.send_command(&[0x81, 0x8F])?;

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

    fn set_address_mode(&mut self, mode: AddressMode) -> Result<()> {
        if self.address_mode != mode {
            self.address_mode = mode;
            let mode_code = match mode {
                AddressMode::Page => 0b10,
                AddressMode::Horizontal => 0b00,
                AddressMode::Vertical => 0b01,
            };
            self.send_command(&[0x20, mode_code])?;
        }

        Ok(())
    }

    fn show_frame_whole(&mut self, frame: &gfx::Frame) -> Result<()> {
        self.set_address_mode(AddressMode::Horizontal)?;

        // Set column range.
        let max_col = frame.size().width as u8 - 1;
        self.send_command(&[0x21, 0, max_col])?;

        // Set page range.
        let max_page = (frame.size().height / 8) as u8 - 1;
        self.send_command(&[0x22, 0, max_page])?;

        // Send the frame.
        self.send_data(&frame.data()[..])?;

        Ok(())
    }

    fn show_frame_diff(&mut self, frame: &gfx::Frame, old_frame: &gfx::Frame) -> Result<()> {
        self.set_address_mode(AddressMode::Page)?;
        let width = self.size().width;

        for page in 0..frame.num_rows() as u8 {
            let data_pos = page as usize * width;
            let old = &old_frame.data()[data_pos..(data_pos + width)];
            let new = &frame.data()[data_pos..(data_pos + width)];
            let mut page_set = false;
            let mut pos = 0;
            while pos < width {
                if old[pos] == new[pos] {
                    pos += 1;
                    continue;
                }

                let mut end = pos + find_span_end(&old[pos..], &new[pos..]);

                if !page_set {
                    self.send_command(&[0xB0 | page])?;
                    page_set = true;
                }

                // Set high and low addresses.
                self.send_command(&[0x00 | (pos as u8 & 0x0f)])?;
                self.send_command(&[0x10 | ((pos as u8 & 0xf0) >> 4)])?;

                self.send_data(&new[pos..end])?;

                pos = end;
            }
        }

        Ok(())
    }

    fn show_frame_internal(&mut self, frame: gfx::Frame) -> Result<()> {
        assert!(frame.size() == self.size());

        match self.cur_frame.take() {
            None => {
                self.show_frame_whole(&frame)?;
            }
            Some(old) => {
                self.show_frame_diff(&frame, &old)?;
                self.old_frame = Some(old);
            }
        };
        self.cur_frame = Some(frame);

        Ok(())
    }
}

impl super::DisplayDriver for Ssd1306 {
    fn size(&self) -> gfx::Size {
        match self.type_ {
            Ssd1306Type::S128x32 => gfx::Size {
                width: 128,
                height: 32,
            },
            Ssd1306Type::S128x64 => gfx::Size {
                width: 128,
                height: 64,
            },
        }
    }

    fn show_frame(&mut self, frame: gfx::Frame) {
        if let Err(e) = self.show_frame_internal(frame) {
            println!("Ssd1306: Failed to push a frame: {}", e.msg);
        }
    }

    fn get_frame(&mut self) -> gfx::Frame {
        match self.old_frame.take() {
            Some(mut f) => {
                f.clear();
                f
            }
            None => gfx::Frame::new(self.size()),
        }
    }
}
