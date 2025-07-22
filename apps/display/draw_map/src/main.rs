/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
extern crate axstd as std;

mod display;

use self::display::Display;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::primitives::{Circle, PrimitiveStyle};
use embedded_graphics::{image::Image, prelude::*};
use tinybmp::Bmp;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() -> ! {
    // Include the BMP file data.
    let bmp_data = include_bytes!("../pictures/map.bmp");

    // Parse the BMP file.
    // Note that it is necessary to explicitly specify the color type which the colors in the BMP
    // file will be converted into.
    let bmp = Bmp::<Rgb888>::from_slice(bmp_data).unwrap();

    let mut display = Display::new();
    for i in 1..=70 {
        // Draw the image with the top left corner at (0, 0) by wrapping it in
        // an embedded-graphics `Image`.
        let _ = Image::new(&bmp, Point::new(0, 0)).draw(&mut display);

        let _ = Circle::new(Point::new(i * 10, i * 10), 20)
            .into_styled(PrimitiveStyle::with_fill(Rgb888::RED))
            .draw(&mut display);
        let _ = Circle::new(Point::new(1024 - i * 10, i * 10), 20)
            .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
            .draw(&mut display);
        display.flush();

        // as a sleep()
        for _ in 1..=100 {
            display.flush();
        }
    }

    loop {
        core::hint::spin_loop();
    }
}
