// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use num::FromPrimitive;

enum_from_primitive! {
    #[derive(Debug)]
    pub enum Opcode {
        JMPA = 0x4C
    }
}

/// Decodes an opcode by converting an opcode number to an enum value.
pub fn decode_opcode(opcode: u8) -> Opcode {
    match Opcode::from_u8(opcode) {
        Some(opcode) => opcode,
        None => { panic!("Unknown opcode detected: {:#X}", opcode); }
    }
}
