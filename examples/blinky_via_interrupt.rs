#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

#[allow(unused_extern_crates)]
use crate::hal::{
    gpio::{gpioa::PA5, Output, PushPull},
    interrupt,
    prelude::*,
    stm32::{self, Interrupt, TIM2},
    timer::Timer,
};
use cmim::{Context, Move};
use cortex_m::iprintln;
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

// Data that is "moved" into the interrupt via the cmim crate
struct LedContext {
    on: bool,
    pin: PA5<Output<PushPull>>,
    timer: Timer<TIM2>,
}

// Using cmim crate to move control of the LED GPIO pin and timer peripheral to the interrupt
// Could also use a critical section or atomic cell here
static LEDS: Move<LedContext, Interrupt> =
    Move::new_uninitialized(Context::Interrupt(Interrupt::TIM2));

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
        let led = gpioa.pa5.into_push_pull_output();

        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

        unsafe {
            cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
        }

        let mut timer = Timer::tim2(dp.TIM2, 1.hz(), clocks);
        timer.listen(hal::timer::Event::TimeOut);
        LEDS.try_move(LedContext {
            on: false,
            pin: led,
            timer,
        })
        .ok();
    }

    loop {}
}

#[interrupt]
fn TIM2() {
    LEDS.try_lock(|led_ctx| {
        if led_ctx.on {
            led_ctx.pin.set_low().unwrap();
        } else {
            led_ctx.pin.set_high().unwrap();
        }
        led_ctx.on = !led_ctx.on;
        led_ctx.timer.clear_interrupt(hal::timer::Event::TimeOut);
    })
    .ok();
}
