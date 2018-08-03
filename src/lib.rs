mod driver;
pub mod gfx;
pub mod ssd1306;

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
