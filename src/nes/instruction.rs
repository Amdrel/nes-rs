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
use nes::opcode::{Opcode, opcode_len, decode_opcode};
use std::io::Cursor;
use utils::paging::{PageCross, page_cross};
use utils::arithmetic::{add_relative};

/// All 6502 instructions are a maximum size of 3 bytes. The first byte is the
/// opcode which is determines the action of the instruction. The following 2
/// bytes are the arguments and are present depending on the opcode.
#[derive(Debug, PartialEq)]
pub struct Instruction(pub u8, pub u8, pub u8);

impl Instruction {
    /// Parses an instruction from memory at the address of the passed program
    /// counter. Some instructions when parsed by the original 6502 will read
    /// arguments from the wrong addresses (e.g indirect JMP), so those bugs are
    /// emulated accurately here.
    pub fn parse(pc: usize, memory: &mut Memory) -> Instruction {
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
        let opcode = self.opcode();
        let len = opcode_len(&opcode);

        match opcode {
            ANDImm   => format!("AND #${:02X}", self.1),
            ANDZero  => format!("AND ${:02X}", self.1),
            ANDZeroX => format!("AND ${:02X},X", self.1),
            ANDAbs   => format!("AND ${:02X}{:02X}", self.2, self.1),
            ANDAbsX  => format!("AND ${:02X}{:02X},X", self.2, self.1),
            ANDAbsY  => format!("AND ${:02X}{:02X},Y", self.2, self.1),
            ANDIndX  => format!("AND (${:02X},X)", self.1),
            ANDIndY  => format!("AND (${:02X}),Y", self.1),
            BCCRel   => format!("BCC ${:04X}", add_relative(cpu.pc, self.relative()) + len as u16),
            BCSRel   => format!("BCS ${:04X}", add_relative(cpu.pc, self.relative()) + len as u16),
            BEQRel   => format!("BEQ ${:04X}", add_relative(cpu.pc, self.relative()) + len as u16),
            BMIRel   => format!("BMI ${:04X}", add_relative(cpu.pc, self.relative()) + len as u16),
            EORImm   => format!("EOR #${:02X}", self.1),
            EORZero  => format!("EOR ${:02X}", self.1),
            EORZeroX => format!("EOR ${:02X},X", self.1),
            EORAbs   => format!("EOR ${:02X}{:02X}", self.2, self.1),
            EORAbsX  => format!("EOR ${:02X}{:02X},X", self.2, self.1),
            EORAbsY  => format!("EOR ${:02X}{:02X},Y", self.2, self.1),
            EORIndX  => format!("EOR (${:02X},X)", self.1),
            EORIndY  => format!("EOR (${:02X}),Y", self.1),
            ORAImm   => format!("ORA #${:02X}", self.1),
            ORAZero  => format!("ORA ${:02X}", self.1),
            ORAZeroX => format!("ORA ${:02X},X", self.1),
            ORAAbs   => format!("ORA ${:02X}{:02X}", self.2, self.1),
            ORAAbsX  => format!("ORA ${:02X}{:02X},X", self.2, self.1),
            ORAAbsY  => format!("ORA ${:02X}{:02X},Y", self.2, self.1),
            ORAIndX  => format!("ORA (${:02X},X)", self.1),
            ORAIndY  => format!("ORA (${:02X}),Y", self.1),
            BITZero  => format!("BIT ${:02X} = {:02X}", self.1, self.dereference_zero_page(memory)),
            BITAbs   => format!("BIT ${:02X}{:02X} = {:02X}", self.2, self.1, self.dereference_absolute(memory)),
            BNERel   => format!("BNE ${:04X}", add_relative(cpu.pc, self.relative()) + len as u16),
            BPLRel   => format!("BPL ${:04X}", add_relative(cpu.pc, self.relative()) + len as u16),
            BVCRel   => format!("BVC ${:04X}", add_relative(cpu.pc, self.relative()) + len as u16),
            BVSRel   => format!("BVS ${:04X}", add_relative(cpu.pc, self.relative()) + len as u16),
            CLCImp   => format!("CLC"),
            CLDImp   => format!("CLD"),
            CLIImp   => format!("CLI"),
            CLVImp   => format!("CLV"),
            ADCImm   => format!("ADC #${:02X}", self.1),
            ADCZero  => format!("ADC ${:02X}", self.1),
            ADCZeroX => format!("ADC ${:02X},X", self.1),
            ADCAbs   => format!("ADC ${:02X}{:02X}", self.2, self.1),
            ADCAbsX  => format!("ADC ${:02X}{:02X},X", self.2, self.1),
            ADCAbsY  => format!("ADC ${:02X}{:02X},Y", self.2, self.1),
            ADCIndX  => format!("ADC (${:02X},X)", self.1),
            ADCIndY  => format!("ADC (${:02X}),Y", self.1),
            SBCImm   => format!("SBC #${:02X}", self.1),
            SBCZero  => format!("SBC ${:02X}", self.1),
            SBCZeroX => format!("SBC ${:02X},X", self.1),
            SBCAbs   => format!("SBC ${:02X}{:02X}", self.2, self.1),
            SBCAbsX  => format!("SBC ${:02X}{:02X},X", self.2, self.1),
            SBCAbsY  => format!("SBC ${:02X}{:02X},Y", self.2, self.1),
            SBCIndX  => format!("SBC (${:02X},X)", self.1),
            SBCIndY  => format!("SBC (${:02X}),Y", self.1),
            CMPImm   => format!("CMP #${:02X}", self.1),
            CMPZero  => format!("CMP ${:02X}", self.1),
            CMPZeroX => format!("CMP ${:02X},X", self.1),
            CMPAbs   => format!("CMP ${:02X}{:02X}", self.2, self.1),
            CMPAbsX  => format!("CMP ${:02X}{:02X},X", self.2, self.1),
            CMPAbsY  => format!("CMP ${:02X}{:02X},Y", self.2, self.1),
            CMPIndX  => format!("CMP (${:02X},X)", self.1),
            CMPIndY  => format!("CMP (${:02X}),Y", self.1),
            CPXImm   => format!("CPX #${:02X}", self.1),
            CPXZero  => format!("CPX ${:02X}", self.1),
            CPXAbs   => format!("CPX ${:02X}{:02X}", self.2, self.1),
            CPYImm   => format!("CPY #${:02X}", self.1),
            CPYZero  => format!("CPY ${:02X}", self.1),
            CPYAbs   => format!("CPY ${:02X}{:02X}", self.2, self.1),
            INXImp   => format!("INX"),
            INYImp   => format!("INY"),
            DEXImp   => format!("DEX"),
            DEYImp   => format!("DEY"),
            JMPAbs   => format!("JMP ${:02X}{:02X}", self.2, self.1),
            JMPInd   => format!("JMP (${:02X}{:02X})", self.2, self.1),
            JSRAbs   => format!("JSR ${:02X}{:02X}", self.2, self.1),
            LDAImm   => format!("LDA #${:02X}", self.1),
            LDAZero  => format!("LDA ${:02X}", self.1),
            LDAZeroX => format!("LDA ${:02X},X", self.1),
            LDAAbs   => format!("LDA ${:02X}{:02X}", self.2, self.1),
            LDAAbsX  => format!("LDA ${:02X}{:02X},X", self.2, self.1),
            LDAAbsY  => format!("LDA ${:02X}{:02X},Y", self.2, self.1),
            LDAIndX  => format!("LDA (${:02X},X)", self.1),
            LDAIndY  => format!("LDA (${:02X}),Y", self.1),
            LDXImm   => format!("LDX #${:02X}", self.1),
            LDXZero  => format!("LDX ${:02X}", self.1),
            LDXZeroY => format!("LDX ${:02X},Y", self.1),
            LDXAbs   => format!("LDX ${:02X}{:02X} = {:02X}", self.2, self.1, self.dereference_absolute(memory)),
            LDXAbsY  => format!("LDX ${:02X}{:02X},Y", self.2, self.1),
            LDYImm   => format!("LDY #${:02X}", self.1),
            LDYZero  => format!("LDY ${:02X}", self.1),
            LDYZeroX => format!("LDY ${:02X},X", self.1),
            LDYAbs   => format!("LDY ${:02X}{:02X}", self.2, self.1),
            LDYAbsX  => format!("LDY ${:02X}{:02X},X", self.2, self.1),
            NOPImp   => format!("NOP"),
            PHAImp   => format!("PHA"),
            PHPImp   => format!("PHP"),
            PLAImp   => format!("PLA"),
            PLPImp   => format!("PLP"),
            RTSImp   => format!("RTS"),
            SECImp   => format!("SEC"),
            SEDImp   => format!("SED"),
            SEIImp   => format!("SEI"),
            STAZero  => format!("STA ${:02X} = {:02X}", self.1, self.dereference_zero_page(memory)),
            STAZeroX => format!("STA ${:02X},X = {:02X}", self.1, self.dereference_zero_page_x(memory, cpu)),
            STAAbs   => format!("STA ${:02X}{:02X} = {:02X}", self.2, self.1, self.dereference_absolute(memory)),
            STAAbsX  => format!("STA ${:02X}{:02X},X = {:02X}", self.2, self.1, self.dereference_absolute_x(memory, cpu)),
            STAAbsY  => format!("STA ${:02X}{:02X},Y = {:02X}", self.2, self.1, self.dereference_absolute_y(memory, cpu)),
            STAIndX  => format!("STA (${:02X},X) = {:02X}", self.1, self.dereference_indirect_x(memory, cpu)),
            STAIndY  => format!("STA (${:02X}),Y = {:02X}", self.1, self.dereference_indirect_y(memory, cpu)),
            STXZero  => format!("STX ${:02X} = {:02X}", self.1, self.dereference_zero_page(memory)),
            STXZeroY => format!("STX ${:02X},Y = {:02X}", self.1, self.dereference_zero_page_y(memory, cpu)),
            STXAbs   => format!("STX ${:02X}{:02X} = {:02X}", self.2, self.1, self.dereference_absolute(memory)),
            TAXImp   => format!("TAX"),
            TAYImp   => format!("TAY"),
            TSXImp   => format!("TSX"),
            TXAImp   => format!("TXA"),
            TXSImp   => format!("TXS"),
            TYAImp   => format!("TYA"),
            _ => { panic!("Unimplemented opcode found: {:?}", opcode); }
        }
    }

    /// Logs a human-readable representation of the instruction along with the
    /// CPU state in an easy to parse format.
    ///
    /// TODO: Return a string for the test suite so CPU correctness can be
    /// checked. Also it may be more appropriate to move this function into the
    /// CPU.
    pub fn log(&self, cpu: &CPU, memory: &mut Memory) -> String {
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
        //       0       6   16     48       53       58       63       68        74
        format!("{:04X}  {}  {:30}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:3}",
                 cpu.pc, instr_str, disassembled, cpu.a, cpu.x, cpu.y, cpu.p,
                 cpu.sp, 0)
    }

    /// Execute the instruction with a routine that corresponds with it's
    /// opcode. All routines for every instruction in the 6502 instruction set
    /// are present here.
    #[inline(always)]
    pub fn execute(&self, cpu: &mut CPU, memory: &mut Memory) {
        let opcode = self.opcode();
        let len = opcode_len(&opcode) as u16;

        // Execute the internal logic of the instruction based on it's opcode.
        match opcode {
            ANDImm => {
                cpu.a &= self.immediate();
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            ANDZero => {
                cpu.a &= self.dereference_zero_page(memory);
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            ANDZeroX => {
                cpu.a &= self.dereference_zero_page_x(memory, cpu);
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            ANDAbs => {
                cpu.a &= self.dereference_absolute(memory);
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            ANDAbsX => {
                let (addr, page_cross) = self.absolute_x(cpu);
                cpu.a &= memory.read_u8(addr);
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                cpu.cycles += 4;
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.pc += len;
            },
            ANDAbsY => {
                let (addr, page_cross) = self.absolute_y(cpu);
                cpu.a &= memory.read_u8(addr);
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                cpu.cycles += 4;
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.pc += len;
            },
            ANDIndX => {
                cpu.a &= self.dereference_indirect_x(memory, cpu);
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            ANDIndY => {
                let (addr, page_cross) = self.indirect_y(cpu, memory);
                cpu.a &= memory.read_u8(addr);
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                cpu.cycles += 5;
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.pc += len;
            },
            BCCRel => {
                if !cpu.carry_flag_set() {
                    let old_pc = cpu.pc as usize;
                    cpu.pc = add_relative(cpu.pc, self.relative());
                    cpu.cycles += 1;
                    if page_cross(old_pc, cpu.pc as usize) != PageCross::Same {
                        cpu.cycles += 2;
                    }
                }
                cpu.cycles += 2;
                cpu.pc += len;
            },
            BCSRel => {
                if cpu.carry_flag_set() {
                    let old_pc = cpu.pc as usize;
                    cpu.pc = add_relative(cpu.pc, self.relative());
                    cpu.cycles += 1;
                    if page_cross(old_pc, cpu.pc as usize) != PageCross::Same {
                        cpu.cycles += 2;
                    }
                }
                cpu.cycles += 2;
                cpu.pc += len;
            },
            BEQRel => {
                if cpu.zero_flag_set() {
                    let old_pc = cpu.pc as usize;
                    cpu.pc = add_relative(cpu.pc, self.relative());
                    cpu.cycles += 1;
                    if page_cross(old_pc, cpu.pc as usize) != PageCross::Same {
                        cpu.cycles += 2;
                    }
                }
                cpu.cycles += 2;
                cpu.pc += len;
            },
            BMIRel => {
                if cpu.negative_flag_set() {
                    let old_pc = cpu.pc as usize;
                    cpu.pc = add_relative(cpu.pc, self.relative());
                    cpu.cycles += 1;
                    if page_cross(old_pc, cpu.pc as usize) != PageCross::Same {
                        cpu.cycles += 2;
                    }
                }
                cpu.cycles += 2;
                cpu.pc += len;
            },
            EORImm => {
                let result = cpu.a ^ self.immediate();
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            EORZero => {
                let result = cpu.a ^ self.dereference_zero_page(memory);
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            EORZeroX => {
                let result = cpu.a ^ self.dereference_zero_page_x(memory, cpu);
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            EORAbs => {
                let result = cpu.a ^ self.dereference_absolute(memory);
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            EORAbsX => {
                let (addr, page_cross) = self.absolute_x(cpu);
                let result = cpu.a ^ memory.read_u8(addr);
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.pc += len;
            },
            EORAbsY => {
                let (addr, page_cross) = self.absolute_y(cpu);
                let result = cpu.a ^ memory.read_u8(addr);
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.pc += len;
            },
            EORIndX => {
                let result = cpu.a ^ self.dereference_indirect_x(memory, cpu);
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            EORIndY => {
                let (addr, page_cross) = self.indirect_y(cpu, memory);
                let result = cpu.a ^ memory.read_u8(addr);
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 5;
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.pc += len;
            },
            ORAImm => {
                let result = cpu.a | self.immediate();
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            ORAZero => {
                let result = cpu.a | self.dereference_zero_page(memory);
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            ORAZeroX => {
                let result = cpu.a | self.dereference_zero_page_x(memory, cpu);
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            ORAAbs => {
                let result = cpu.a | self.dereference_absolute(memory);
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            ORAAbsX => {
                let (addr, page_cross) = self.absolute_x(cpu);
                let result = cpu.a | memory.read_u8(addr);
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.pc += len;
            },
            ORAAbsY => {
                let (addr, page_cross) = self.absolute_y(cpu);
                let result = cpu.a | memory.read_u8(addr);
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.pc += len;
            },
            ORAIndX => {
                let result = cpu.a | self.dereference_indirect_x(memory, cpu);
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            ORAIndY => {
                let (addr, page_cross) = self.indirect_y(cpu, memory);
                let result = cpu.a | memory.read_u8(addr);
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 5;
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.pc += len;
            },
            BITZero => {
                let byte = self.dereference_zero_page(memory);
                let result = byte & cpu.a;
                cpu.toggle_zero_flag(result);
                let mask = 0xC0;
                cpu.p = (cpu.p & !mask) | (byte & mask);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            BITAbs => {
                let byte = self.dereference_zero_page(memory);
                let result = byte & cpu.a;
                cpu.toggle_zero_flag(result);
                let mask = 0xC0;
                cpu.p = (cpu.p & !mask) | (byte & mask);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            BNERel => {
                if !cpu.zero_flag_set() {
                    let old_pc = cpu.pc as usize;
                    cpu.pc = add_relative(cpu.pc, self.relative());
                    cpu.cycles += 1;
                    if page_cross(old_pc, cpu.pc as usize) != PageCross::Same {
                        cpu.cycles += 2;
                    }
                }
                cpu.cycles += 2;
                cpu.pc += len;
            },
            BPLRel => {
                if !cpu.negative_flag_set() {
                    let old_pc = cpu.pc as usize;
                    cpu.pc = add_relative(cpu.pc, self.relative());
                    cpu.cycles += 1;
                    if page_cross(old_pc, cpu.pc as usize) != PageCross::Same {
                        cpu.cycles += 2;
                    }
                }
                cpu.cycles += 2;
                cpu.pc += len;
            },
            BVCRel => {
                if !cpu.overflow_flag_set() {
                    let old_pc = cpu.pc as usize;
                    cpu.pc = add_relative(cpu.pc, self.relative());
                    cpu.cycles += 1;
                    if page_cross(old_pc, cpu.pc as usize) != PageCross::Same {
                        cpu.cycles += 2;
                    }
                }
                cpu.cycles += 2;
                cpu.pc += len;
            },
            BVSRel => {
                if cpu.overflow_flag_set() {
                    let old_pc = cpu.pc as usize;
                    cpu.pc = add_relative(cpu.pc, self.relative());
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
            ADCImm => {
                let arg = self.immediate();
                let (result, overflow);
                if cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_add(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_add(arg);
                    result = r;
                    overflow = o;
                }
                if !(cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            ADCZero => {
                let arg = self.dereference_zero_page(memory);
                let (result, overflow);
                if cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_add(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_add(arg);
                    result = r;
                    overflow = o;
                }
                if !(cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            ADCZeroX => {
                let arg = self.dereference_zero_page_x(memory, cpu);
                let (result, overflow);
                if cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_add(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_add(arg);
                    result = r;
                    overflow = o;
                }
                if !(cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            ADCAbs => {
                let arg = self.dereference_absolute(memory);
                let (result, overflow);
                if cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_add(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_add(arg);
                    result = r;
                    overflow = o;
                }
                if !(cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            ADCAbsX => {
                let (addr, page_cross) = self.absolute_x(cpu);
                let arg = memory.read_u8(addr);
                let (result, overflow);
                if cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_add(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_add(arg);
                    result = r;
                    overflow = o;
                }
                if !(cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.cycles += 4;
                cpu.pc += len;
            },
            ADCAbsY => {
                let (addr, page_cross) = self.absolute_y(cpu);
                let arg = memory.read_u8(addr);
                let (result, overflow);
                if cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_add(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_add(arg);
                    result = r;
                    overflow = o;
                }
                if !(cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.cycles += 4;
                cpu.pc += len;
            },
            ADCIndX => {
                let arg = self.dereference_indirect_x(memory, cpu);
                let (result, overflow);
                if cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_add(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_add(arg);
                    result = r;
                    overflow = o;
                }
                if !(cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            ADCIndY => {
                let (addr, page_cross) = self.indirect_y(cpu, memory);
                let arg = memory.read_u8(addr);
                let (result, overflow);
                if cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_add(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_add(arg);
                    result = r;
                    overflow = o;
                }
                if !(cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.cycles += 5;
                cpu.pc += len;
            },
            SBCImm => {
                let arg = self.immediate();
                let (result, overflow);
                if !cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_sub(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_sub(arg);
                    result = r;
                    overflow = o;
                }
                if (cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(!overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            SBCZero => {
                let arg = self.dereference_zero_page(memory);
                let (result, overflow);
                if !cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_sub(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_sub(arg);
                    result = r;
                    overflow = o;
                }
                if (cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(!overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            SBCZeroX => {
                let arg = self.dereference_zero_page_x(memory, cpu);
                let (result, overflow);
                if !cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_sub(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_sub(arg);
                    result = r;
                    overflow = o;
                }
                if (cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(!overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            SBCAbs => {
                let arg = self.dereference_absolute(memory);
                let (result, overflow);
                if !cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_sub(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_sub(arg);
                    result = r;
                    overflow = o;
                }
                if (cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(!overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            SBCAbsX => {
                let (addr, page_cross) = self.absolute_x(cpu);
                let arg = memory.read_u8(addr);
                let (result, overflow);
                if !cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_sub(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_sub(arg);
                    result = r;
                    overflow = o;
                }
                if (cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(!overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.cycles += 4;
                cpu.pc += len;
            },
            SBCAbsY => {
                let (addr, page_cross) = self.absolute_y(cpu);
                let arg = memory.read_u8(addr);
                let (result, overflow);
                if !cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_sub(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_sub(arg);
                    result = r;
                    overflow = o;
                }
                if (cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(!overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.cycles += 4;
                cpu.pc += len;
            },
            SBCIndX => {
                let arg = self.dereference_indirect_x(memory, cpu);
                let (result, overflow);
                if !cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_sub(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_sub(arg);
                    result = r;
                    overflow = o;
                }
                if (cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(!overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            SBCIndY => {
                let (addr, page_cross) = self.indirect_y(cpu, memory);
                let arg = memory.read_u8(addr);
                let (result, overflow);
                if !cpu.carry_flag_set() {
                    let (r, o) = cpu.a.overflowing_sub(arg.wrapping_add(1));
                    result = r;
                    overflow = o;
                } else {
                    let (r, o) = cpu.a.overflowing_sub(arg);
                    result = r;
                    overflow = o;
                }
                if (cpu.a ^ arg) & (cpu.a ^ result) & 0x80 == 0x80 {
                    cpu.set_overflow_flag();
                } else {
                    cpu.unset_overflow_flag();
                }
                cpu.a = result;
                cpu.toggle_carry_flag(!overflow);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.cycles += 5;
                cpu.pc += len;
            },
            CMPImm => {
                let arg = self.immediate();
                let result = cpu.a.wrapping_sub(arg);
                if cpu.a >= arg {
                    cpu.set_carry_flag();
                } else {
                    cpu.unset_carry_flag()
                }
                if result == 0 {
                    cpu.set_zero_flag();
                } else {
                    cpu.unset_zero_flag();
                }
                cpu.toggle_negative_flag(result);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            CMPZero => {
                let arg = self.dereference_zero_page(memory);
                let result = cpu.a.wrapping_sub(arg);
                if cpu.a >= arg {
                    cpu.set_carry_flag();
                } else {
                    cpu.unset_carry_flag()
                }
                if result == 0 {
                    cpu.set_zero_flag();
                } else {
                    cpu.unset_zero_flag();
                }
                cpu.toggle_negative_flag(result);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            CMPZeroX => {
                let arg = self.dereference_zero_page_x(memory, cpu);
                let result = cpu.a.wrapping_sub(arg);
                if cpu.a >= arg {
                    cpu.set_carry_flag();
                } else {
                    cpu.unset_carry_flag()
                }
                if result == 0 {
                    cpu.set_zero_flag();
                } else {
                    cpu.unset_zero_flag();
                }
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            CMPAbs => {
                let arg = self.dereference_absolute(memory);
                let result = cpu.a.wrapping_sub(arg);
                if cpu.a >= arg {
                    cpu.set_carry_flag();
                } else {
                    cpu.unset_carry_flag()
                }
                if result == 0 {
                    cpu.set_zero_flag();
                } else {
                    cpu.unset_zero_flag();
                }
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            CMPAbsX => {
                let (addr, page_cross) = self.absolute_x(cpu);
                let arg = memory.read_u8(addr);
                let result = cpu.a.wrapping_sub(arg);
                if cpu.a >= arg {
                    cpu.set_carry_flag();
                } else {
                    cpu.unset_carry_flag()
                }
                if result == 0 {
                    cpu.set_zero_flag();
                } else {
                    cpu.unset_zero_flag();
                }
                cpu.toggle_negative_flag(result);
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.cycles += 4;
                cpu.pc += len;
            },
            CMPAbsY => {
                let (addr, page_cross) = self.absolute_y(cpu);
                let arg = memory.read_u8(addr);
                let result = cpu.a.wrapping_sub(arg);
                if cpu.a >= arg {
                    cpu.set_carry_flag();
                } else {
                    cpu.unset_carry_flag()
                }
                if result == 0 {
                    cpu.set_zero_flag();
                } else {
                    cpu.unset_zero_flag();
                }
                cpu.toggle_negative_flag(result);
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.cycles += 4;
                cpu.pc += len;
            },
            CMPIndX => {
                let arg = self.dereference_indirect_x(memory, cpu);
                let result = cpu.a.wrapping_sub(arg);
                if cpu.a >= arg {
                    cpu.set_carry_flag();
                } else {
                    cpu.unset_carry_flag()
                }
                if result == 0 {
                    cpu.set_zero_flag();
                } else {
                    cpu.unset_zero_flag();
                }
                cpu.toggle_negative_flag(result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            CMPIndY => {
                let (addr, page_cross) = self.indirect_y(cpu, memory);
                let arg = memory.read_u8(addr);
                let result = cpu.a.wrapping_sub(arg);
                if cpu.a >= arg {
                    cpu.set_carry_flag();
                } else {
                    cpu.unset_carry_flag()
                }
                if result == 0 {
                    cpu.set_zero_flag();
                } else {
                    cpu.unset_zero_flag();
                }
                cpu.toggle_negative_flag(result);
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.cycles += 5;
                cpu.pc += len;
            },
            CPXImm => {
                let arg = self.immediate();
                let result = cpu.x.wrapping_sub(arg);
                if cpu.x >= arg {
                    cpu.set_carry_flag();
                } else {
                    cpu.unset_carry_flag()
                }
                if result == 0 {
                    cpu.set_zero_flag();
                } else {
                    cpu.unset_zero_flag();
                }
                cpu.toggle_negative_flag(result);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            CPXZero => {
                let arg = self.dereference_zero_page(memory);
                let result = cpu.x.wrapping_sub(arg);
                if cpu.x >= arg {
                    cpu.set_carry_flag();
                } else {
                    cpu.unset_carry_flag()
                }
                if result == 0 {
                    cpu.set_zero_flag();
                } else {
                    cpu.unset_zero_flag();
                }
                cpu.toggle_negative_flag(result);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            CPXAbs => {
                let arg = self.dereference_absolute(memory);
                let result = cpu.x.wrapping_sub(arg);
                if cpu.x >= arg {
                    cpu.set_carry_flag();
                } else {
                    cpu.unset_carry_flag()
                }
                if result == 0 {
                    cpu.set_zero_flag();
                } else {
                    cpu.unset_zero_flag();
                }
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            CPYImm => {
                let arg = self.immediate();
                let result = cpu.y.wrapping_sub(arg);
                if cpu.y >= arg {
                    cpu.set_carry_flag();
                } else {
                    cpu.unset_carry_flag()
                }
                if result == 0 {
                    cpu.set_zero_flag();
                } else {
                    cpu.unset_zero_flag();
                }
                cpu.toggle_negative_flag(result);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            CPYZero => {
                let arg = self.dereference_zero_page(memory);
                let result = cpu.y.wrapping_sub(arg);
                if cpu.y >= arg {
                    cpu.set_carry_flag();
                } else {
                    cpu.unset_carry_flag()
                }
                if result == 0 {
                    cpu.set_zero_flag();
                } else {
                    cpu.unset_zero_flag();
                }
                cpu.toggle_negative_flag(result);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            CPYAbs => {
                let arg = self.dereference_absolute(memory);
                let result = cpu.y.wrapping_sub(arg);
                if cpu.y >= arg {
                    cpu.set_carry_flag();
                } else {
                    cpu.unset_carry_flag()
                }
                if result == 0 {
                    cpu.set_zero_flag();
                } else {
                    cpu.unset_zero_flag();
                }
                cpu.toggle_negative_flag(result);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            INXImp => {
                let result = cpu.x.wrapping_add(1);
                cpu.x = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            INYImp => {
                let result = cpu.y.wrapping_add(1);
                cpu.y = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            DEXImp => {
                let result = cpu.x.wrapping_sub(1);
                cpu.x = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            DEYImp => {
                let result = cpu.y.wrapping_sub(1);
                cpu.y = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
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
                let pc = cpu.pc;
                memory.stack_push_u16(cpu, pc + len - 1);
                cpu.pc = self.absolute() as u16;
                cpu.cycles += 6;
            },
            LDAImm => {
                cpu.a = self.immediate();
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            LDAZero => {
                cpu.a = memory.read_u8(self.zero_page());
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            LDAZeroX => {
                cpu.a = memory.read_u8(self.zero_page_x(cpu));
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            LDAAbs => {
                cpu.a = memory.read_u8(self.absolute());
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            LDAAbsX => {
                let (addr, page_cross) = self.absolute_x(cpu);
                cpu.a = memory.read_u8(addr);
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.cycles += 4;
                cpu.pc += len;
            },
            LDAAbsY => {
                let (addr, page_cross) = self.absolute_y(cpu);
                cpu.a = memory.read_u8(addr);
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.cycles += 4;
                cpu.pc += len;
            },
            LDAIndX => {
                let (addr, _) = self.indirect_x(cpu, memory);
                cpu.a = memory.read_u8(addr);
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            LDAIndY => {
                let (addr, page_cross) = self.indirect_y(cpu, memory);
                cpu.a = memory.read_u8(addr);
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.cycles += 5;
                cpu.pc += len;
            },
            LDXImm => {
                cpu.x = self.immediate();
                let x = cpu.x;
                cpu.toggle_zero_flag(x);
                cpu.toggle_negative_flag(x);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            LDXZero => {
                cpu.x = memory.read_u8(self.zero_page());
                let x = cpu.x;
                cpu.toggle_zero_flag(x);
                cpu.toggle_negative_flag(x);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            LDXZeroY => {
                cpu.x = memory.read_u8(self.zero_page_y(cpu));
                let x = cpu.x;
                cpu.toggle_zero_flag(x);
                cpu.toggle_negative_flag(x);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            LDXAbs => {
                cpu.x = memory.read_u8(self.absolute());
                let x = cpu.x;
                cpu.toggle_zero_flag(x);
                cpu.toggle_negative_flag(x);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            LDXAbsY => {
                let (addr, page_cross) = self.absolute_y(cpu);
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.x = memory.read_u8(addr);
                let x = cpu.x;
                cpu.toggle_zero_flag(x);
                cpu.toggle_negative_flag(x);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            LDYImm => {
                cpu.y = self.immediate();
                let y = cpu.y;
                cpu.toggle_zero_flag(y);
                cpu.toggle_negative_flag(y);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            LDYZero => {
                cpu.y = self.dereference_zero_page(memory);
                let y = cpu.y;
                cpu.toggle_zero_flag(y);
                cpu.toggle_negative_flag(y);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            LDYZeroX => {
                cpu.y = self.dereference_zero_page_x(memory, cpu);
                let y = cpu.y;
                cpu.toggle_zero_flag(y);
                cpu.toggle_negative_flag(y);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            LDYAbs => {
                cpu.y = self.dereference_absolute(memory);
                let y = cpu.y;
                cpu.toggle_zero_flag(y);
                cpu.toggle_negative_flag(y);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            LDYAbsX => {
                let (addr, page_cross) = self.absolute_x(cpu);
                cpu.y = memory.read_u8(addr);
                let y = cpu.y;
                cpu.toggle_zero_flag(y);
                cpu.toggle_negative_flag(y);
                if page_cross != PageCross::Same {
                    cpu.cycles += 1;
                }
                cpu.cycles += 4;
                cpu.pc += len;
            },
            NOPImp => {
                // This is the most difficult instruction to implement.
                cpu.cycles += 2;
                cpu.pc += len;
            },
            PHAImp => {
                let a = cpu.a;
                memory.stack_push_u8(cpu, a);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            PHPImp => {
                let p = cpu.p | 0x10; // Ensure bit 5 is always set.
                memory.stack_push_u8(cpu, p);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            PLAImp => {
                cpu.a = memory.stack_pop_u8(cpu);
                let a = cpu.a;
                cpu.toggle_zero_flag(a);
                cpu.toggle_negative_flag(a);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            PLPImp => {
                let old_flags = cpu.p;
                let p = (memory.stack_pop_u8(cpu) & 0xEF) | (old_flags & 0x20);
                cpu.p = p;
                cpu.cycles += 4;
                cpu.pc += len;
            },
            RTSImp => {
                cpu.pc = memory.stack_pop_u16(cpu) + len;
                cpu.cycles += 6;
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
            STAZero => {
                memory.write_u8(self.zero_page(), cpu.a);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            STAZeroX => {
                memory.write_u8(self.zero_page_x(cpu), cpu.a);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            STAAbs => {
                memory.write_u8(self.absolute(), cpu.a);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            STAAbsX => {
                memory.write_u8(self.absolute_x(cpu).0, cpu.a);
                cpu.cycles += 5;
                cpu.pc += len;
            },
            STAAbsY => {
                memory.write_u8(self.absolute_y(cpu).0, cpu.a);
                cpu.cycles += 5;
                cpu.pc += len;
            },
            STAIndX => {
                let addr = self.indirect_x(cpu, memory).0;
                memory.write_u8(addr, cpu.a);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            STAIndY => {
                let addr = self.indirect_y(cpu, memory).0;
                memory.write_u8(addr, cpu.a);
                cpu.cycles += 6;
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
            TAXImp => {
                let result = cpu.a;
                cpu.x = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            TAYImp => {
                let result = cpu.a;
                cpu.y = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            TSXImp => {
                let result = cpu.sp;
                cpu.x = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            TXAImp => {
                let result = cpu.x;
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 2;
                cpu.pc += len;
            },
            TXSImp => {
                let result = cpu.x;
                cpu.sp = result;
                cpu.cycles += 2;
                cpu.pc += len;
            },
            TYAImp => {
                let result = cpu.y;
                cpu.a = result;
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
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
    ///
    /// TODO: Remove page cross detection as indirect x never has a page
    /// crossing penalty.
    #[inline(always)]
    fn indirect_x(&self, cpu: &CPU, memory: &mut Memory) -> (usize, PageCross) {
        let arg = self.arg_u8();
        let addr = arg.wrapping_add(cpu.x) as usize;
        let page_cross = page_cross(arg as usize, addr);
        (memory.read_u16(addr) as usize, page_cross)
    }

    /// Sane version of indirect_x that gets the zero page address in the
    /// instruction, adds Y to it, then returns the resulting address.
    #[inline(always)]
    fn indirect_y(&self, cpu: &CPU, memory: &mut Memory) -> (usize, PageCross) {
        let arg = self.arg_u8() as usize;
        let base_addr = memory.read_u16(arg);
        let addr = base_addr.wrapping_add(cpu.y as u16) as usize;
        let page_cross = page_cross(base_addr as usize, addr);
        (addr, page_cross)
    }

    /// Dereferences a zero page address.
    #[inline(always)]
    fn dereference_zero_page(&self, memory: &mut Memory) -> u8 {
        let addr = self.zero_page();
        memory.read_u8(addr)
    }

    /// Dereferences a zero page x address.
    #[inline(always)]
    fn dereference_zero_page_x(&self, memory: &mut Memory, cpu: &CPU) -> u8 {
        let addr = self.zero_page_x(cpu);
        memory.read_u8(addr)
    }

    /// Dereferences a zero page y address.
    #[inline(always)]
    fn dereference_zero_page_y(&self, memory: &mut Memory, cpu: &CPU) -> u8 {
        let addr = self.zero_page_y(cpu);
        memory.read_u8(addr)
    }

    /// Dereferences an absolute address.
    #[inline(always)]
    fn dereference_absolute(&self, memory: &mut Memory) -> u8 {
        let addr = self.absolute();
        memory.read_u8(addr)
    }

    /// Dereferences an absolute x address.
    #[inline(always)]
    fn dereference_absolute_x(&self, memory: &mut Memory, cpu: &CPU) -> u8 {
        let addr = self.absolute_x(cpu).0;
        memory.read_u8(addr)
    }

    /// Dereferences an absolute y address.
    #[inline(always)]
    fn dereference_absolute_y(&self, memory: &mut Memory, cpu: &CPU) -> u8 {
        let addr = self.absolute_y(cpu).0;
        memory.read_u8(addr)
    }

    /// Dereferences an indirect address.
    #[inline(always)]
    fn dereference_indirect(&self, memory: &mut Memory) -> u8 {
        let addr = self.indirect(memory);
        memory.read_u8(addr)
    }

    /// Dereferences an indirect x address.
    #[inline(always)]
    fn dereference_indirect_x(&self, memory: &mut Memory, cpu: &CPU) -> u8 {
        let addr = self.indirect_x(cpu, memory).0;
        memory.read_u8(addr)
    }

    /// Dereferences an indirect y address.
    #[inline(always)]
    fn dereference_indirect_y(&self, memory: &mut Memory, cpu: &CPU) -> u8 {
        let addr = self.indirect_y(cpu, memory).0;
        memory.read_u8(addr)
    }
}
