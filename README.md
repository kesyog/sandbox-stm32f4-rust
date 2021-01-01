# Nucleo-F401RE Rust sandbox

A sandbox with some basic examples created to help wrap my head around what using Rust for embedded
development looks like and keep a running list of helpful resources.

There's only a tiny bit new here beyond what's already in the linked embedded tutorials or the
examples available in the linked crates, but maybe the links down below are useful to someone 🤷🏽‍♂

## Hardware

I'm using an STM32 Nucleo-64 dev board I had laying around that's based around an STM32F401RE. Some
specs:

* 84MHz ARM Cortex M4 with floating point instructions
* 512K Flash, 96K SRAM
* Built-in ST-LINK debugger/programmer
* An LED and a button
* Lots of exposed pins

Let's push this board to the edge by blinking some LEDs 🔥

## Plans

Check out the examples folder for some ugly implementations.

* [x] Follow the Rust embedded discovery [book](https://docs.rust-embedded.org/discovery/index.html)
  * [x] blinky
  * [x] Hello world via ITM
  * [x] UART echo server
  * [x] blinky with hardware timer-based busy-waits
* [x] blinky using timer interrupts
* [x] UART echo server using interrupts
* [x] RTIC-based UART echo server
* [x] Draw some things on an SH1106 OLED display with the [embedded\_graphics](https://github.com/embedded-graphics/embedded-graphics)
  crate and the [sh1106](https://github.com/jamwaffles/sh1106) driver crate.
* [ ] Figure out DMA
* [ ] ???
* [ ] 🚀

## License

This repository is licensed under the [MIT license](LICENSE)

## References

This repository was originally generated from the [cortex-m-quickstart template](https://github.com/rust-embedded/cortex-m-quickstart).

* [awesome-embedded-rust](https://github.com/rust-embedded/awesome-embedded-rust) ⭐⭐⭐
* [Embedded Rust docs](https://docs.rust-embedded.org), which includes the [discovery tutorial](https://docs.rust-embedded.org/discovery/index.html)
* [cortex-m-quickstart template project](https://github.com/rust-embedded/cortex-m-quickstart). This
  sandbox repository was originally generated from this template.
* [NUCLEO-F401RE documentation](https://www.st.com/en/evaluation-tools/nucleo-f401re.html)
* [STM32F401 reference manual](https://www.st.com/resource/en/reference_manual/dm00096844-stm32f401xbc-and-stm32f401xde-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
* [stm32-rs](https://github.com/stm32-rs)
* [The embedonomicon](https://docs.rust-embedded.org/embedonomicon/index.html): some advanced topics

### Useful crates

* [svd2rust](https://docs.rs/svd2rust/0.17.0/svd2rust/index.html)
* [embedded\_hal](https://docs.rs/embedded-hal/0.2.4/embedded_hal/index.html)
* [stm32f4xx\_hal](https://docs.rs/stm32f4xx-hal/0.8.3/stm32f4xx_hal)
* [stm32f4.stm32f401](https://docs.rs/stm32f4/0.12.1/stm32f4/stm32f401/index.html)
* [RTIC concurrency framework](https://rtic.rs/0.5/book/en/preface.html)
* [embedded\_graphics](https://github.com/embedded-graphics/embedded-graphics)
