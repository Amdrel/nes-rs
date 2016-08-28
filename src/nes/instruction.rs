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

        match len {
            1 => Instruction(raw_opcode, 0, 0),
            2 => Instruction(raw_opcode, memory.read_u8(pc + 1), 0),
            3 => Instruction(raw_opcode, memory.read_u8(pc + 1),
                             memory.read_u8(pc + 2)),
            _ => { panic!("Invalid instruction length returned") }
        }
    }

    /// Disassembles the instruction into human readable assembly.
    pub fn disassemble(&self, cpu: &CPU, memory: &mut Memory) -> String {
        use nes::opcode::opcode_len;

        let opcode = self.opcode();
        let len = opcode_len(&opcode);

        match opcode {
            BCSRel   => format!("BCS ${:04X}", cpu.pc + self.1 as u16 + len as u16),
            CLCImp   => format!("CLC"),
            CLDImp   => format!("CLD"),
            CLIImp   => format!("CLI"),
            CLVImp   => format!("CLV"),
            JMPAbs   => format!("JMP ${:02X}{:02X}", self.2, self.1),
            JMPInd   => format!("JMP (${:02X}{:02X})", self.2, self.1),
            JSRAbs   => format!("JSR ${:02X}{:02X}", self.2, self.1),
            LDXImm   => format!("LDX #${:02X}", self.1),
            LDXZero  => format!("LDX ${:02X}", self.1),
            LDXZeroY => format!("LDX ${:02X},Y", self.1),
            LDXAbs   => format!("LDX ${:02X}${:02X}", self.2, self.1),
            LDXAbsY  => format!("LDX ${:02X}${:02X},Y", self.2, self.1),
            NOPImp   => format!("NOP"),
            SECImp   => format!("SEC"),
            SEDImp   => format!("SED"),
            SEIImp   => format!("SEI"),
            STXZero  => format!("STX ${:02X} = {:02X}", self.1, cpu.x),
            STXZeroY => format!("STX ${:02X},Y = {:02X}", self.1, cpu.x),
            STXAbs   => format!("STX ${:02X}${:02X} = {:02X}", self.2, self.1, cpu.x),
            _ => { panic!("Unimplemented opcode found: {:?}", opcode); }
        }
    }

    /// Logs a human-readable representation of the instruction along with the
    /// CPU state in an easy to parse format.
    ///
    /// TODO: Return a string for the test suite so CPU correctness can be
    /// checked. Also it may be more appropriate to move this function into the
    /// CPU.
    pub fn log(&self, cpu: &CPU, memory: &mut Memory) {
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
        let disassembled = self.disassemble(cpu, memory);

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
            BCSRel => {
                if cpu.carry_flag_set() {
                    let old_pc = cpu.pc as usize;
                    cpu.pc = cpu.pc.wrapping_add(self.relative() as u16);
                    cpu.cycles += 1;
                    if page_cross(old_pc, cpu.pc as usize) != PageCross::Same {
                        cpu.cycles += 2;
                    }
                }
                cpu.cycles += 2;
                cpu.pc += len;
            },
            CLCImp => {
                cpu.unset_carry_flag();
                cpu.cycles += 2;
                cpu.pc += len;
            },
            CLDImp => {
                cpu.unset_decimal_mode();
                cpu.cycles += 2;
                cpu.pc += len;
            },
            CLIImp => {
                cpu.unset_interrupt_disable();
                cpu.cycles += 2;
                cpu.pc += len;
            },
            CLVImp => {
                cpu.unset_overflow_flag();
                cpu.cycles += 2;
                cpu.pc += len;
            },
            JMPAbs => {
                cpu.pc = self.absolute() as u16;
                cpu.cycles += 3;
            },
            JMPInd => {
                // A special version of indirect addressing is implemented here
                // due to a bug in the indirect JMP operation.
                // https://github.com/Reshurum/nes-rs/issues/3
                let arg = self.arg_u16() as usize;
                cpu.pc = memory.read_u16_wrapped_msb(arg);
                cpu.cycles += 5;
            },
            JSRAbs => {
                let addr = self.absolute() as u16;
                memory.stack_push_u16(cpu, addr - 1);
                cpu.pc = addr;
                cpu.cycles += 6;
            },
            LDXImm => {
                cpu.x = self.immediate();
                if cpu.x == 0 {
                    cpu.set_zero_flag();
                }
                if is_negative(cpu.x) {
                    cpu.set_negative_flag();
                }
                cpu.cycles += 2;
                cpu.pc += len;
            },
            LDXZero => {
                cpu.x = memory.read_u8(self.zero_page());
                if cpu.x == 0 {
                    cpu.set_zero_flag();
                }
                if is_negative(cpu.x) {
                    cpu.set_negative_flag();
                }
                cpu.cycles += 3;
                cpu.pc += len;
            },
            LDXZeroY => {
                cpu.x = memory.read_u8(self.zero_page_y(cpu));
                if cpu.x == 0 {
                    cpu.set_zero_flag();
                }
                if is_negative(cpu.x) {
                    cpu.set_negative_flag();
                }
                cpu.cycles += 4;
                cpu.pc += len;
            },
            LDXAbs => {
                cpu.x = memory.read_u8(self.absolute());
                if cpu.x == 0 {
                    cpu.set_zero_flag();
                }
                if is_negative(cpu.x) {
                    cpu.set_negative_flag();
                }
                cpu.cycles += 4;
                cpu.pc += len;
            },
            LDXAbsY => {
                let (addr, page_cross) = self.absolute_y(cpu);
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.x = memory.read_u8(addr);
                if cpu.x == 0 {
                    cpu.set_zero_flag();
                }
                if is_negative(cpu.x) {
                    cpu.set_negative_flag();
                }
                cpu.cycles += 4;
                cpu.pc += len;
            },
            NOPImp => {
                // This is the most difficult instruction to implement.
                cpu.cycles += 2;
                cpu.pc += len;
            },
            SECImp => {
                cpu.set_carry_flag();
                cpu.cycles += 2;
                cpu.pc += len;
            },
            SEDImp => {
                cpu.set_decimal_mode();
                cpu.cycles += 2;
                cpu.pc += len;
            },
            SEIImp => {
                cpu.set_interrupt_disable();
                cpu.cycles += 2;
                cpu.pc += len;
            },
            STXZero => {
                memory.write_u8(self.zero_page(), cpu.x);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            STXZeroY => {
                memory.write_u8(self.zero_page_y(cpu), cpu.x);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            STXAbs => {
                memory.write_u8(self.absolute(), cpu.x);
                cpu.cycles += 4;
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

    /// Dereferences a zero page address in the instruction.
    #[inline(always)]
    fn dereference_u8(&self, memory: &mut Memory) -> u8 {
        memory.read_u8(self.arg_u8() as usize)
    }

    /// Dereferences a memory address in the instruction.
    #[inline(always)]
    fn dereference_u16(&self, memory: &mut Memory) -> u8 {
        memory.read_u8(self.arg_u16() as usize)
    }

    // Addressing mode utility functions here. These are used to simplify the
    // development of instructions and a good explanation of addressing modes
    // can be found at https://skilldrick.github.io/easy6502/#addressing.

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
    fn zero_page(&self) -> usize {
        self.arg_u8() as usize
    }

    /// Returns a zero page address stored in the instruction with the X
    /// register added to it.
    #[inline(always)]
    fn zero_page_x(&self, cpu: &CPU) -> usize {
        self.arg_u8().wrapping_add(cpu.x) as usize
    }

    /// Returns a zero page address stored in the instruction with the Y
    /// register added to it.
    #[inline(always)]
    fn zero_page_y(&self, cpu: &CPU) -> usize {
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
    fn absolute_x(&self, cpu: &CPU) -> (usize, PageCross) {
        let base_addr = self.arg_u16();
        let addr = base_addr.wrapping_add(cpu.x as u16) as usize;
        let page_cross = page_cross(base_addr as usize, addr);
        (addr, page_cross)
    }

    /// Returns an address from the instruction argument with the value in the Y
    /// register added to it.
    #[inline(always)]
    fn absolute_y(&self, cpu: &CPU) -> (usize, PageCross) {
        let base_addr = self.arg_u16();
        let addr = base_addr.wrapping_add(cpu.y as u16) as usize;
        let page_cross = page_cross(base_addr as usize, addr);
        (addr, page_cross)
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

#[derive(PartialEq)]
enum PageCross {
    Same,
    Backwards,
    Forwards,
}

// Additional utility functions used often in instruction logic.

/// Checks if an unsigned number would be negative if it was signed. This is
/// done by checking if the 7th bit is set.
#[inline(always)]
fn is_negative(arg: u8) -> bool {
    let negative_bitmask = 0b10000000;
    arg & negative_bitmask == negative_bitmask
}

/// Returns the page index of the given address. Each memory page for the
/// 6502 is 256 (FF) bytes in size and is relevant because some instructions
/// need extra cycles to use addresses in different pages.
#[inline(always)]
fn page(addr: usize) -> u8 {
    (addr as u16 >> 8) as u8
}

/// Determine if there was a page cross between the addresses and what
/// direction was crossed. Most instructions don't care which direction the
/// page cross was in so those instructions will check for either forwards
/// or backwards.
#[inline(always)]
fn page_cross(addr1: usize, addr2: usize) -> PageCross {
    let page1 = page(addr1);
    let page2 = page(addr2);

    if page1 > page2 {
        PageCross::Backwards
    } else if page1 < page2 {
        PageCross::Forwards
    } else {
        PageCross::Same
    }
}
