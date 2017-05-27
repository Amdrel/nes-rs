// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Local};
use nes::nes::NESRuntimeOptions;

/// Logs a message to stdout with a given prefix if the emulator was started
/// with the verbose flag set.
pub fn log<P, T>(prefix: P, text: T, runtime_options: &NESRuntimeOptions) where P: Into<String>, T: Into<String> {
    if runtime_options.verbose {
        let local: DateTime<Local> = Local::now();
        println!("[{}] -- [{}] {}", local, prefix.into(), text.into());
    }
}
