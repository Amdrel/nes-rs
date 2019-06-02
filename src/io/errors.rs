// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Exit codes used throughout the application. These exit codes has specific
// meanings and are used when no OS error codes are available.
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_FAILURE: i32 = 1; // Generic error ¯\_(ツ)_/¯.
pub const EXIT_INVALID_ROM: i32 = 2; // Invalid rom passed.
pub const EXIT_CPU_LOG_NOT_FOUND: i32 = 3;
pub const EXIT_INVALID_PC: i32 = 4;
pub const EXIT_RUNTIME_FAILURE: i32 = 101;
