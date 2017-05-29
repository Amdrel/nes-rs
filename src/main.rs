// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use] extern crate enum_primitive;
extern crate byteorder;
extern crate getopts;
extern crate num;
extern crate chrono;
extern crate rustyline;

mod debugger;
mod io;
mod nes;
mod utils;

use getopts::Options;
use io::binutils::INESHeader;
use io::errors::*;
use nes::nes::NES;
use nes::nes::NESRuntimeOptions;
use std::env;
use std::io::Write;
use std::u16;

/// Prints the application name alongside the cargo version.
fn print_version() {
    println!("nes-rs {}", env!("CARGO_PKG_VERSION"));
}

/// Prints usage information with an optional reason.
fn print_usage(opts: Options, reason: Option<&str>) {
    let mut stderr = std::io::stderr();
    match reason {
        Some(r) => {
            writeln!(stderr, "{}", r).unwrap();
        },
        None => {}
    }
    writeln!(stderr, "nes-rs is an incomplete NES emulator written in Rust.").unwrap();
    writeln!(stderr, "").unwrap();
    writeln!(stderr, "{}", opts.usage("Usage: nes-rs [OPTION]... [FILE]")).unwrap();
    writeln!(stderr, "To contribute or report bugs, please see:").unwrap();
    writeln!(stderr, "<https://github.com/Reshurum/nes-rs>").unwrap();
}

/// Initializes and starts the emulator. Returns an exit code after which the
/// program unwinds and stops executing. Once the emulator starts executing, the
/// application should only stop due to user input, or a panic.
fn init() -> i32 {
    // Collect the argument from the environment (command-line arguments).
    let args: Vec<String> = env::args().collect();

    // Initialize the argument parser and parse the args with getopts using the
    // rules defined against the option object.
    let mut opts = Options::new();
    opts.optopt("t", "test", "test the emulator against a CPU log", "[FILE]");
    opts.optopt("p", "program-counter", "set the initial program counter to a specified address", "[HEX]");
    opts.optflag("v", "verbose", "display CPU frame information");
    opts.optflag("", "version", "print version information");
    opts.optflag("h", "help", "print this message");
    opts.optflag("d", "debug", "allow use of the CPU debugger");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f.to_string());
            print_usage(opts, None);
            return EXIT_FAILURE
        },
    };

    // Handle flag based arguments.
    if matches.opt_present("version") {
        print_version();
        return EXIT_SUCCESS
    }
    if matches.opt_present("help") {
        print_usage(opts, None);
        return EXIT_SUCCESS
    }

    // Get the ROM filename from the first free argument and read the ROM into
    // memory (vector of bytes). The ROM is a required argument.
    let rom_file_name = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(opts, Some("nes-rs: no rom passed, cannot start emulation"));
        return EXIT_FAILURE
    };
    let rom = match io::binutils::read_bin(&rom_file_name) {
        Ok(rom) => rom,
        Err(e) => {
            let mut stderr = std::io::stderr();
            writeln!(stderr, "nes-rs: cannot open {}: {}", rom_file_name, e).unwrap();
            return e.raw_os_error().unwrap()
        }
    };

    // Parse the rom's header to check if it's a valid iNES ROM and store it in
    // an internal structure.
    let header = match INESHeader::new(&rom) {
        Ok(header) => header,
        Err(e) => {
            let mut stderr = std::io::stderr();
            writeln!(stderr, "nes-rs: cannot parse {}: {}", rom_file_name, e).unwrap();
            return EXIT_INVALID_ROM
        }
    };

    // Parse the program counter argument if specified which will then be passed
    // to the CPU later on.
    //
    // The first 2 characters in the hex string are to be skipped if they're
    // "0x" as users are likely to insert this when inputting hexadecimal
    // numbers. Otherwise just convert the hex string to a 16-bit unsigned
    // integer as-is.
    let program_counter = match matches.opt_str("program-counter") {
        Some(arg) => {
            let hex = if arg.len() >= 2 && &arg[0..2] == "0x" {
                &arg[2..]
            } else {
                arg.as_str()
            };
            match u16::from_str_radix(hex, 16) {
                Ok(pc) => Some(pc),
                Err(e) => {
                    let mut stderr = std::io::stderr();
                    writeln!(stderr, "nes-rs: cannot parse program counter: {}", e).unwrap();
                    return EXIT_INVALID_PC;
                },
            }
        },
        None => None,
    };

    // Initialize the NES with the mapper specified in the INES file and start
    // executing the ROM. The run function will only return when there is a
    // panic in the CPU or other emulated hardware.
    let runtime_options = NESRuntimeOptions {
        program_counter: program_counter,
        cpu_log:         matches.opt_str("test"),
        verbose:         matches.opt_present("verbose"),
        debugging:       matches.opt_present("debug"),
    };
    let mut nes = NES::new(rom, header, runtime_options);
    nes.run()
}

/// Entry point of the program and wrapper of init. Takes the exit code returned
/// from init and exits with it.
fn main() {
    // std::process::exit requires a signed 32 bit integer, however POSIX
    // systems cannot have an exit code greater than 8 bits so that is what the
    // init function returns.
    let exit_code = init();
    std::process::exit(exit_code); // Unwinding done, safe to exit.
}
