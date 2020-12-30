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
    gpio::{
        gpioa::{PA2, PA3, PA5},
        Output, PushPull, AF7,
    },
    interrupt,
    prelude::*,
    serial::{config::Config, Serial},
    stm32::{self, Interrupt, TIM2, USART2},
    timer::Timer,
};
use cmim::{Context, Move};
use cortex_m::{interrupt::Mutex, iprintln};
use cortex_m_rt::entry;
use serial_driver::UART_HANDLE;
use stm32f4xx_hal as hal;

type LedPin = PA5<Output<PushPull>>;

// Data that is "moved" into the interrupt via the cmim crate
struct LedContext {
    pin: LedPin,
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
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

        // Don't think there's a way to do this without unsafe code
        unsafe {
            cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
            cortex_m::peripheral::NVIC::unmask(Interrupt::USART2);
        }

        // Print up a bootup message over ITM
        let stim = &mut cp.ITM.stim[0];
        iprintln!(stim, "Hello world!");

        // Set up the LED. On the NUCLEO-F401RE it's connected to pin PA5.
        // Calling split also powers up the GPIOA peripheral clock
        let gpioa = dp.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        let mut timer = hal::timer::Timer::tim2(dp.TIM2, 1.hz(), clocks);
        timer.listen(hal::timer::Event::TimeOut);
        LEDS.try_move(LedContext { pin: led, timer }).ok();

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
        serial.listen(hal::serial::Event::Rxne);

        cortex_m::interrupt::free(|cs| {
            *UART_HANDLE.borrow(cs).borrow_mut() = Some(serial);
        });
    }

    loop {}
}

#[interrupt]
fn TIM2() {
    static mut LED_ON: bool = false;

    LEDS.try_lock(|led_ctx| {
        if *LED_ON {
            led_ctx.pin.set_low().unwrap();
        } else {
            led_ctx.pin.set_high().unwrap();
        }
        *LED_ON = !*LED_ON;
        led_ctx.timer.clear_interrupt(hal::timer::Event::TimeOut);
    })
    .ok();

    if *LED_ON {
        serial_driver::write("LED on".bytes());
    } else {
        serial_driver::write("LED off".bytes());
    }
    serial_driver::write("\r\n".bytes());
}

mod serial_driver {
    /// Janky interrupt-based serial driver. It can transmit bytes via [write_byte] or [write].
    /// Otherwise it echos any received bytes back to the sender
    use super::*;
    use core::{
        cell::RefCell,
        iter::IntoIterator,
        ops::DerefMut,
        sync::atomic::{AtomicBool, Ordering},
    };
    use heapless::mpmc::Q32;

    type DebugUart = Serial<
        USART2,
        (
            PA2<hal::gpio::Alternate<AF7>>,
            PA3<hal::gpio::Alternate<AF7>>,
        ),
    >;

    // Lock-free MPMC queue. Not super space efficient and doesn't support peeking but otherwise
    // convenient for hacking things together.
    static TX_QUEUE: Q32<u8> = Q32::new();
    static TX_PENDING: AtomicBool = AtomicBool::new(false);
    // Need true sharing of UART peripheral between interrupt and user code
    pub static UART_HANDLE: Mutex<RefCell<Option<DebugUart>>> = Mutex::new(RefCell::new(None));

    pub fn write_byte(byte: u8) {
        cortex_m::interrupt::free(|cs| {
            let mut cell = UART_HANDLE.borrow(cs).borrow_mut();
            let serial = cell.deref_mut().as_mut().unwrap();

            if TX_PENDING.load(Ordering::Acquire) {
                TX_QUEUE.enqueue(byte).ok();
            } else {
                serial.write(byte).ok();
                TX_PENDING.store(true, Ordering::Release);
                serial.listen(hal::serial::Event::Txe);
            }
        });
    }

    pub fn write<T: IntoIterator<Item = u8>>(bytes: T) {
        for byte in bytes.into_iter() {
            write_byte(byte);
        }
    }

    #[interrupt]
    fn USART2() {
        cortex_m::interrupt::free(|cs| {
            let mut cell = UART_HANDLE.borrow(cs).borrow_mut();
            let serial = cell.deref_mut().as_mut().unwrap();

            if serial.is_rxne() {
                if let Some(rx_byte) = serial.read().ok() {
                    TX_QUEUE.enqueue(rx_byte).ok();
                    if rx_byte == b'\r' {
                        TX_QUEUE.enqueue(b'\n').ok();
                        TX_QUEUE.enqueue(b'$').ok();
                        TX_QUEUE.enqueue(b'>').ok();
                        TX_QUEUE.enqueue(b' ').ok();
                    }
                }
            }

            if serial.is_txe() {
                if let Some(next_byte) = TX_QUEUE.dequeue() {
                    serial.write(next_byte).ok();
                    serial.listen(hal::serial::Event::Txe);
                    TX_PENDING.store(true, Ordering::Release);
                } else {
                    // Nothing more to send
                    serial.unlisten(hal::serial::Event::Txe);
                    TX_PENDING.store(false, Ordering::Release);
                }
            } else if !TX_PENDING.load(Ordering::Acquire) {
                if let Some(next_byte) = TX_QUEUE.dequeue() {
                    serial.write(next_byte).ok();
                    serial.listen(hal::serial::Event::Txe);
                    TX_PENDING.store(true, Ordering::Release);
                }
            }
        });
    }
}
