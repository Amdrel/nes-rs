// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use byteorder::{LittleEndian, ReadBytesExt};
use nes::cpu::{CPU, NEGATIVE_FLAG};
use nes::memory::Memory;
use nes::opcode::Opcode::*;
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
        let opcode = self.opcode();
        match opcode {
            JMPAbs => format!("JMP ${:02X}{:02X}", self.2, self.1),
            LDXImm => format!("LDX #${:02X}", self.1),
            _ => { panic!("Unimplemented opcode found: {:?}", opcode); }
        }
    }

    /// Logs a human-readable representation of the instruction along with the
    /// CPU state in an easy to parse format.
    ///
    /// TODO: Return a string for the test suite so CPU correctness can be
    /// checked. Also it may be more appropriate to move this function into the
    /// CPU.
    pub fn log(&self, cpu: &CPU) {
        use nes::opcode::opcode_len;
        let opcode = self.opcode();
        let len = opcode_len(&opcode) as u16;

        // Get human readable hex of the instruction bytes. A pattern match is
        // used as bytes that do not exist in an instruction should not be
        // displayed (rather than displaying the default struct value 0).
        let instr_str = match len {
            1 => format!("{:02X}      ", self.0),
            2 => format!("{:02X} {:02X}   ", self.0, self.1),
            3 => format!("{:02X} {:02X} {:02X}", self.0, self.1, self.2),
            _ => { panic!("Invalid instruction length given"); }
        };

        // Disassemble the instruction to a human readable format for the log.
        let disassembled = self.disassemble();

        // Prints the CPU state and disassembled instruction in a nice parsable
        // format. In the future this output will be used for automatically
        // testing the CPU's accuracy.
        //
        // NOTE: CYC is not cycles like the name sugests, but PPU dots. The PPU
        // can output 3 dots every CPU cycle on NTSC (PAL outputs an extra dot
        // every fifth CPU cycle).
        println!("{:04X}  {}  {:30}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:3}",
                 cpu.pc, instr_str, disassembled, cpu.a, cpu.x, cpu.y, cpu.p,
                 cpu.sp, 0);
    }

    /// Execute the instruction with a routine that corresponds with it's
    /// opcode. All routines for every instruction in the 6502 instruction set
    /// are present here.
    #[inline(always)]
    pub fn execute(&self, cpu: &mut CPU, memory: &mut Memory) {
        use nes::opcode::opcode_len;

        let opcode = self.opcode();
        let len = opcode_len(&opcode) as u16;

        // Execute the internal logic of the instruction based on it's opcode.
        match opcode {
            JMPAbs => {
                cpu.pc = self.arg_u16();
                cpu.cycles += 3;
            },
            LDXImm => {
                cpu.x = self.arg_u8();
                if cpu.x == 0 { cpu.set_zero_flag(); }
                if cpu.x & NEGATIVE_FLAG == NEGATIVE_FLAG { cpu.set_negative_flag(); }
                cpu.cycles += 2;
                cpu.pc += len;
            },
            _ => { panic!("Unimplemented opcode found: {:?}", opcode); }
        };
    }

    /// Obtain the opcode of the instruction.
    #[inline(always)]
    fn opcode(&self) -> Opcode {
        use nes::opcode::decode_opcode;
        decode_opcode(self.0)
    }

    /// Read the instruction argument as an 8-bit value.
    #[inline(always)]
    fn arg_u8(&self) -> u8 {
        self.1
    }

    /// Read the instruction argument as a 16-bit value.
    #[inline(always)]
    fn arg_u16(&self) -> u16 {
        let mut reader = Cursor::new(vec![self.1, self.2]);
        reader.read_u16::<LittleEndian>().unwrap()
    }

    /// Accumulator addressing simply gets values from the accumulator register
    /// rather than from the instruction.
    #[inline(always)]
    fn accumulator(&self, cpu: &CPU) -> u8 {
        cpu.a
    }

    /// Directly return the argument. Immediate addressing simply stores the
    /// value in the argument unlike other addressing modes which typically use
    /// this space for memory addresses.
    #[inline(always)]
    fn immediate(&self) -> u8 {
        self.arg_u8()
    }

    /// Returns an address from the instruction arguments that's between
    /// $00-$FF. This is used for zero page addressing which is typically faster
    /// than it's counterpart absolute addressing.
    #[inline(always)]
    fn zero_page(&self, memory: &mut Memory) -> usize {
        self.arg_u8() as usize
    }

    /// Returns a zero page address stored in the instruction with the X
    /// register added to it.
    #[inline(always)]
    fn zero_page_x(&self, cpu: &CPU, memory: &mut Memory) -> usize {
        self.arg_u8().wrapping_add(cpu.x) as usize
    }

    /// Returns a zero page address stored in the instruction with the Y
    /// register added to it.
    #[inline(always)]
    fn zero_page_y(&self, cpu: &CPU, memory: &mut Memory) -> usize {
        self.arg_u8().wrapping_add(cpu.y) as usize
    }

    /// Returns a signed variation of the 8-bit argument. Relative addressing is
    /// used for branch operations and uses a signed integer containing an
    /// offset of bytes of where to place the program counter.
    #[inline(always)]
    fn relative(&self) -> i8 {
        self.arg_u8() as i8
    }

    /// Returns an address from the instruction argument unaltered.
    #[inline(always)]
    fn absolute(&self) -> usize {
        self.arg_u16() as usize
    }

    /// Returns an address from the instruction argument with the value in the X
    /// register added to it.
    #[inline(always)]
    fn absolute_x(&self, cpu: &CPU) -> usize {
        self.arg_u16().wrapping_add(cpu.x as u16) as usize
    }

    /// Returns an address from the instruction argument with the value in the Y
    /// register added to it.
    #[inline(always)]
    fn absolute_y(&self, cpu: &CPU) -> usize {
        self.arg_u16().wrapping_add(cpu.y as u16) as usize
    }

    /// Indirect addressing uses an absolute address to lookup another address.
    #[inline(always)]
    fn indirect(&self, memory: &mut Memory) -> usize {
        let arg = self.arg_u16() as usize;
        memory.read_u16(arg) as usize
    }

    /// Calculates a memory address using by adding X to the 8-bit value in the
    /// instruction, THEN use that address to find ANOTHER address, then return
    /// THAT address.
    #[inline(always)]
    fn indirect_x(&self, cpu: &CPU, memory: &mut Memory) -> usize {
        let addr = self.arg_u8().wrapping_add(cpu.x) as usize;
        memory.read_u16(addr) as usize
    }

    /// Sane version of indirect_x that gets the zero page address in the
    /// instruction, adds Y to it, then returns the resulting address.
    #[inline(always)]
    fn indirect_y(&self, cpu: &CPU, memory: &mut Memory) -> usize {
        let arg = self.arg_u8() as usize;
        memory.read_u16(arg).wrapping_add(cpu.y as u16) as usize
    }
}
