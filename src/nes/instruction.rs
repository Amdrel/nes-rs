// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use byteorder::{LittleEndian, ReadBytesExt};
use nes::cpu::CPU;
use nes::memory::Memory;
use nes::opcode::Opcode;
use std::io::Cursor;

/// All 6502 instructions are a maximum size of 3 bytes. The first byte is the
/// opcode which is determines the action of the instruction. The following 2
/// bytes are the arguments and are present depending on the opcode.
#[derive(Debug)]
pub struct Instruction(pub u8, pub u8, pub u8);

impl Instruction {
    /// Parses an instruction from memory at the address of the passed program
    /// counter. Some instructions when parsed by the original 6502 will read
    /// arguments from the wrong addresses (e.g indirect JMP), so those bugs are
    /// emulated accurately here.
    pub fn parse(pc: usize, memory: &mut Memory) -> Instruction {
        use nes::opcode::decode_opcode;
        use nes::opcode::opcode_len;

        let raw_opcode = memory.read_u8(pc);
        let opcode = decode_opcode(raw_opcode);
        let len = opcode_len(&opcode);

        // TODO: Check for indirect JMP to emulate page boundary bug.

        match len {
            1 => Instruction(raw_opcode, 0, 0),
            2 => Instruction(raw_opcode, memory.read_u8(pc + 1), 0),
            3 => Instruction(raw_opcode, memory.read_u8(pc + 1),
                memory.read_u8(pc + 2)),
            _ => { panic!("Invalid instruction length returned") }
        }
    }

    /// Disassembles the instruction into human readable assembly.
    pub fn disassemble(&self) -> String {
        match self.opcode() {
            Opcode::JMPA => format!("JMP ${:2X}{:2X}", self.2, self.1)
        }
    }

    /// Logs a human-readable representation of the instruction along with the
    /// CPU state in an easy to parse format.
    ///
    /// TODO: Return a string for the test suite so CPU correctness can be
    /// checked. Also it may be more appropriate to move this function into the
    /// CPU.
    pub fn log(&self, cpu: &CPU) {
        // Prints the CPU state and disassembled instruction in a nice parsable
        // format. In the future this output will be used for automatically
        // testing the CPU's accuracy.
        //
        // NOTE: CYC is not cycles like the name sugests, but PPU dots. The PPU
        // can output 3 dots every CPU cycle on NTSC (PAL outputs an extra dot
        // every fifth CPU cycle).
        let disassembled = self.disassemble();
        println!("{:04X}  {:02X} {:02X} {:02X}  {:30}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:3}",
                 cpu.pc, self.0, self.1, self.2, disassembled, cpu.a, cpu.x,
                 cpu.y, cpu.p, cpu.sp, 0);
    }

    /// Obtain the opcode of the instruction.
    #[inline(always)]
    pub fn opcode(&self) -> Opcode {
        use nes::opcode::decode_opcode;
        decode_opcode(self.0)
    }

    /// Read the instruction argument as an 8-bit value.
    #[inline(always)]
    pub fn arg_u8(&self) -> u8 {
        self.1
    }

    /// Read the instruction argument as a 16-bit value.
    #[inline(always)]
    pub fn arg_u16(&self) -> u16 {
        let mut reader = Cursor::new(vec![self.1, self.2]);
        reader.read_u16::<LittleEndian>().unwrap()
    }

    /// Execute the instruction with a routine that corresponds with it's
    /// opcode.
    #[inline(always)]
    pub fn execute(&self, cpu: &mut CPU, memory: &mut Memory) {
        match self.opcode() {
            Opcode::JMPA => {
                cpu.pc = self.arg_u16();
                cpu.cycles += 3;
            }
        };
    }
}
