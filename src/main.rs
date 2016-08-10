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

mod io;
mod nes;

// Exit codes used throughout the application. These exit codes has specific
// meanings and are used when no OS error codes are available.
const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1; // Generic error ¯\_(ツ)_/¯.
const EXIT_INVALID: i32 = 2; // Invalid rom passed.

/// Prints the application name alongside the cargo version.
fn print_version() {
    println!("nes-rs {}", env!("CARGO_PKG_VERSION"));
}

/// Prints usage information with an optional reason.
fn print_usage(opts: Options, reason: Option<&str>) {
    let mut stderr = std::io::stderr();

    // Print the reason only if it was passed.
    match reason {
        Some(r) => {
            writeln!(stderr, "{}", r).unwrap();
        },
        None => {}
    }

    writeln!(stderr, "nes-rs is an incomplete NES emulator written in Rust.").unwrap();
    writeln!(stderr, "").unwrap();
    writeln!(stderr, "{}", opts.usage("Usage: nes-rs [OPTIONS] ROM")).unwrap();
    writeln!(stderr, "To contribute or report bugs, please see:").unwrap();
    writeln!(stderr, "<https://github.com/Reshurum/nes-rs>").unwrap();
}

/// Initializes and starts the emulator. Returns an exit code after which the
/// program unwinds and stops executing. Once the emulator starts executing, the
/// application should only stop due to user input, or a panic.
fn init() -> i32 {
    // Initialize the argument parser and parse them.
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("v", "version", "print version information");
    opts.optflag("h", "help", "print this message");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f.to_string());
            print_usage(opts, None);
            return EXIT_FAILURE
        },
    };

    // Handle flag based arguments.
    if matches.opt_present("v") {
        print_version();
        return EXIT_SUCCESS
    }
    if matches.opt_present("h") {
        print_usage(opts, None);
        return EXIT_SUCCESS
    }

    // Assume the first free argument is the rom filename. Bail if there are
    // no free arguments.
    let rom_file_name = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(opts, Some("nes-rs: no rom passed, cannot start emulation"));
        return EXIT_FAILURE
    };

    // Read the rom from file using the filename passed from the command-line.
    let rom = match io::binutils::read_bin(&rom_file_name) {
        Ok(rom) => rom,
        Err(e) => {
            let mut stderr = std::io::stderr();
            writeln!(stderr, "nes-rs: cannot open {}: {}", rom_file_name, e).unwrap();
            return e.raw_os_error().unwrap()
        }
    };

    // Parse the rom's header to check if it's a valid iNES rom.
    let header = match io::binutils::parse_rom_header(&rom) {
        Ok(header) => header,
        Err(e) => {
            // TODO: Add complain macro or function, too much repetition.
            let mut stderr = std::io::stderr();
            writeln!(stderr, "nes-rs: cannot parse {}: {}", rom_file_name, e).unwrap();
            return EXIT_INVALID
        }
    };
    println!("{:?}", header);

    println!("Hello, emulation scene!");
    EXIT_SUCCESS
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
