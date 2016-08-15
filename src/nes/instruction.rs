// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use nes::cpu::CPU;
use nes::opcode::Opcode;
use num::FromPrimitive;

/// All 6502 instructions are a maximum size of 3 bytes. The first byte is the
/// opcode which is determines the action of the instruction. The following 2
/// bytes are the arguments and are present depending on the opcode.
#[derive(Debug)]
pub struct Instruction(u8, u8, u8);

impl Instruction {
    /// The decoder will store all bytes in the instruction regardless if they
    /// are needed or not.
    ///
    /// TODO: Add page boundary flag for instructions that need extra cycles to
    /// decode across pages.
    pub fn decode(instr: &[u8], boundary: bool) -> (Instruction, u8, u16) {
        use nes::opcode::decode_opcode;

        // Determine the length of the instruction based on the opcode.
        let opcode = decode_opcode(instr[0]);
        let (len, mut cycles, boundary_sensitive) = match opcode {
            Opcode::JMPA => (3, 3, false)
        };

        // Add an additional cycle when page boundaries are crossed while
        // parsing arguments (only if the opcode is susceptible to it).
        if boundary_sensitive && boundary {
            cycles += 1;
        }

        // Return the instruction with it's arguments filled along with the
        // amount of cycles it will take for it to execute.
        (match len {
            1 => Instruction(instr[0], 0, 0),
            2 => Instruction(instr[0], instr[1], 0),
            3 => Instruction(instr[0], instr[1], instr[2]),
            _ => { panic!("Invalid length calculated"); }
        }, len, cycles)
    }

    pub fn log(&self, cpu: &CPU) {
        // Disassemble the instruction based on the opcode.
        let disassembled = match self.opcode() {
            Opcode::JMPA => format!("JMP ${:2X}{:2X}", self.2, self.1)
        };

        // TODO: Add CPU state information.
        println!("{:04X}  {:02X} {:02X} {:02X}  {:30}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:3}",
                 cpu.pc, self.0, self.1, self.2, disassembled, cpu.a, cpu.x,
                 cpu.y, cpu.p, cpu.sp, cpu.cycles);
    }

    #[inline(always)]
    pub fn opcode(&self) -> Opcode {
        use nes::opcode::decode_opcode;
        decode_opcode(self.0)
    }
}
