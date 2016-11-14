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

/// Returns true if the sign bit is set.
pub fn sign_bit_set(arg: u8) -> bool {
    arg >> 7 == 0x1
}
