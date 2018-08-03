extern crate mono_display;

use mono_display::gfx;
use mono_display::ssd1306;
use mono_display::DisplayDriver;

fn show_demo() -> mono_display::Result<()> {
    let mut driver = ssd1306::Ssd1306::new(ssd1306::Ssd1306Type::S128x32, false)?;
    for i in 0..500 {
        let mut canvas = gfx::Canvas::new(driver.get_frame());
        let v = i % 2 > 0;
        println!("{}", i);
        canvas.set_pixel(
            gfx::Point {
                x: i % 128,
                y: i % 28 + 1,
            },
            v,
        );
        canvas.set_pixel(
            gfx::Point {
                x: i % 123 + 2,
                y: i % 28 + 2,
            },
            v,
        );
        canvas.set_pixel(
            gfx::Point {
                x: i % 120 + 3,
                y: i % 28 + 3,
            },
            v,
        );
        driver.show_frame(canvas.take_frame());
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
