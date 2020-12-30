#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use crate::hal::{
    prelude::*,
    serial::{config::Config, Pins, Serial},
    stm32::{self, usart1, USART2},
};
#[allow(unused_extern_crates)]
use cortex_m::iprintln;
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(mut cp)) = (
        stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        let stim = &mut cp.ITM.stim[0];
        iprintln!(stim, "Boot");

        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

        // Set up the LED. On the NUCLEO-F401RE it's connected to pin PA5.
        // Calling split also powers up the GPIOA peripheral clock
        let gpioa = dp.GPIOA.split();
        let mut led = gpioa.pa5.into_push_pull_output();

        // ST-Link is connected to USART2
        // RX: PA3
        // TX: PA2
        let tx = gpioa.pa2.into_alternate_af7();
        let rx = gpioa.pa3.into_alternate_af7();
        let mut serial = Serial::usart2(
            dp.USART2,
            (tx, rx),
            Config::default().baudrate(115200.bps()),
            clocks,
        )
        .unwrap();
        let mut delay = hal::delay::Delay::new(cp.SYST, clocks);
        let usart2 = unsafe { &mut *(USART2::ptr() as *mut usart1::RegisterBlock) };

        // Use peripheral access crate (PAC) write a character to the serial port's transmission data register
        for byte in b"\r\n" {
            usart2.dr.write(|w| w.dr().bits(u16::from(*byte)));
            // Busy-wait on TXE (transmit data register empty) flag via PAC
            while !usart2.sr.read().txe().bits() {}
        }

        // Using HAL library to write to USART
        for byte in r"¯\_(ツ)_/¯".bytes() {
            serial.write(byte).unwrap();
            // Busy-wait on TXE (transmit data register empty) flag via HAL
            while !serial.is_txe() {}
        }

        loop {
            led.set_high().unwrap();
            for _ in 0..500 {
                if serial.is_rxne() {
                    let byte = serial.read().unwrap();
                    serial.write(byte).unwrap();
                    while !serial.is_txe() {}
                }
                delay.delay_ms(1_u32);
            }
            led.set_low().unwrap();
            for _ in 0..500 {
                if serial.is_rxne() {
                    let byte = serial.read().unwrap();
                    serial.write(byte).unwrap();
                    while !serial.is_txe() {}
                }
                delay.delay_ms(1_u32);
            }
        }
    }

    loop {}
}
