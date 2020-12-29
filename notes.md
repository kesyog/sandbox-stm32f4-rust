# Notes

A place to jot down useful notes. Most if not all of this was lifted from the linked references, but
I'm copying the highlights here for my own notes.

**TODO: move to its own mdbook and separate repo**

## Glossary

* peripheral access crate (PAC): device crate (e.g. stm32f4) created using the `svd2rust` crate that
presents an API to access a particular microcontroller's registers
* hardware abstraction library (HAL) crate: higher-level abstraction on top of the PAC. This follows
the generic framework laid out by the `embedded_hal` crate.

## Braindump

### Getting started

The easiest way to get started on a ARM Cortex-M processor is to use the [cortex-m-quickstart template project](https://github.com/rust-embedded/cortex-m-quickstart).
This provides a linker script template and various other boilerplate.

### Concurrency and Ownership

The svd2rust/PAC/HAL API represents peripherals as singletons and takes advantage of Rust's
ownership rules to protect access to them. It can be bypassed using unsafe. See svd2rust's [Peripheral API notes](https://docs.rs/svd2rust/0.17.0/svd2rust/#peripheral-api).

Passing these singletons around and/or sharing them can be a pain, but on the plus side, Rust is
forcing you to think about concurrency and making it harder to shoot yourself in the foot.
There's a great writeup on various techniques [here](https://rust-embedded.github.io/book/concurrency/index.html).

| Technique | Multiple functions within one thread | Multiple threads | Thread and interrupt |
|-----------|--------------------------------------|------------------|----------------------|
| Pass singleton by value |  ✅ | ✅ (can move into thread at thread creation) | n/a |
| Unsafe code (e.g. direct register access, global UnsafeCell) |  ✅ (but would rather stick to safe code if possible) | ⚠ (bypasses safety checks) | ⚠ (bypasses safety checks) |
| Global `Cell`/`RefCell` protected by mutex | ✅ (but adds unnecessary overhead) | ✅ | ⛔ (shouldn't lock in an interrupt) |
| Global `Cell`/`RefCell` protected by critical section (e.g. via `cortex_m::interrupt`) |✅ (but adds unnecessary overhead and interrupt masking) | ✅ | ✅ |
| Global atomic cell (e.g. crossbeam `AtomicCell`) |✅ | ✅ | ✅ |
| [cmim](https://github.com/jamesmunns/cmim) or [irq](https://github.com/jonas-schievink/irq) crate | ❓ (maybe for moving) | ❓ (maybe for moving) | ✅  (for moving) |

The [RTIC](https://github.com/rtic-rs) concurrency framework is supposedly another solution, but I
haven't looked into it at all yet.

### Interrupts

* The [cortex-m-rt-macros](https://docs.rs/cortex-m-rt-macros/0.1.5/cortex_m_rt_macros/attr.interrupt.html)
crate defines an `interrupt` attribute macro that can be applied to a function to override the
default interrupt handler. The macro is re-exported by the PAC and HAL crates. These crates should
have an enum defining what the valid interrupt handler names are e.g. [`stm32f4xx_hal::interrupt`](https://docs.rs/stm32f4xx-hal/0.8.3/stm32f4xx_hal/enum.interrupt.html).

