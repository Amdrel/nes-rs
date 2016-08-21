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
        ADCImm   = 0x69,
        ADCZero  = 0x65,
        ADCZeroX = 0x75,
        ADCAbs   = 0x6D,
        ADCAbsX  = 0x7D,
        ADCAbsY  = 0x79,
        ADCIndX  = 0x61,
        ADCIndY  = 0x71,

        ANDImm   = 0x29,
        ANDZero  = 0x25,
        ANDZeroX = 0x35,
        ANDAbs   = 0x2D,
        ANDAbsX  = 0x3D,
        ANDAbsY  = 0x39,
        ANDIndX  = 0x21,
        ANDIndY  = 0x31,

        ASLAcc   = 0x0A,
        ASLZero  = 0x06,
        ASLZeroX = 0x16,
        ASLAbs   = 0x0E,
        ASLAbsX  = 0x1E,

        BCCRel   = 0x90,

        BCSRel   = 0xB0,

        BEQRel   = 0xF0,

        BITZero  = 0x24,
        BITAbs   = 0x2C,

        BMIRel   = 0x30,

        JMPA = 0x4C,

        LDXI = 0xA2,

        STXZ = 0x86,
    }
}

/// Decodes an opcode by converting an opcode number to an enum value.
pub fn decode_opcode(opcode: u8) -> Opcode {
    match Opcode::from_u8(opcode) {
        Some(opcode) => opcode,
        None => { panic!("Unimplemented opcode detected: {:2X}", opcode); }
    }
}

/// Determine the length of an instruction with the given opcode.
pub fn opcode_len(opcode: &Opcode) -> u8 {
    use self::Opcode::*;

    match *opcode {
        ADCImm   => 2,
        ADCZero  => 2,
        ADCZeroX => 2,
        ADCAbs   => 3,
        ADCAbsX  => 3,
        ADCAbsY  => 3,
        ADCIndX  => 2,
        ADCIndY  => 2,

        ANDImm   => 2,
        ANDZero  => 2,
        ANDZeroX => 2,
        ANDAbs   => 3,
        ANDAbsX  => 3,
        ANDAbsY  => 3,
        ANDIndX  => 2,
        ANDIndY  => 2,

        ASLAcc   => 1,
        ASLZero  => 2,
        ASLZeroX => 2,
        ASLAbs   => 3,
        ASLAbsX  => 3,

        BCCRel   => 2,

        BCSRel   => 2,

        BEQRel   => 2,

        BITZero  => 2,
        BITAbs   => 3,

        BMIRel   => 2,

        JMPA     => 3,

        LDXI     => 2,

        STXZ     => 2,
    }
}
