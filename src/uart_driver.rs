//! Janky interrupt-based serial driver. It can transmit bytes via [write_byte] or [write].
//! Otherwise it echos any received bytes back to the sender
//!
use core::iter::IntoIterator;
use heapless::{consts::U32, spsc::SingleCore};
use stm32f4xx_hal as hal;
use stm32f4xx_hal::{
    gpio::{
        gpioa::{PA2, PA3},
        AF7,
    },
    prelude::*,
    stm32::USART2,
};

pub type UartPeripheral = hal::serial::Serial<
    USART2,
    (
        PA2<hal::gpio::Alternate<AF7>>,
        PA3<hal::gpio::Alternate<AF7>>,
    ),
>;

pub type Queue = heapless::spsc::Queue<u8, U32, u8, SingleCore>;

pub struct UartContext {
    pub handle: UartPeripheral,
    pub rx_queue: Queue,
    pub tx_queue: Queue,
    pub tx_pending: bool,
}

impl UartContext {
    pub fn new(handle: UartPeripheral) -> Self {
        Self {
            handle,
            rx_queue: unsafe { Queue::u8_sc() },
            tx_queue: unsafe { Queue::u8_sc() },
            tx_pending: false,
        }
    }
}

pub fn write_byte(ctx: &mut UartContext, byte: u8) {
    if ctx.tx_pending {
        ctx.tx_queue.enqueue(byte).ok();
    } else {
        ctx.handle.write(byte).ok();
        ctx.tx_pending = true;
        ctx.handle.listen(hal::serial::Event::Txe);
    }
}

pub fn write<T: IntoIterator<Item = u8>>(ctx: &mut UartContext, bytes: T) {
    for byte in bytes.into_iter() {
        write_byte(ctx, byte);
    }
}

pub fn interrupt(ctx: &mut UartContext) {
    if ctx.handle.is_rxne() {
        if let Some(rx_byte) = ctx.handle.read().ok() {
            ctx.tx_queue.enqueue(rx_byte).ok();
            // Drop oldest data if the queue is full
            if ctx.rx_queue.len() == ctx.rx_queue.capacity() {
                ctx.rx_queue.dequeue().unwrap();
            }
            ctx.rx_queue.enqueue(rx_byte).unwrap();
            if rx_byte == b'\r' {
                ctx.tx_queue.enqueue(b'\n').ok();
                ctx.tx_queue.enqueue(b'$').ok();
                ctx.tx_queue.enqueue(b'>').ok();
                ctx.tx_queue.enqueue(b' ').ok();
            }
        }
    }

    if ctx.handle.is_txe() {
        if let Some(next_byte) = ctx.tx_queue.dequeue() {
            ctx.handle.write(next_byte).ok();
            ctx.handle.listen(hal::serial::Event::Txe);
            ctx.tx_pending = true;
        } else {
            // Nothing more to send
            ctx.handle.unlisten(hal::serial::Event::Txe);
            ctx.tx_pending = false;
        }
    } else if !ctx.tx_pending {
        if let Some(next_byte) = ctx.tx_queue.dequeue() {
            ctx.handle.write(next_byte).ok();
            ctx.handle.listen(hal::serial::Event::Txe);
            ctx.tx_pending = true;
        }
    }
}
