#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

#[allow(unused_extern_crates)]
use crate::hal::{
    prelude::*,
    stm32::{self, TIM2},
    timer::Timer,
};
use core::cell::Cell;
use cortex_m::{
    interrupt::{self, Mutex},
    iprintln
};
use cortex_m_rt::entry;
use embedded_hal::timer::{Cancel, CountDown};
use stm32f4xx_hal as hal;

// Even though this is a contrived example, it's a little silly to wrap these peripherals in a
// "mutex"/critical section just to pass them to a helper function (`delay`) without having to
// explicitly pass them as arguments.
// Alternatives:
// * Suck it up and explicitly move the needed peripheral into the helper function as an argument.
// This is probably the preferred solution
// * Use something like crossbeam's AtomicCell
// * Unsafe code (core::cell::UnsafeCell?)
// * Some crate
static TIM2_PERIPH: Mutex<Cell<Option<Timer<TIM2>>>> = Mutex::new(Cell::new(None));

#[inline(never)]
fn delay() {
    interrupt::free(|cs| {
        let cell = TIM2_PERIPH.borrow(cs);
        if let Some(mut tim2) = cell.replace(None) {
            tim2.start(1.hz());
            while let Err(_) = tim2.wait() {}
            cell.replace(Some(tim2));
        }
    });
}

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(mut cp)) = (
        stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {

        let stim = &mut cp.ITM.stim[0];
        iprintln!(stim, "Boot");
        // Set up the LED. On the NUCLEO-F401RE it's connected to pin PA5.
        // Calling split also powers up the GPIOA peripheral clock
        let gpioa = dp.GPIOA.split();
        let mut led = gpioa.pa5.into_push_pull_output();

        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(16.mhz()).freeze();

        let mut timer = hal::timer::Timer::tim2(dp.TIM2, 1.hz(), clocks);
        timer.cancel().unwrap();
        timer.clear_interrupt(hal::timer::Event::TimeOut);

        interrupt::free(|cs| {
            TIM2_PERIPH.borrow(cs).replace(Some(timer));
        });

        loop {
            led.set_low().unwrap();
            delay();
            led.set_high().unwrap();
            delay();
        }
    }

    loop {}
}
