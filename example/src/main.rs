extern crate mono_display;

use mono_display::gfx;
use mono_display::DisplayDriver;
use std::env;
use std::thread;
use std::time;

#[cfg(feature = "sdl2")]
fn create_driver() -> mono_display::Result<mono_display::sdl_driver::SdlDriver> {
    mono_display::sdl_driver::SdlDriver::new(gfx::Size::wh(128, 32))
}

#[cfg(not(feature = "sdl2"))]
fn create_driver() -> mono_display::Result<mono_display::ssd1306::Ssd1306> {
    mono_display::ssd1306::Ssd1306::new(mono_display::ssd1306::Ssd1306Type::S128x32, false)
}

fn show_demo() -> mono_display::Result<()> {
    let mut driver = create_driver()?;
    let args: Vec<String> = env::args().collect();
    let font16 = gfx::Font::load_bdf(args[1].as_str(), 18)?;
    let font8 = gfx::Font::load_bdf(args[2].as_str(), 10)?;
    for i in 0..500 {
        let mut canvas = gfx::Canvas::new(driver.get_frame());

        let text_pos = gfx::Vector::xy(i % 100, 10 + i % 30);
        canvas.draw_text(text_pos, &font16, "ABC abc 123 base", gfx::Color::Light);

        let text_2 = "ABC abc 123";
        let text_pos_2 = text_pos.add_xy(0, 10);
        let text_rect_2 = canvas.get_text_rect(text_pos_2, &font8, text_2);
        canvas.draw_rect(text_rect_2, gfx::Color::Light);
        canvas.draw_text(text_pos_2, &font8, text_2, gfx::Color::Dark);

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
