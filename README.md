# nes-rs [![Build Status](https://travis-ci.org/Reshurum/nes-rs.svg?branch=master)](https://travis-ci.org/Reshurum/nes-rs)

nes-rs is an incomplete NES emulator written in Rust. The name is subject
to change once I find something more catchy.

## Goals

This is a list of my long term goals for the project that I do not expect
to be done for a long time.

* Make the emulator as accurate as possible
* Automated testing of the CPU with existing test roms
* Automated testing of the PPU (frame by frame compare)
* RPC-like api to allow external scripts change the emulator state
* Full featured debugger accessible through a command-line interface

## Building and Running

For now the only dependency is rust itself, however this is subject to change
once I start working on the PPU and I plan to use SDL.

## License

Licensed under either of the following licenses at your option:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)
