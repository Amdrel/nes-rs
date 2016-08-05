// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate getopts;

use getopts::Options;
use std::env;
use std::io::Write;

const EXIT_SUCCESS: u8 = 0;
const EXIT_FAILURE: u8 = 1; // Generic error ¯\_(ツ)_/¯.

/// Prints the application name alongside the cargo version.
fn print_version() {
    println!("nes-rs {}", env!("CARGO_PKG_VERSION"));
}

/// Prints usage information.
fn print_usage(opts: Options) {
    let mut stderr = std::io::stderr();

    writeln!(stderr, "nes-rs is an incomplete NES emulator written in Rust.").unwrap();
    writeln!(stderr, "").unwrap();
    writeln!(stderr, "{}", opts.usage("Usage: nes-rs [OPTIONS] ROM")).unwrap();
    writeln!(stderr, "To contribute or report bugs, please see:").unwrap();
    writeln!(stderr, "<https://github.com/Reshurum/nes-rs>").unwrap();
}

/// Initializes and starts the emulator. Returns an exit code after which the
/// program unwinds and stops executing. Once the emulator starts executing, the
/// application should only stop due to user input, or a panic.
fn init() -> u8 {
    // Initialize the argument parser and parse them.
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("v", "version", "print version information");
    opts.optflag("h", "help", "print this message");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f.to_string());
            print_usage(opts);
            return EXIT_FAILURE
        },
    };

    if matches.opt_present("v") {
        print_version();
        return EXIT_SUCCESS
    }
    if matches.opt_present("h") {
        print_usage(opts);
        return EXIT_SUCCESS
    }

    // Assume the first free argument is the rom filename.
    let rom_file_name = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(opts);
        return EXIT_FAILURE
    };

    println!("Hello, emulation scene!");
    EXIT_SUCCESS
}

/// Entry point of the program and wrapper of init. Takes the exit code returned
/// from init and exits with it.
fn main() {
    // std::process::exit requires a signed 32 bit integer, however POSIX
    // systems cannot have an exit code greater than 8 bits so that is what the
    // init function returns.
    let exit_code = init() as i32;
    std::process::exit(exit_code); // Unwinding done, safe to exit.
}
