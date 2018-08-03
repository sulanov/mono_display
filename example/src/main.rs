extern crate mono_display;

use std::thread;
use std::time;

use mono_display::gfx;
use mono_display::ssd1306;
use mono_display::DisplayDriver;

fn show_demo() -> mono_display::Result<()> {
    let mut driver = ssd1306::Ssd1306::new(ssd1306::Ssd1306Type::S128x32, false)?;
    for i in 0..500 {
        let mut canvas = gfx::Canvas::new(driver.get_frame());

        let v = if i % 2 > 0 {
            gfx::Color::Light
        } else {
            gfx::Color::Dark
        };
        println!("{}", i);
        canvas.draw_rect(
            gfx::Point {
                x: i % 120,
                y: i % 20,
            },
            gfx::Size {
                width: 8,
                height: 12,
            },
            v,
        );

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

        thread::sleep(time::Duration::from_millis(16));
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
