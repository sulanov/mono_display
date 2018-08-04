mod driver;
pub mod gfx;
pub mod ssd1306;

#[cfg(feature = "sdl2")]
pub mod sdl_driver;

use std::result;

#[derive(Debug)]
pub struct Error {
    pub msg: String,
}

impl Error {
    pub fn new(msg: &str) -> Error {
        Error {
            msg: String::from(msg),
        }
    }
    pub fn from_string(msg: String) -> Error {
        Error { msg: msg }
    }
}

pub type Result<T> = result::Result<T, Error>;
pub use self::driver::DisplayDriver;
