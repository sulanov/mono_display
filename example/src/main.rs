extern crate mono_display;

use mono_display::gfx;
use mono_display::ssd1306;
use mono_display::DisplayDriver;

fn show_demo() -> mono_display::Result<()> {
    let mut driver = ssd1306::Ssd1306::new(ssd1306::Ssd1306Type::S128x32, false)?;
    let mut canvas = gfx::Canvas::new(gfx::Frame::new(driver.size()));
    canvas.set_pixel(gfx::Point { x: 1, y: 1 }, true);
    canvas.set_pixel(gfx::Point { x: 2, y: 2 }, true);
    canvas.set_pixel(gfx::Point { x: 3, y: 3 }, true);
    let frame = canvas.take_frame();
    driver.show_frame(&frame);

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
