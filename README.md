# STM32F401 Nucleo-64 Sandbox

A sandbox to play with my STM32 Nucleo-64 development board. The board is based on a STM32F401RE,
which provides:

* 84MHz ARM Cortex M4
* 512K Flash
* 96K SRAM
* Built-in ST-LINK debugger/programmer
* An LED and a button

This repository was generated from the [cortex-m-quickstart template](https://github.com/rust-embedded/cortex-m-quickstart).

## Plans

* [ ] Follow the Rust embedded discovery [book](https://docs.rust-embedded.org/discovery/index.html)
  * [x] blinky
  * [x] Hello world via ITM
* [ ] Get button input working
* [ ] Get input working
* [ ] Move on to the STM32F7 Discovery boards I have laying around and drive their touchscreen
displays
* [ ] ???
* [ ] 🚀


## Board specs

* ARM Cortex M4 84MHz
* 512-KB Flash, 96-KB SRAM

## License

This repository is licensed under the [MIT license](LICENSE)

## References

### Useful crate docs

* [svd2rust](https://docs.rs/svd2rust/0.17.0/svd2rust/index.html)
* [embedded\_hal](https://docs.rs/embedded-hal/0.2.4/embedded_hal/index.html)
* [stm3244xx\_hal](https://docs.rs/stm32f4xx-hal/0.8.3/stm32f4xx_hal)
* [stm32f4.stm32f401](https://docs.rs/stm32f4/0.12.1/stm32f4/stm32f401/index.html)

### Tutorials

* [Rust embedded discovery book](https://docs.rust-embedded.org/discovery/index.html)
* [Rust embedded book](https://rust-embedded.github.io/book/intro/index.html)

### Other links

* [NUCLEO-F401RE documentation](https://www.st.com/en/evaluation-tools/nucleo-f401re.html)
* [stm32-rs](https://github.com/stm32-rs)
* [awesome-embedded-rust](https://github.com/rust-embedded/awesome-embedded-rust)

