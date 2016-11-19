// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

const SIGN_BITMASK: u8 = 0b10000000;

/// Checks if an unsigned number would be negative if it was signed. This is
/// done by checking if the 7th bit is set.
#[inline(always)]
pub fn is_negative(arg: u8) -> bool {
    arg & SIGN_BITMASK == SIGN_BITMASK
}

/// Adds a relative displacement to an address. This is useful for operations
/// using relative addressing that allow branching forwards or backwards.
#[inline(always)]
pub fn add_relative(base_addr: u16, displacement: i8) -> u16 {
    if displacement < 0 {
        base_addr.wrapping_sub(-(displacement) as u16)
    } else {
        base_addr.wrapping_add(displacement as u16)
    }
}
