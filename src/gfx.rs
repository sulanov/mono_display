extern crate font_rs;

use std;
use std::cmp;
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::ops::Add;

use super::Error;
use super::Result;

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::from_string(format!("IO error: {}", e))
    }
}

impl From<font_rs::font::FontError> for Error {
    fn from(e: font_rs::font::FontError) -> Error {
        Error::from_string(format!("Fonts error: {:?}", e))
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

impl Size {
    pub fn wh(width: usize, height: usize) -> Size {
        Size { width, height }
    }

    pub fn zero() -> Size {
        Size::wh(0, 0)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Vector {
    pub x: i16,
    pub y: i16,
}

impl Vector {
    pub fn zero() -> Vector {
        Vector::xy(0, 0)
    }

    pub fn xy(x: i16, y: i16) -> Vector {
        Vector { x, y }
    }

    pub fn add_xy(&self, dx: i16, dy: i16) -> Vector {
        Vector::xy(self.x + dx, self.y + dy)
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, b: Vector) -> Vector {
        Vector::xy(self.x + b.x, self.y + b.y)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Rect {
    pub pos: Vector,
    pub size: Size,
}

impl Rect {
    pub fn zero() -> Rect {
        Rect::ps(Vector::zero(), Size::zero())
    }

    pub fn xywh(x: i16, y: i16, width: usize, height: usize) -> Rect {
        Rect::ps(Vector::xy(x, y), Size::wh(width, height))
    }

    pub fn ltrb(left: i16, top: i16, right: i16, bottom: i16) -> Rect {
        Rect::ps(
            Vector::xy(left, top),
            Size::wh((right - left) as usize, (bottom - top) as usize),
        )
    }

    pub fn ps(pos: Vector, size: Size) -> Rect {
        Rect { pos, size }
    }

    pub fn left(&self) -> i16 {
        self.pos.x
    }

    pub fn right(&self) -> i16 {
        self.pos.x + self.size.width as i16
    }

    pub fn top(&self) -> i16 {
        self.pos.y
    }

    pub fn bottom(&self) -> i16 {
        self.pos.y + self.size.height as i16
    }

    pub fn translate(&self, d: Vector) -> Rect {
        Rect::ps(self.pos + d, self.size)
    }

    pub fn intersect(&self, b: Rect) -> Rect {
        let left = cmp::max(self.left(), b.left());
        let right = cmp::min(self.right(), b.right());
        let top = cmp::max(self.top(), b.top());
        let bottom = cmp::min(self.bottom(), b.bottom());
        if left >= right || top >= bottom {
            Rect::zero()
        } else {
            Rect::ltrb(left, top, right, bottom)
        }
    }
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

struct Glyph {
    rect: Rect,
    data: Vec<u8>,
}

pub struct Font {
    glyphs: BTreeMap<u32, Glyph>,
}

fn render_glyph(font: &font_rs::font::Font, codepoint: u32, size: usize) -> Option<Glyph> {
    let id = font.lookup_glyph_id(codepoint)?;
    let glyph = font.render_glyph(id, size as u32)?;
    let width = glyph.width;
    let height = glyph.height;
    let rows = (height + 7) / 8;
    let mut data = Vec::with_capacity(width * rows);
    for row in 0..rows {
        let y = row * 8;
        for x in 0..width {
            let mut v = 0u8;
            for b in 0..cmp::min(height - y, 8) {
                if glyph.data[x + (y + b) * width] > 128 {
                    v |= 1 << b;
                }
            }
            data.push(v);
        }
    }
    Some(Glyph {
        rect: Rect::xywh(glyph.left as i16, glyph.top as i16, width, height),
        data,
    })
}

impl Font {
    pub fn load(filename: &str, size: usize) -> Result<Font> {
        let mut file = fs::File::open(filename)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let font = font_rs::font::parse(&data)?;
        let mut glyphs = BTreeMap::new();
        for i in 16..2000 {
            if let Some(g) = render_glyph(&font, i, size) {
                glyphs.insert(i, g);
            }
        }

        Ok(Font { glyphs })
    }

    fn get_glyph(&self, codepoint: u32) -> Option<&Glyph> {
        self.glyphs.get(&codepoint)
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

fn iter_text_glyphs<F>(mut pos: Vector, font: &Font, text: &str, mut func: F)
where
    F: FnMut(Vector, &Glyph),
{
    for c in text.chars() {
        if let Some(glyph) = font.get_glyph(c as u32) {
            func(pos, glyph);
            pos.x += glyph.rect.size.width as i16 + 1;
        }
    }
}

impl Canvas {
    pub fn new(frame: Frame) -> Canvas {
        Canvas { frame }
    }

    pub fn set_pixel(&mut self, point: Vector, color: Color) {
        self.draw_rect(Rect::ps(point, Size::wh(1, 1)), color)
    }

    pub fn draw_rect(&mut self, mut rect: Rect, color: Color) {
        rect = rect.intersect(Rect::ps(Vector::zero(), self.frame.size()));

        let top = rect.top();
        let bottom = rect.bottom() - 1;
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

            for x in rect.left()..rect.right() {
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

    fn draw_glyph(&mut self, pos: Vector, glyph: &Glyph, color: Color) {
        let left = pos.x + glyph.rect.left();
        let top = pos.y + glyph.rect.top();
        let frame_width = self.frame.size().width as i16;
        let x_start = cmp::max(0, -left);
        let x_end = cmp::min(glyph.rect.size.width as i16, frame_width - left);

        let mut y = 0i16;
        while y < glyph.rect.size.height as i16 {
            let out_y = top + y;
            if out_y < 0 {
                y += 1;
                continue;
            }
            if out_y >= self.frame.size().height as i16 {
                break;
            }

            let rows_to_copy = cmp::min(
                cmp::min(8 - (y % 8), 8 - (out_y % 8)),
                glyph.rect.size.height as i16 - y,
            );
            let mask = 0xffu8 << (8 - rows_to_copy) >> (8 - rows_to_copy);
            let in_shift = y % 8;
            let out_shift = out_y % 8;

            let in_row = &glyph.data[(y as usize / 8) * glyph.rect.size.width..];
            for x in x_start..x_end {
                let out = &mut self.frame.mut_data()
                    [(out_y as usize / 8) * frame_width as usize + (x + left) as usize];
                let inp = ((in_row[x as usize] >> in_shift) & mask) << out_shift;
                if color == Color::Light {
                    *out |= inp;
                } else {
                    *out &= !inp;
                }
            }

            y += rows_to_copy;
        }
    }

    pub fn draw_text(&mut self, pos: Vector, font: &Font, text: &str, color: Color) {
        iter_text_glyphs(pos, font, text, |pos, glyph| {
            self.draw_glyph(pos, glyph, color)
        });
    }

    pub fn get_text_rect(&self, pos: Vector, font: &Font, text: &str) -> Rect {
        let mut rect = Rect::ps(pos, Size::zero());
        iter_text_glyphs(pos, font, text, |pos, glyph| {
            let g_rect = glyph.rect.translate(pos);

            let dl = rect.left() - g_rect.left();
            if dl > 0 {
                rect.pos.x -= dl;
                rect.size.width += dl as usize;
            }

            let dr = g_rect.right() - rect.right();
            if dr > 0 {
                rect.size.width += dr as usize;
            }

            let dt = rect.top() - g_rect.top();
            if dt > 0 {
                rect.pos.y -= dt;
                rect.size.height += dt as usize;
            }

            let db = g_rect.bottom() - rect.bottom();
            if db > 0 {
                rect.size.height += db as usize;
            }
        });

        rect
    }

    pub fn take_frame(self) -> Frame {
        self.frame
    }
}
