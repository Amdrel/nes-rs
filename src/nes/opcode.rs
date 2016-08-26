// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use num::FromPrimitive;

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
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
        BNERel   = 0xD0,
        BPLRel   = 0x10,
        BRKImp   = 0x00,
        BVCRel   = 0x50,
        BVSRel   = 0x70,
        CLCImp   = 0x18,
        CLDImp   = 0xD8,
        CLIImp   = 0x58,
        CLVImp   = 0xB8,
        CMPImm   = 0xC9,
        CMPZero  = 0xC5,
        CMPZeroX = 0xD5,
        CMPAbs   = 0xCD,
        CMPAbsX  = 0xDD,
        CMPAbsY  = 0xD9,
        CMPIndX  = 0xC1,
        CMPIndY  = 0xD1,
        CPXImm   = 0xE0,
        CPXZero  = 0xE4,
        CPXAbs   = 0xEC,
        CPYImm   = 0xC0,
        CPYZero  = 0xC4,
        CPYAbs   = 0xCC,
        DECZero  = 0xC6,
        DECZeroX = 0xD6,
        DECAbs   = 0xCE,
        DECAbsX  = 0xDE,
        DEXImp   = 0xCA,
        DEYImp   = 0x88,
        EORImm   = 0x49,
        EORZero  = 0x45,
        EORZeroX = 0x55,
        EORAbs   = 0x4D,
        EORAbsX  = 0x5D,
        EORAbsY  = 0x59,
        EORIndX  = 0x41,
        EORIndY  = 0x51,
        INCZero  = 0xE6,
        INCZeroX = 0xF6,
        INCAbs   = 0xEE,
        INCAbsX  = 0xFE,
        INXImp   = 0xE8,
        INYImp   = 0xC8,
        JMPAbs   = 0x4C,
        JMPInd   = 0x6C,
        JSRAbs   = 0x20,
        LDAImm   = 0xA9,
        LDAZero  = 0xA5,
        LDAZeroX = 0xB5,
        LDAAbs   = 0xAD,
        LDAAbsX  = 0xBD,
        LDAAbsY  = 0xB9,
        LDAIndX  = 0xA1,
        LDAIndY  = 0xB1,
        LDXImm   = 0xA2,
        LDXZero  = 0xA6,
        LDXZeroY = 0xB6,
        LDXAbs   = 0xAE,
        LDXAbsY  = 0xBE,
        LDYImm   = 0xA0,
        LDYZero  = 0xA4,
        LDYZeroX = 0xB4,
        LDYAbs   = 0xAC,
        LDYAbsX  = 0xBC,
        LSRAcc   = 0x4A,
        LSRZero  = 0x46,
        LSRZeroX = 0x56,
        LSRAbs   = 0x4E,
        LSRAbsX  = 0x5E,
        NOPImp   = 0xEA,
        ORAImm   = 0x09,
        ORAZero  = 0x05,
        ORAZeroX = 0x15,
        ORAAbs   = 0x0D,
        ORAAbsX  = 0x1D,
        ORAAbsY  = 0x19,
        ORAIndX  = 0x01,
        ORAIndY  = 0x11,
        PHAImp   = 0x48,
        PHPImp   = 0x08,
        PLAImp   = 0x68,
        PLPImp   = 0x28,
        ROLAcc   = 0x2A,
        ROLZero  = 0x26,
        ROLZeroX = 0x36,
        ROLAbs   = 0x2E,
        ROLAbsX  = 0x3E,
        RORAcc   = 0x6A,
        RORZero  = 0x66,
        RORZeroX = 0x76,
        RORAbs   = 0x6E,
        RORAbsX  = 0x7E,
        RTIImp   = 0x40,
        RTSImp   = 0x60,
        SBCImm   = 0xE9,
        SBCZero  = 0xE5,
        SBCZeroX = 0xF5,
        SBCAbs   = 0xED,
        SBCAbsX  = 0xFD,
        SBCAbsY  = 0xF9,
        SBCIndX  = 0xE1,
        SBCIndY  = 0xF1,
        SECImp   = 0x38,
        SEDImp   = 0xF8,
        SEIImp   = 0x78,
        STAZero  = 0x85,
        STAZeroX = 0x95,
        STAAbs   = 0x8D,
        STAAbsX  = 0x9D,
        STAAbsY  = 0x99,
        STAIndX  = 0x81,
        STAIndY  = 0x91,
        STXZero  = 0x86,
        STXZeroY = 0x96,
        STXAbs   = 0x8E,
        STYZero  = 0x84,
        STYZeroX = 0x94,
        STYAbs   = 0x8C,
        TAXImp   = 0xAA,
        TAYImp   = 0xA8,
        TSXImp   = 0xBA,
        TXAImp   = 0x8A,
        TXSImp   = 0x9A,
        TYAImp   = 0x98,
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
        BNERel   => 2,
        BPLRel   => 2,
        BRKImp   => 1,
        BVCRel   => 2,
        BVSRel   => 2,
        CLCImp   => 1,
        CLDImp   => 1,
        CLIImp   => 1,
        CLVImp   => 1,
        CMPImm   => 2,
        CMPZero  => 2,
        CMPZeroX => 2,
        CMPAbs   => 3,
        CMPAbsX  => 3,
        CMPAbsY  => 3,
        CMPIndX  => 2,
        CMPIndY  => 2,
        CPXImm   => 2,
        CPXZero  => 2,
        CPXAbs   => 3,
        CPYImm   => 2,
        CPYZero  => 2,
        CPYAbs   => 3,
        DECZero  => 2,
        DECZeroX => 2,
        DECAbs   => 3,
        DECAbsX  => 3,
        DEXImp   => 1,
        DEYImp   => 1,
        EORImm   => 2,
        EORZero  => 2,
        EORZeroX => 2,
        EORAbs   => 3,
        EORAbsX  => 3,
        EORAbsY  => 3,
        EORIndX  => 2,
        EORIndY  => 2,
        INCZero  => 2,
        INCZeroX => 2,
        INCAbs   => 3,
        INCAbsX  => 3,
        INXImp   => 1,
        INYImp   => 1,
        JMPAbs   => 3,
        JMPInd   => 3,
        JSRAbs   => 3,
        LDAImm   => 2,
        LDAZero  => 2,
        LDAZeroX => 2,
        LDAAbs   => 3,
        LDAAbsX  => 3,
        LDAAbsY  => 3,
        LDAIndX  => 2,
        LDAIndY  => 2,
        LDXImm   => 2,
        LDXZero  => 2,
        LDXZeroY => 2,
        LDXAbs   => 3,
        LDXAbsY  => 3,
        LDYImm   => 2,
        LDYZero  => 2,
        LDYZeroX => 2,
        LDYAbs   => 3,
        LDYAbsX  => 3,
        LSRAcc   => 1,
        LSRZero  => 2,
        LSRZeroX => 2,
        LSRAbs   => 3,
        LSRAbsX  => 3,
        NOPImp   => 1,
        ORAImm   => 2,
        ORAZero  => 2,
        ORAZeroX => 2,
        ORAAbs   => 3,
        ORAAbsX  => 3,
        ORAAbsY  => 3,
        ORAIndX  => 2,
        ORAIndY  => 2,
        PHAImp   => 1,
        PHPImp   => 1,
        PLAImp   => 1,
        PLPImp   => 1,
        ROLAcc   => 1,
        ROLZero  => 2,
        ROLZeroX => 2,
        ROLAbs   => 3,
        ROLAbsX  => 3,
        RORAcc   => 1,
        RORZero  => 2,
        RORZeroX => 2,
        RORAbs   => 3,
        RORAbsX  => 3,
        RTIImp   => 1,
        RTSImp   => 1,
        SBCImm   => 2,
        SBCZero  => 2,
        SBCZeroX => 2,
        SBCAbs   => 3,
        SBCAbsX  => 3,
        SBCAbsY  => 3,
        SBCIndX  => 2,
        SBCIndY  => 2,
        SECImp   => 1,
        SEDImp   => 1,
        SEIImp   => 1,
        STAZero  => 2,
        STAZeroX => 2,
        STAAbs   => 3,
        STAAbsX  => 3,
        STAAbsY  => 3,
        STAIndX  => 2,
        STAIndY  => 2,
        STXZero  => 2,
        STXZeroY => 2,
        STXAbs   => 3,
        STYZero  => 2,
        STYZeroX => 2,
        STYAbs   => 3,
        TAXImp   => 1,
        TAYImp   => 1,
        TSXImp   => 1,
        TXAImp   => 1,
        TXSImp   => 1,
        TYAImp   => 1,
    }
}
