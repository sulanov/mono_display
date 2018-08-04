extern crate mono_display;

use std::thread;
use std::time;

use mono_display::gfx;
use mono_display::DisplayDriver;

#[cfg(feature = "sdl2")]
use mono_display::sdl_driver;
#[cfg(feature = "sdl2")]
fn create_driver() -> mono_display::Result<sdl_driver::SdlDriver> {
    sdl_driver::SdlDriver::new(gfx::Size {
        width: 128,
        height: 32,
    })
}

#[cfg(not(feature = "sdl2"))]
use mono_display::ssd1306;
#[cfg(not(feature = "sdl2"))]
fn create_driver() -> mono_display::Result<ssd1306::Ssd1306> {
    ssd1306::Ssd1306::new(ssd1306::Ssd1306Type::S128x32, false)
}

fn show_demo() -> mono_display::Result<()> {
    let mut driver = create_driver()?;
    let font16 = gfx::Font::load("font16.ttf", 16)?;
    let font8 = gfx::Font::load("font8.ttf", 8)?;
    for i in 0..500 {
        let mut canvas = gfx::Canvas::new(driver.get_frame());

        let text_pos = gfx::Vector::xy(i % 100, 10 + i % 30);
        canvas.draw_text(text_pos, &font16, "Hello", gfx::Color::Light);

        let text_pos_2 = text_pos.add_xy(0, 8);
        let text_rect_2 = canvas.get_text_rect(text_pos_2, &font8, "Hello");
        canvas.draw_rect(text_rect_2, gfx::Color::Light);
        canvas.draw_text(text_pos_2, &font8, "Hello", gfx::Color::Dark);

        driver.show_frame(canvas.take_frame());

        thread::sleep(time::Duration::from_millis(30));
    }

    Ok(())
}

fn main() {
    match show_demo() {
        Err(e) => {
            println!("{}", e.msg);
            std::process::exit(-1);
        }
        Ok(_) => (),
    }
}
