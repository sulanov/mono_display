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
        Frame {
            size,
            data: vec![0; buf_size],
        }
    }

    pub fn clear(&mut self) {
        for byte in self.data.iter_mut() {
            *byte = 0;
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn num_rows(&self) -> usize {
        self.size.height / 8
    }

    pub fn mut_data(&mut self) -> &mut [u8] {
        &mut self.data[..]
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum Color {
    Light,
    Dark,
}

pub struct Canvas {
    frame: Frame,
}

impl Canvas {
    pub fn new(frame: Frame) -> Canvas {
        Canvas { frame }
    }

    pub fn set_pixel(&mut self, point: Point, color: Color) {
        self.draw_rect(
            point,
            Size {
                width: 1,
                height: 1,
            },
            color,
        )
    }

    pub fn draw_rect(&mut self, top_left: Point, size: Size, color: Color) {
        let top = top_left.y;
        let bottom = top + size.height as i16 - 1;
        for row in (top / 8)..(bottom / 8 + 1) {
            let mut mask = 0xff;
            if row == top / 8 {
                let bits = top % 8;
                mask >>= bits;
                mask <<= bits;
            }

            if row == bottom / 8 {
                let bits = 7 - bottom % 8;
                mask <<= bits;
                mask >>= bits;
            }

            for x in top_left.x..(top_left.x + size.width as i16) {
                let pos = row as usize * self.frame.size().width + x as usize;
                let byte = &mut self.frame.mut_data()[pos];
                if color == Color::Light {
                    *byte |= mask;
                } else {
                    *byte &= !mask;
                }
            }
        }
    }

    // pub fn draw_line(&mut self, a: Point, b: Point, color: Color) {}

    pub fn take_frame(self) -> Frame {
        self.frame
    }
}
