extern crate getopts;

use getopts::Options;

const EXIT_SUCCESS: u8 = 0;
const EXIT_FAILURE: u8 = 1; // Generic error.

/// Initializes and starts the emulator. Returns an exit code after which the
/// program unwinds and stops executing. Once the emulator starts executing, the
/// application should only stop due to user input, or a panic.
fn init() -> u8 {
    // TODO: Parse command line arguments (e.g rom file).

    println!("Hello, world!");
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
