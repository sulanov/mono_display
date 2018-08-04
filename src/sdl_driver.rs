extern crate sdl2;
use super::*;

impl From<String> for Error {
    fn from(e: String) -> Error {
        Error::from_string(format!("SDL error: {}", e))
    }
}

impl From<sdl2::video::WindowBuildError> for Error {
    fn from(e: sdl2::video::WindowBuildError) -> Error {
        Error::from_string(format!("SDL error: {}", e))
    }
}

impl From<sdl2::IntegerOrSdlError> for Error {
    fn from(e: sdl2::IntegerOrSdlError) -> Error {
        Error::from_string(format!("SDL error: {}", e))
    }
}

impl From<sdl2::render::TextureValueError> for Error {
    fn from(e: sdl2::render::TextureValueError) -> Error {
        Error::from_string(format!("SDL texture error: {}", e))
    }
}

impl From<sdl2::render::UpdateTextureError> for Error {
    fn from(e: sdl2::render::UpdateTextureError) -> Error {
        Error::from_string(format!("SDL texture error: {}", e))
    }
}

pub struct SdlDriver {
    size: gfx::Size,
    canvas: sdl2::render::WindowCanvas,
}

const SCALE: u32 = 4;

impl SdlDriver {
    pub fn new(size: gfx::Size) -> Result<SdlDriver> {
        let sdl_context = sdl2::init()?;
        let video = sdl_context.video()?;
        let window = video
            .window(
                "mono_display",
                SCALE * size.width as u32,
                SCALE * size.height as u32,
            )
            .position_centered()
            .build()?;
        let canvas = window.into_canvas().build()?;

        Ok(SdlDriver { size, canvas })
    }

    fn show_frame_may_fail(&mut self, frame: gfx::Frame) -> Result<()> {
        let width = frame.size().width;
        let mut bitmap = Vec::with_capacity(width * frame.size().height * 4 * 4);
        for y in 0..self.size.height {
            for x in 0..self.size.width {
                let v = (((frame.data()[(y / 8) * width + x]) >> (y % 8)) & 1) > 0;
                let pixel = if v {
                    &[0xff, 0xff, 0xff, 0xff]
                } else {
                    &[0xff, 0x00, 0x00, 0xff]
                };
                bitmap.extend_from_slice(pixel);
            }
        }

        let r = sdl2::rect::Rect::new(0, 0, width as u32, frame.size().height as u32);
        let texture_creator = self.canvas.texture_creator();
        let mut texture = texture_creator.create_texture(
            sdl2::pixels::PixelFormatEnum::ARGB8888,
            sdl2::render::TextureAccess::Streaming,
            width as u32,
            self.size.height as u32,
        )?;
        texture.update(r, &bitmap[..], width * 4)?;
        self.canvas.copy(
            &texture,
            r,
            sdl2::rect::Rect::new(
                0,
                0,
                SCALE * width as u32,
                SCALE * frame.size().height as u32,
            ),
        )?;
        self.canvas.present();

        Ok(())
    }
}

impl DisplayDriver for SdlDriver {
    fn size(&self) -> gfx::Size {
        self.size
    }

    fn show_frame(&mut self, frame: gfx::Frame) {
        if let Err(e) = self.show_frame_may_fail(frame) {
            println!("ERROR: Failed to render SDL frame: {:?}", e);
        }
    }

    fn get_frame(&mut self) -> gfx::Frame {
        gfx::Frame::new(self.size)
    }
}
