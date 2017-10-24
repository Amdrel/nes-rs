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
use utils::arithmetic::{add_relative};
use utils::paging::{PageCross, page_cross};

/// All 6502 instructions are a maximum size of 3 bytes. The first byte is the
/// opcode which is determines the action of the instruction. The following 2
/// bytes are the arguments and are present depending on the opcode.
#[derive(Debug, PartialEq)]
pub struct Instruction(pub u8, pub u8, pub u8);

impl Instruction {
    /// Parses an instruction from memory at the address of the program counter.
    pub fn parse(pc: usize, memory: &mut Memory) -> Instruction {
        let raw_opcode = memory.read_u8(pc);
        let opcode = decode_opcode(raw_opcode);
        let len = opcode_len(&opcode);

        match len {
            1 => Instruction(raw_opcode, 0, 0),
            2 => Instruction(raw_opcode, memory.read_u8(pc + 1), 0),
            3 => Instruction(raw_opcode, memory.read_u8(pc + 1),
                             memory.read_u8(pc + 2)),
            _ => panic!("Invalid instruction length returned"),
        }
    }

    /// Disassembles the instruction into human readable assembly. Each opcode is
    /// mapped to a human readable name and a pretty print function. The pretty
    /// print function mimic Nintendulator and are used during CPU log
    /// comparisions.
    pub fn disassemble(&self, cpu: &CPU, memory: &mut Memory) -> String {
        let opcode = self.opcode();
        let len = opcode_len(&opcode);

        match opcode {
            ANDImm   => self.disassemble_immediate("AND"),
            ANDZero  => self.disassemble_zero_page("AND", memory),
            ANDZeroX => self.disassemble_zero_page_x("AND", memory, cpu),
            ANDAbs   => self.disassemble_absolute("AND", memory),
            ANDAbsX  => self.disassemble_absolute_x("AND", memory, cpu),
            ANDAbsY  => self.disassemble_absolute_y("AND", memory, cpu),
            ANDIndX  => self.disassemble_indirect_x("AND", memory, cpu),
            ANDIndY  => self.disassemble_indirect_y("AND", memory, cpu),
            BCCRel   => self.disassemble_relative("BCC", len, cpu),
            BCSRel   => self.disassemble_relative("BCS", len, cpu),
            BEQRel   => self.disassemble_relative("BEQ", len, cpu),
            BMIRel   => self.disassemble_relative("BMI", len, cpu),
            EORImm   => self.disassemble_immediate("EOR"),
            EORZero  => self.disassemble_zero_page("EOR", memory),
            EORZeroX => self.disassemble_zero_page_x("EOR", memory, cpu),
            EORAbs   => self.disassemble_absolute("EOR", memory),
            EORAbsX  => self.disassemble_absolute_x("EOR", memory, cpu),
            EORAbsY  => self.disassemble_absolute_y("EOR", memory, cpu),
            EORIndX  => self.disassemble_indirect_x("EOR", memory, cpu),
            EORIndY  => self.disassemble_indirect_y("EOR", memory, cpu),
            ORAImm   => self.disassemble_immediate("ORA"),
            ORAZero  => self.disassemble_zero_page("ORA", memory),
            ORAZeroX => self.disassemble_zero_page_x("ORA", memory, cpu),
            ORAAbs   => self.disassemble_absolute("ORA", memory),
            ORAAbsX  => self.disassemble_absolute_x("ORA", memory, cpu),
            ORAAbsY  => self.disassemble_absolute_y("ORA", memory, cpu),
            ORAIndX  => self.disassemble_indirect_x("ORA", memory, cpu),
            ORAIndY  => self.disassemble_indirect_y("ORA", memory, cpu),
            BITZero  => self.disassemble_zero_page("BIT", memory),
            BITAbs   => self.disassemble_absolute("BIT", memory),
            BNERel   => self.disassemble_relative("BNE", len, cpu),
            BPLRel   => self.disassemble_relative("BPL", len, cpu),
            BVCRel   => self.disassemble_relative("BVC", len, cpu),
            BVSRel   => self.disassemble_relative("BVS", len, cpu),
            CLCImp   => self.disassemble_implied("CLC"),
            CLDImp   => self.disassemble_implied("CLD"),
            CLIImp   => self.disassemble_implied("CLI"),
            CLVImp   => self.disassemble_implied("CLV"),
            ADCImm   => self.disassemble_immediate("ADC"),
            ADCZero  => self.disassemble_zero_page("ADC", memory),
            ADCZeroX => self.disassemble_zero_page_x("ADC", memory, cpu),
            ADCAbs   => self.disassemble_absolute("ADC", memory),
            ADCAbsX  => self.disassemble_absolute_x("ADC", memory, cpu),
            ADCAbsY  => self.disassemble_absolute_y("ADC", memory, cpu),
            ADCIndX  => self.disassemble_indirect_x("ADC", memory, cpu),
            ADCIndY  => self.disassemble_indirect_y("ADC", memory, cpu),
            SBCImm   => self.disassemble_immediate("SBC"),
            SBCZero  => self.disassemble_zero_page("SBC", memory),
            SBCZeroX => self.disassemble_zero_page_x("SBC", memory, cpu),
            SBCAbs   => self.disassemble_absolute("SBC", memory),
            SBCAbsX  => self.disassemble_absolute_x("SBC", memory, cpu),
            SBCAbsY  => self.disassemble_absolute_y("SBC", memory, cpu),
            SBCIndX  => self.disassemble_indirect_x("SBC", memory, cpu),
            SBCIndY  => self.disassemble_indirect_y("SBC", memory, cpu),
            CMPImm   => self.disassemble_immediate("CMP"),
            CMPZero  => self.disassemble_zero_page("CMP", memory),
            CMPZeroX => self.disassemble_zero_page_x("CMP", memory, cpu),
            CMPAbs   => self.disassemble_absolute("CMP", memory),
            CMPAbsX  => self.disassemble_absolute_x("CMP", memory, cpu),
            CMPAbsY  => self.disassemble_absolute_y("CMP", memory, cpu),
            CMPIndX  => self.disassemble_indirect_x("CMP", memory, cpu),
            CMPIndY  => self.disassemble_indirect_y("CMP", memory, cpu),
            CPXImm   => self.disassemble_immediate("CPX"),
            CPXZero  => self.disassemble_zero_page("CPX", memory),
            CPXAbs   => self.disassemble_absolute("CPX", memory),
            CPYImm   => self.disassemble_immediate("CPY"),
            CPYZero  => self.disassemble_zero_page("CPY", memory),
            CPYAbs   => self.disassemble_absolute("CPY", memory),
            INCZero  => self.disassemble_zero_page("INC", memory),
            INCZeroX => self.disassemble_zero_page_x("INC", memory, cpu),
            INCAbs   => self.disassemble_absolute("INC", memory),
            INCAbsX  => self.disassemble_absolute_x("INC", memory, cpu),
            INXImp   => self.disassemble_implied("INX"),
            INYImp   => self.disassemble_implied("INY"),
            DECZero  => self.disassemble_zero_page("DEC", memory),
            DECZeroX => self.disassemble_zero_page_x("DEC", memory, cpu),
            DECAbs   => self.disassemble_absolute("DEC", memory),
            DECAbsX  => self.disassemble_absolute_x("DEC", memory, cpu),
            DEXImp   => self.disassemble_implied("DEX"),
            DEYImp   => self.disassemble_implied("DEY"),
            ASLAcc   => self.disassemble_accumulator("ASL"),
            ASLZero  => self.disassemble_zero_page("ASL", memory),
            ASLZeroX => self.disassemble_zero_page_x("ASL", memory, cpu),
            ASLAbs   => self.disassemble_absolute("ASL", memory),
            ASLAbsX  => self.disassemble_absolute_x("ASL", memory, cpu),
            LSRAcc   => self.disassemble_accumulator("LSR"),
            LSRZero  => self.disassemble_zero_page("LSR", memory),
            LSRZeroX => self.disassemble_zero_page_x("LSR", memory, cpu),
            LSRAbs   => self.disassemble_absolute("LSR", memory),
            LSRAbsX  => self.disassemble_absolute_x("LSR", memory, cpu),
            ROLAcc   => self.disassemble_accumulator("ROL"),
            ROLZero  => self.disassemble_zero_page("ROL", memory),
            ROLZeroX => self.disassemble_zero_page_x("ROL", memory, cpu),
            ROLAbs   => self.disassemble_absolute("ROL", memory),
            ROLAbsX  => self.disassemble_absolute_x("ROL", memory, cpu),
            RORAcc   => self.disassemble_accumulator("ROR"),
            RORZero  => self.disassemble_zero_page("ROR", memory),
            RORZeroX => self.disassemble_zero_page_x("ROR", memory, cpu),
            RORAbs   => self.disassemble_absolute("ROR", memory),
            RORAbsX  => self.disassemble_absolute_x("ROR", memory, cpu),
            JMPAbs   => self.disassemble_absolute_noref("JMP"),
            JMPInd   => self.disassemble_indirect("JMP", memory),
            JSRAbs   => self.disassemble_absolute_noref("JSR"),
            LDAImm   => self.disassemble_immediate("LDA"),
            LDAZero  => self.disassemble_zero_page("LDA", memory),
            LDAZeroX => self.disassemble_zero_page_x("LDA", memory, cpu),
            LDAAbs   => self.disassemble_absolute("LDA", memory),
            LDAAbsX  => self.disassemble_absolute_x("LDA", memory, cpu),
            LDAAbsY  => self.disassemble_absolute_y("LDA", memory, cpu),
            LDAIndX  => self.disassemble_indirect_x("LDA", memory, cpu),
            LDAIndY  => self.disassemble_indirect_y("LDA", memory, cpu),
            LDXImm   => self.disassemble_immediate("LDX"),
            LDXZero  => self.disassemble_zero_page("LDX", memory),
            LDXZeroY => self.disassemble_zero_page_y("LDX", memory, cpu),
            LDXAbs   => self.disassemble_absolute("LDX", memory),
            LDXAbsY  => self.disassemble_absolute_y("LDX", memory, cpu),
            LDYImm   => self.disassemble_immediate("LDY"),
            LDYZero  => self.disassemble_zero_page("LDY", memory),
            LDYZeroX => self.disassemble_zero_page_x("LDY", memory, cpu),
            LDYAbs   => self.disassemble_absolute("LDY", memory),
            LDYAbsX  => self.disassemble_absolute_x("LDY", memory, cpu),
            BRKImp   => self.disassemble_implied("BRK"),
            NOPImp   => self.disassemble_implied("NOP"),
            PHAImp   => self.disassemble_implied("PHA"),
            PHPImp   => self.disassemble_implied("PHP"),
            PLAImp   => self.disassemble_implied("PLA"),
            PLPImp   => self.disassemble_implied("PLP"),
            RTIImp   => self.disassemble_implied("RTI"),
            RTSImp   => self.disassemble_implied("RTS"),
            SECImp   => self.disassemble_implied("SEC"),
            SEDImp   => self.disassemble_implied("SED"),
            SEIImp   => self.disassemble_implied("SEI"),
            STAZero  => self.disassemble_zero_page("STA", memory),
            STAZeroX => self.disassemble_zero_page_x("STA", memory, cpu),
            STAAbs   => self.disassemble_absolute("STA", memory),
            STAAbsX  => self.disassemble_absolute_x("STA", memory, cpu),
            STAAbsY  => self.disassemble_absolute_y("STA", memory, cpu),
            STAIndX  => self.disassemble_indirect_x("STA", memory, cpu),
            STAIndY  => self.disassemble_indirect_y("STA", memory, cpu),
            STXZero  => self.disassemble_zero_page("STX", memory),
            STXZeroY => self.disassemble_zero_page_y("STX", memory, cpu),
            STXAbs   => self.disassemble_absolute("STX", memory),
            STYZero  => self.disassemble_zero_page("STY", memory),
            STYZeroX => self.disassemble_zero_page_x("STY", memory, cpu),
            STYAbs   => self.disassemble_absolute("STY", memory),
            TAXImp   => self.disassemble_implied("TAX"),
            TAYImp   => self.disassemble_implied("TAY"),
            TSXImp   => self.disassemble_implied("TSX"),
            TXAImp   => self.disassemble_implied("TXA"),
            TXSImp   => self.disassemble_implied("TXS"),
            TYAImp   => self.disassemble_implied("TYA"),
            _ => { "GARBAGE".to_string() },
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

        // Get human readable hex of the instruction bytes. A pattern match is
        // used as bytes that do not exist in an instruction should not be
        // displayed (rather than displaying the default struct value 0). This
        // is to keep the logs consistent with Nintendulator's logs.
        let instr_str = match opcode_len(&opcode) {
            1 => format!("{:02X}      ", self.0),
            2 => format!("{:02X} {:02X}   ", self.0, self.1),
            3 => format!("{:02X} {:02X} {:02X}", self.0, self.1, self.2),
            _ => panic!("Invalid instruction length given")
        };

        // Prints the CPU state and disassembled instruction in a nice parsable
        // format. In the future this output will be used for automatically
        // testing the CPU's accuracy.
        //
        // NOTE: CYC is not cycles like the name sugests, but PPU dots. The PPU
        // can output 3 dots every CPU cycle on NTSC (PAL outputs an extra dot
        // every fifth CPU cycle).
        //       0       6   16     48       53       58       63       68        74
        let disassembled = self.disassemble(cpu, memory);
        return format!("{:04X}  {}  {:30}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:3}",
            cpu.pc, instr_str, disassembled, cpu.a, cpu.x, cpu.y, cpu.p, cpu.sp,
            cpu.ppu_dots);
    }

    /// Execute the instruction with a routine that corresponds with it's
    /// opcode. All routines for every instruction in the 6502 instruction set
    /// are present here.
    #[inline(always)]
    pub fn execute(&self, cpu: &mut CPU, memory: &mut Memory) {
        let opcode = self.opcode();
        let len = opcode_len(&opcode) as u16;

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
                    if page_cross(old_pc.wrapping_add(len as usize), cpu.pc as usize) != PageCross::Same {
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
                    if page_cross(old_pc.wrapping_add(len as usize), cpu.pc as usize) != PageCross::Same {
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
                    if page_cross(old_pc.wrapping_add(len as usize), cpu.pc as usize) != PageCross::Same {
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
                    if page_cross(old_pc.wrapping_add(len as usize), cpu.pc as usize) != PageCross::Same {
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
                let byte = self.dereference_absolute(memory);
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
                    if page_cross(old_pc.wrapping_add(len as usize), cpu.pc as usize) != PageCross::Same {
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
                    if page_cross(old_pc.wrapping_add(len as usize), cpu.pc as usize) != PageCross::Same {
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
                    if page_cross(old_pc.wrapping_add(len as usize), cpu.pc as usize) != PageCross::Same {
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
                    if page_cross(old_pc.wrapping_add(len as usize), cpu.pc as usize) != PageCross::Same {
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
            INCZero => {
                let addr = self.zero_page();
                let result = memory.read_u8(addr).wrapping_add(1);
                memory.write_u8(addr, result);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 5;
                cpu.pc += len;
            },
            INCZeroX => {
                let addr = self.zero_page_x(cpu);
                let result = memory.read_u8(addr).wrapping_add(1);
                memory.write_u8(addr, result);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            INCAbs => {
                let addr = self.absolute();
                let result = memory.read_u8(addr).wrapping_add(1);
                memory.write_u8(addr, result);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            INCAbsX => {
                let (addr, _) = self.absolute_x(cpu);
                let result = memory.read_u8(addr).wrapping_add(1);
                memory.write_u8(addr, result);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 7;
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
            DECZero => {
                let addr = self.zero_page();
                let result = memory.read_u8(addr).wrapping_sub(1);
                memory.write_u8(addr, result);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 5;
                cpu.pc += len;
            },
            DECZeroX => {
                let addr = self.zero_page_x(cpu);
                let result = memory.read_u8(addr).wrapping_sub(1);
                memory.write_u8(addr, result);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            DECAbs => {
                let addr = self.absolute();
                let result = memory.read_u8(addr).wrapping_sub(1);
                memory.write_u8(addr, result);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            DECAbsX => {
                let (addr, _) = self.absolute_x(cpu);
                let result = memory.read_u8(addr).wrapping_sub(1);
                memory.write_u8(addr, result);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.cycles += 7;
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
            ASLAcc => {
                let carry = cpu.a & 0x80 == 0x80;
                let result = cpu.a << 1;
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.a = result;
                cpu.cycles += 2;
                cpu.pc += len;
            },
            ASLZero => {
                let addr = self.zero_page();
                let mem = memory.read_u8(addr);
                let carry = mem & 0x80 == 0x80;
                let result = mem << 1;
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 5;
                cpu.pc += len;
            },
            ASLZeroX => {
                let addr = self.zero_page_x(cpu);
                let mem = memory.read_u8(addr);
                let carry = mem & 0x80 == 0x80;
                let result = mem << 1;
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            ASLAbs => {
                let addr = self.absolute();
                let mem = memory.read_u8(addr);
                let carry = mem & 0x80 == 0x80;
                let result = mem << 1;
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            ASLAbsX => {
                let (addr, _) = self.absolute_x(cpu);
                let mem = memory.read_u8(addr);
                let carry = mem & 0x80 == 0x80;
                let result = mem << 1;
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 7;
                cpu.pc += len;
            },
            LSRAcc => {
                let carry = cpu.a & 0x1 == 0x1;
                let result = cpu.a >> 1;
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.a = result;
                cpu.cycles += 2;
                cpu.pc += len;
            },
            LSRZero => {
                let addr = self.zero_page();
                let mem = memory.read_u8(addr);
                let carry = mem & 0x1 == 0x1;
                let result = mem >> 1;
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 5;
                cpu.pc += len;
            },
            LSRZeroX => {
                let addr = self.zero_page_x(cpu);
                let mem = memory.read_u8(addr);
                let carry = mem & 0x1 == 0x1;
                let result = mem >> 1;
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            LSRAbs => {
                let addr = self.absolute();
                let mem = memory.read_u8(addr);
                let carry = mem & 0x1 == 0x1;
                let result = mem >> 1;
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            LSRAbsX => {
                let (addr, _) = self.absolute_x(cpu);
                let mem = memory.read_u8(addr);
                let carry = mem & 0x1 == 0x1;
                let result = mem >> 1;
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 7;
                cpu.pc += len;
            },
            RORAcc => {
                let carry = cpu.a & 0x1 == 0x1;
                let result = (cpu.a >> 1) | (cpu.p << 7);
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.a = result;
                cpu.cycles += 2;
                cpu.pc += len;
            },
            RORZero => {
                let addr = self.zero_page();
                let mem = memory.read_u8(addr);
                let carry = mem & 0x1 == 0x1;
                let result = (mem >> 1) | (cpu.p << 7);
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 5;
                cpu.pc += len;
            },
            RORZeroX => {
                let addr = self.zero_page_x(cpu);
                let mem = memory.read_u8(addr);
                let carry = mem & 0x1 == 0x1;
                let result = (mem >> 1) | (cpu.p << 7);
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            RORAbs => {
                let addr = self.absolute();
                let mem = memory.read_u8(addr);
                let carry = mem & 0x1 == 0x1;
                let result = (mem >> 1) | (cpu.p << 7);
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            RORAbsX => {
                let (addr, _) = self.absolute_x(cpu);
                let mem = memory.read_u8(addr);
                let carry = mem & 0x1 == 0x1;
                let result = (mem >> 1) | (cpu.p << 7);
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 7;
                cpu.pc += len;
            },
            ROLAcc => {
                let carry = cpu.a & 0x80 == 0x80;
                let result = (cpu.a << 1) | (cpu.p & 0x1);
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                cpu.a = result;
                cpu.cycles += 2;
                cpu.pc += len;
            },
            ROLZero => {
                let addr = self.zero_page();
                let mem = memory.read_u8(addr);
                let carry = mem & 0x80 == 0x80;
                let result = (mem << 1) | (cpu.p & 0x1);
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 5;
                cpu.pc += len;
            },
            ROLZeroX => {
                let addr = self.zero_page_x(cpu);
                let mem = memory.read_u8(addr);
                let carry = mem & 0x80 == 0x80;
                let result = (mem << 1) | (cpu.p & 0x1);
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            ROLAbs => {
                let addr = self.absolute();
                let mem = memory.read_u8(addr);
                let carry = mem & 0x80 == 0x80;
                let result = (mem << 1) | (cpu.p & 0x1);
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 6;
                cpu.pc += len;
            },
            ROLAbsX => {
                let (addr, _) = self.absolute_x(cpu);
                let mem = memory.read_u8(addr);
                let carry = mem & 0x80 == 0x80;
                let result = (mem << 1) | (cpu.p & 0x1);
                cpu.toggle_carry_flag(carry);
                cpu.toggle_zero_flag(result);
                cpu.toggle_negative_flag(result);
                memory.write_u8(addr, result);
                cpu.cycles += 7;
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
            BRKImp => {
                // Fires an IRQ interrupt.
                let p = cpu.p;
                let pc = cpu.pc.wrapping_add(len);
                memory.stack_push_u16(cpu, pc);
                memory.stack_push_u8(cpu, p);
                cpu.set_break_command();
                cpu.cycles += 7;
                cpu.pc = pc;
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
            RTIImp => {
                let result = (memory.stack_pop_u8(cpu) & 0xEF) | (cpu.p & 0x20);
                cpu.p = result;
                cpu.pc = memory.stack_pop_u16(cpu);
                cpu.cycles += 6;
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
            STYZero => {
                memory.write_u8(self.zero_page(), cpu.y);
                cpu.cycles += 3;
                cpu.pc += len;
            },
            STYZeroX => {
                memory.write_u8(self.zero_page_x(cpu), cpu.y);
                cpu.cycles += 4;
                cpu.pc += len;
            },
            STYAbs => {
                memory.write_u8(self.absolute(), cpu.y);
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

        cpu.poll_irq(memory); // Poll IRQ after execution.
    }

    /// Obtain the opcode of the instruction.
    #[inline(always)]
    fn opcode(&self) -> Opcode {
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
        memory.read_u16_wrapped_msb(arg) as usize
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
        (memory.read_u16_wrapped_msb(addr) as usize, page_cross)
    }

    /// Sane version of indirect_x that gets the zero page address in the
    /// instruction, adds Y to it, then returns the resulting address.
    #[inline(always)]
    fn indirect_y(&self, cpu: &CPU, memory: &mut Memory) -> (usize, PageCross) {
        let arg = self.arg_u8() as usize;
        let base_addr = memory.read_u16_wrapped_msb(arg);
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

    /// Dereferences a zero page address.
    #[inline(always)]
    fn dereference_zero_page_unrestricted(&self, memory: &mut Memory) -> u8 {
        let addr = self.zero_page();
        memory.read_u8_unrestricted(addr)
    }

    /// Dereferences a zero page x address.
    #[inline(always)]
    fn dereference_zero_page_x_unrestricted(&self, memory: &mut Memory, cpu: &CPU) -> u8 {
        let addr = self.zero_page_x(cpu);
        memory.read_u8_unrestricted(addr)
    }

    /// Dereferences a zero page y address.
    #[inline(always)]
    fn dereference_zero_page_y_unrestricted(&self, memory: &mut Memory, cpu: &CPU) -> u8 {
        let addr = self.zero_page_y(cpu);
        memory.read_u8_unrestricted(addr)
    }

    /// Dereferences an absolute address.
    #[inline(always)]
    fn dereference_absolute_unrestricted(&self, memory: &mut Memory) -> u8 {
        let addr = self.absolute();
        memory.read_u8_unrestricted(addr)
    }

    /// Dereferences an absolute x address.
    #[inline(always)]
    fn dereference_absolute_x_unrestricted(&self, memory: &mut Memory, cpu: &CPU) -> u8 {
        let addr = self.absolute_x(cpu).0;
        memory.read_u8_unrestricted(addr)
    }

    /// Dereferences an absolute y address.
    #[inline(always)]
    fn dereference_absolute_y_unrestricted(&self, memory: &mut Memory, cpu: &CPU) -> u8 {
        let addr = self.absolute_y(cpu).0;
        memory.read_u8_unrestricted(addr)
    }

    /// Dereferences an indirect address.
    #[inline(always)]
    fn dereference_indirect_unrestricted(&self, memory: &mut Memory) -> u8 {
        let addr = self.indirect(memory);
        memory.read_u8_unrestricted(addr)
    }

    /// Dereferences an indirect x address.
    #[inline(always)]
    fn dereference_indirect_x_unrestricted(&self, memory: &mut Memory, cpu: &CPU) -> u8 {
        let addr = self.indirect_x(cpu, memory).0;
        memory.read_u8_unrestricted(addr)
    }

    /// Dereferences an indirect y address.
    #[inline(always)]
    fn dereference_indirect_y_unrestricted(&self, memory: &mut Memory, cpu: &CPU) -> u8 {
        let addr = self.indirect_y(cpu, memory).0;
        memory.read_u8_unrestricted(addr)
    }

    // Functions for aiding in disassembly. Each addressing mode has it's own
    // disassembly format in Nintendulator logs. These functions simply fill in
    // the blanks with provided parameters.

    /// Disassembles the instruction as if it's using accumulator addressing.
    fn disassemble_accumulator(&self, instr: &str) -> String {
        format!("{} A", instr)
    }

    /// Disassembles the instruction as if it's using implied addressing.
    fn disassemble_implied(&self, instr: &str) -> String {
        format!("{}", instr)
    }

    /// Disassembles the instruction as if it's using immediate addressing.
    fn disassemble_immediate(&self, instr: &str) -> String {
        format!("{} #${:02X}", instr, self.1)
    }

    /// Disassembles the instruction as if it's using zero page addressing.
    fn disassemble_zero_page(&self, instr: &str, memory: &mut Memory) -> String {
        format!("{} ${:02X} = {:02X}", instr, self.1, self.dereference_zero_page_unrestricted(memory))
    }

    /// Disassembles the instruction as if it's using zero page x addressing.
    fn disassemble_zero_page_x(&self, instr: &str, memory: &mut Memory, cpu: &CPU) -> String {
        format!("{} ${:02X},X @ {:02X} = {:02X}", instr, self.1, self.zero_page_x(cpu),
            self.dereference_zero_page_x_unrestricted(memory, cpu))
    }

    /// Disassembles the instruction as if it's using zero page y addressing.
    fn disassemble_zero_page_y(&self, instr: &str, memory: &mut Memory, cpu: &CPU) -> String {
        format!("{} ${:02X},Y @ {:02X} = {:02X}", instr, self.1, self.zero_page_y(cpu),
            self.dereference_zero_page_y_unrestricted(memory, cpu))
    }

    /// Disassembles the instruction as if it's using relative addressing.
    fn disassemble_relative(&self, instr: &str, len: u8, cpu: &CPU) -> String {
        let rel = add_relative(cpu.pc, self.relative());
        format!("{} ${:04X}", instr, rel + len as u16)
    }

    /// Disassembles the instruction as if it's using absolute addressing.
    ///
    /// NOTE: This function differs from disassemble_absolute as it does not
    /// dereference the value at the address. This is primarily used for JMP
    /// instructions for which the address is used directly.
    fn disassemble_absolute_noref(&self, instr: &str) -> String {
        format!("{} ${:02X}{:02X}", instr, self.2, self.1)
    }

    /// Disassembles the instruction as if it's using absolute addressing.
    fn disassemble_absolute(&self, instr: &str, memory: &mut Memory) -> String {
        format!("{} ${:02X}{:02X} = {:02X}", instr, self.2, self.1, self.dereference_absolute_unrestricted(memory))
    }

    /// Disassembles the instruction as if it's using absolute x addressing.
    fn disassemble_absolute_x(&self, instr: &str, memory: &mut Memory, cpu: &CPU) -> String {
        format!("{} ${:02x}{:02X},X @ {:04X} = {:02X}", instr, self.2, self.1,
            self.absolute_x(cpu).0, self.dereference_absolute_x_unrestricted(memory, cpu))
    }

    /// Disassembles the instruction as if it's using absolute y addressing.
    fn disassemble_absolute_y(&self, instr: &str, memory: &mut Memory, cpu: &CPU) -> String {
        format!("{} ${:02X}{:02X},Y @ {:04X} = {:02X}", instr, self.2, self.1,
            self.absolute_y(cpu).0, self.dereference_absolute_y_unrestricted(memory, cpu))
    }

    /// Disassembles the instruction as if it's using indirect addressing.
    fn disassemble_indirect(&self, instr: &str, memory: &mut Memory) -> String {
        format!("{} (${:02X}{:02X}) = {:04X}", instr, self.2, self.1, self.indirect(memory))
    }

    /// Disassembles the instruction as if it's using indirect x addressing.
    fn disassemble_indirect_x(&self, instr: &str, memory: &mut Memory, cpu: &CPU) -> String {
        format!("{} (${:02X},X) @ {:02X} = {:04X} = {:02X}", instr, self.1,
            self.1.wrapping_add(cpu.x), self.indirect_x(cpu, memory).0,
            self.dereference_indirect_x_unrestricted(memory, cpu))
    }

    /// Disassembles the instruction as if it's using indirect y addressing.
    fn disassemble_indirect_y(&self, instr: &str, memory: &mut Memory, cpu: &CPU) -> String {
        format!("{} (${:02X}),Y = {:04X} @ {:04X} = {:02X}", instr, self.1,
            memory.read_u16_wrapped_msb(self.arg_u16() as usize),
            self.indirect_y(cpu, memory).0, self.dereference_indirect_y_unrestricted(memory, cpu))
    }
}
