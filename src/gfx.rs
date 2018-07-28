#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Size {
	pub width: usize,
	pub height: usize,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Point {
	pub x: i16,
	pub y: i16,
}

#[derive(Clone)]
pub struct Frame {
	size: Size,
	data: Vec<u8>,
}

impl Frame {
	pub fn new(size: Size) -> Frame {
		let buf_size = size.width * size.height / 8;
		Frame { size, data: vec![0; buf_size] }
	}

	pub fn size(&self) -> Size {
		self.size
	}

	pub fn mut_data(&mut self) -> &mut [u8] {
		&mut self.data[..]
	}

	pub fn data(&self) -> &[u8] {
		&self.data[..]
	}
}

pub struct Canvas {
	frame: Frame,
}

impl Canvas {
	pub fn new(frame: Frame) -> Canvas {
		Canvas { frame }
	}

	pub fn set_pixel(&mut self, a: Point, color: bool) {
		let pos = (a.y as usize / 8) * self.frame.size().width + a.x as usize;
		let byte = &mut self.frame.mut_data()[pos];
		if color {
			*byte |= 1u8 << (a.y % 8);
		} else {
			*byte &= !(1u8 << (a.y % 8));
		}
	}
	// pub fn draw_line(&mut self, a: Point, b: Point, color: bool) {}

	pub fn take_frame(self) -> Frame {
		self.frame
	}
}