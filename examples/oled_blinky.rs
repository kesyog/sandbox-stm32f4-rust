//! Stolen from https://github.com/stm32-rs/stm32f4xx-hal/blob/master/examples/delay-blinky.rs

#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;
use embedded_graphics::{
    image::{Image, ImageRawLE},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Rectangle, Triangle},
    style::PrimitiveStyleBuilder,
};
use sh1106::{interface::DisplayInterface, prelude::*, Builder};
use stm32f4xx_hal as hal;
use stm32f4xx_hal::{prelude::*, spi, stm32};

// SH1106 Pins
// SPI1 interface (AF5)
// SCK: PA5
// MOSI: PA7
// RST: PA9
// D/C: PC7

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(16.mhz()).freeze();

        let gpioa = dp.GPIOA.split();
        let gpioc = dp.GPIOC.split();

        // Set up OLED GPIO pins
        let mut oled_rst = gpioa.pa9.into_push_pull_output();
        let oled_dc = gpioc.pc7.into_push_pull_output();

        // Set up SPI1 for OLED
        let sck = gpioa.pa5.into_alternate_af5();
        let mosi = gpioa.pa7.into_alternate_af5();
        let spi1 = spi::Spi::spi1(
            dp.SPI1,
            (sck, spi::NoMiso, mosi),
            spi::Mode {
                polarity: spi::Polarity::IdleLow,
                phase: spi::Phase::CaptureOnFirstTransition,
            },
            // Works but seems out of spec of the OLED datasheet ¯\_(ツ)_/¯
            8.mhz().into(),
            clocks,
        );

        let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

        // Set up OLED screen
        let mut disp: GraphicsMode<_> = Builder::new()
            .connect_spi(spi1, oled_dc, sh1106::builder::NoOutputPin::new())
            .into();

        disp.reset(&mut oled_rst, &mut delay).unwrap();
        disp.init().unwrap();

        loop {
            clear(&mut disp);
            draw_kes(&mut disp);
            delay.delay_ms(1000_u32);
            clear(&mut disp);
            draw_shapes(&mut disp);
            delay.delay_ms(1000_u32);
        }
    }

    loop {}
}

/// Clear screen
fn clear<T: DisplayInterface>(disp: &mut GraphicsMode<T>)
where
    T::Error: core::fmt::Debug,
{
    disp.clear();
    disp.flush().unwrap()
}

/// Draw a picture
fn draw_kes<T: DisplayInterface>(disp: &mut GraphicsMode<T>)
where
    T::Error: core::fmt::Debug,
{
    let im: ImageRawLE<BinaryColor> = ImageRawLE::new(include_bytes!("./kes.raw"), 100, 64);
    Image::new(&im, Point::new(14, 0)).draw(disp).unwrap();
    disp.flush().unwrap()
}

/// Draw the embedded_graphics equivalent of hello world
fn draw_shapes<T: DisplayInterface>(disp: &mut GraphicsMode<T>)
where
    T::Error: core::fmt::Debug,
{
    let yoffset = 20;

    let style = PrimitiveStyleBuilder::new()
        .stroke_width(1)
        .stroke_color(BinaryColor::On)
        .build();

    // screen outline
    // default display size is 128x64 if you don't pass a _DisplaySize_
    // enum to the _Builder_ struct
    Rectangle::new(Point::new(0, 0), Point::new(127, 63))
        .into_styled(style)
        .draw(disp)
        .unwrap();

    // triangle
    Triangle::new(
        Point::new(16, 16 + yoffset),
        Point::new(16 + 16, 16 + yoffset),
        Point::new(16 + 8, yoffset),
    )
    .into_styled(style)
    .draw(disp)
    .unwrap();

    // square
    Rectangle::new(Point::new(52, yoffset), Point::new(52 + 16, 16 + yoffset))
        .into_styled(style)
        .draw(disp)
        .unwrap();

    // circle
    Circle::new(Point::new(96, yoffset + 8), 8)
        .into_styled(style)
        .draw(disp)
        .unwrap();

    disp.flush().unwrap();
}
