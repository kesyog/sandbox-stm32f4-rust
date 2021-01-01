#![no_std]
#![no_main]
/// Interrupt-based example that does three things simultaneously, all via interrupts:
/// * Blinks the on-board LED
/// * Prints out a message over the debug serial port whenever the LED toggles
/// * Echos received serial bytes back to the sender
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use crate::hal::{
    gpio::{gpioa::PA5, Output, PushPull},
    prelude::*,
    stm32::TIM2,
    timer::Timer,
};
use cortex_m::iprintln;
use rtic::app;
use sandbox_stm32f4_rust::uart_driver;
use stm32f4xx_hal as hal;

type LedPin = PA5<Output<PushPull>>;

#[app(device = stm32f4xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        led: LedPin,
        serial_ctx: uart_driver::UartContext,
        itm: cortex_m::peripheral::ITM,
        timer: Timer<TIM2>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let cp = cx.core;
        let dp = cx.device;

        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

        // Set up the LED. On the NUCLEO-F401RE it's connected to pin PA5.
        // Calling split also powers up the GPIOA peripheral clock
        let gpioa = dp.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        let mut timer = hal::timer::Timer::tim2(dp.TIM2, 1.hz(), clocks);
        timer.listen(hal::timer::Event::TimeOut);

        // ST-Link is connected to USART2
        // RX: PA3
        // TX: PA2
        let tx = gpioa.pa2.into_alternate_af7();
        let rx = gpioa.pa3.into_alternate_af7();
        let mut serial = hal::serial::Serial::usart2(
            dp.USART2,
            (tx, rx),
            hal::serial::config::Config::default().baudrate(115200.bps()),
            clocks,
        )
        .unwrap();
        serial.listen(hal::serial::Event::Rxne);

        init::LateResources {
            led,
            serial_ctx: uart_driver::UartContext::new(serial),
            itm: cp.ITM,
            timer,
        }
    }

    #[idle(resources = [itm])]
    fn idle(mut ctx: idle::Context) -> ! {
        ctx.resources.itm.lock(|itm| {
            // Print up a bootup message over ITM
            iprintln!(&mut itm.stim[0], "Hello world!");
        });

        loop {}
    }

    #[task(binds = TIM2, resources = [led, itm, timer, serial_ctx])]
    fn tim2(ctx: tim2::Context) {
        static mut LED_ON: bool = false;

        let serial_ctx = ctx.resources.serial_ctx;
        if *LED_ON {
            ctx.resources.led.set_high().unwrap();
            iprintln!(&mut ctx.resources.itm.stim[0], "on");
            uart_driver::write(serial_ctx, "LED on".bytes());
        } else {
            ctx.resources.led.set_low().unwrap();
            iprintln!(&mut ctx.resources.itm.stim[0], "off");
            uart_driver::write(serial_ctx, "LED off".bytes());
        }
        uart_driver::write(serial_ctx, "\r\n".bytes());
        *LED_ON = !*LED_ON;

        ctx.resources
            .timer
            .clear_interrupt(hal::timer::Event::TimeOut);
    }

    #[task(binds = USART2, resources = [serial_ctx])]
    fn usart2(mut ctx: usart2::Context) {
        uart_driver::interrupt(&mut ctx.resources.serial_ctx);
    }
};
