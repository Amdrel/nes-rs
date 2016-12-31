// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use io::log;
use nes::instruction::Instruction;
use nes::memory::Memory;
use nes::nes::NESRuntimeOptions;
use std::fmt;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::num::ParseIntError;
use std::thread;
use std::time::Duration;
use std::u16;
use std::u8;
use utils::arithmetic;

// Flag constants that allow easy bitwise getting and setting of flag values.
pub const CARRY_FLAG:        u8 = 0x1;
pub const ZERO_FLAG:         u8 = 0x2;
pub const INTERRUPT_DISABLE: u8 = 0x4;
pub const DECIMAL_MODE:      u8 = 0x8;
pub const BREAK_COMMAND:     u8 = 0x10;
pub const OVERFLOW_FLAG:     u8 = 0x40;
pub const NEGATIVE_FLAG:     u8 = 0x80;

// How long it takes for a cycle to complete.
const CLOCK_SPEED: f32 = 558.65921787709;

/// This is an implementation of 2A03 processor used in the NES. The 2A03 is
/// based off the 6502 processor with some minor changes such as having no
/// binary-coded decimal mode. Currently only the NTSC variant of the chip is
/// planned to be implemented.
///
/// Much of the information and comments are due credit to www.obelisk.me.uk,
/// which has really good information about the 6502 processor. If you're
/// interested in diving further, I recommend you give that site a visit.
///
/// TODO: Add condition to behave like the 2A07 (PAL).
pub struct CPU {
    // The program counter is a 16-bit register which points to the next
    // instruction to be executed. The value of program counter is modified
    // automatically as instructions are executed.
    //
    // The value of the program counter can be modified by executing a jump, a
    // relative branch, a subroutine call to another memory address, by
    // returning from a subroutine, or by an interrupt.
    pub pc: u16,

    // The processor supports a 256 byte stack located between $0100 and $01FF.
    // The stack pointer is an 8-bit register and holds the next free location
    // on the stack. The location of the stack is fixed and cannot be moved and
    // grows downwards.
    pub sp: u8,

    // The 8-bit accumulator is used all arithmetic and logical operations (with
    // the exception of increments and decrements). The contents of the
    // accumulator can be stored and retrieved either from memory or the stack.
    pub a: u8,

    // The 8-bit X register can be used to control information, compare values
    // in memory, and be incremented or decremented. The X register is special
    // as it can be used to get a copy of the stack pointer or change its value.
    pub x: u8,

    // The 8-bit Y register like X, can be used to manage information and be
    // incremented or decremented; however it doesn't have any special functions
    // like the X register does.
    pub y: u8,

    // The Processor Status register contains a list of flags that are set and
    // cleared by instructions to record the results of operations. Each flag
    // has a special bit within the register (8 bits).  Instructions exist to
    // set, clear, and read the various flags. One even allows pushing or
    // pulling the flags to the stack.
    //
    // Carry Flag:
    //
    // The carry flag is set if the last operation caused an overflow from bit 7
    // of the result or an underflow from bit 0. This condition is set during
    // arithmetic, comparison and during logical shifts. It can be explicitly
    // set using the 'Set Carry Flag' (SEC) instruction and cleared with 'Clear
    // Carry Flag' (CLC).
    //
    // Zero Flag:
    //
    // The zero flag is set if the result of the last operation as was zero.
    //
    // Interrupt Disable:
    //
    // The interrupt disable flag is set if the program has executed a 'Set
    // Interrupt Disable' (SEI) instruction. While this flag is set the
    // processor will not respond to interrupts from devices until it is cleared
    // by a 'Clear Interrupt Disable' (CLI) instruction.
    //
    // Decimal Mode: (UNUSED in 2A03)
    //
    // While the decimal mode flag is set the processor will obey the rules of
    // Binary Coded Decimal (BCD) arithmetic during addition and subtraction.
    // The flag can be explicitly set using 'Set Decimal Flag' (SED) and cleared
    // with 'Clear Decimal Flag' (CLD).
    //
    // Break Command:
    //
    // The break command bit is set when a BRK instruction has been executed and
    // an interrupt has been generated to process it.
    //
    // Overflow Flag:
    //
    // The overflow flag is set during arithmetic operations if the result has
    // yielded an invalid 2's complement result (e.g. adding to positive numbers
    // and ending up with a negative result: 64 + 64 => -128). It is determined
    // by looking at the carry between bits 6 and 7 and between bit 7 and the
    // carry flag.
    //
    // Negative Flag:
    //
    // The negative flag is set if the result of the last operation had bit 7
    // set to a one.
    pub p: u8,

    // The amount of cycles currently accumulated. A cycle represents a unit of
    // time (the time it takes for the CPU clock to fire). Different
    // instructions take a different amount of cycles to complete depending on
    // their complexity.
    pub cycles: u16,

    // Number of cycles since last v-sync.
    pub ppu_dots: u16,

    // Options passed from the command-line that may influence how the CPU
    // behaves.
    runtime_options: NESRuntimeOptions,

    // This will contain an open file if the CPU is in testing mode. It will be
    // read during program execution and compared against.
    execution_log: Option<BufReader<File>>,
}

impl CPU {
    pub fn new(runtime_options: NESRuntimeOptions, pc: u16) -> CPU {
        CPU {
            pc: pc,
            sp: 0xFD,
            a: 0,
            x: 0,
            y: 0,
            p: 0x24,
            cycles: 0,
            ppu_dots: 0,
            runtime_options: runtime_options,
            execution_log: None,
        }
    }

    /// Sets the carry flag in the status register.
    #[inline(always)]
    pub fn set_carry_flag(&mut self) {
        self.p |= CARRY_FLAG;
    }

    /// Sets the zero flag in the status register.
    #[inline(always)]
    pub fn set_zero_flag(&mut self) {
        self.p |= ZERO_FLAG;
    }

    /// Sets the interrupt disable flag in the status register.
    #[inline(always)]
    pub fn set_interrupt_disable(&mut self) {
        self.p |= INTERRUPT_DISABLE;
    }

    /// Sets the decimal mode flag in the status register.
    /// NOTE: This flag is disabled in the 2A03 variation of the 6502.
    #[inline(always)]
    pub fn set_decimal_mode(&mut self) {
        self.p |= DECIMAL_MODE;
    }

    /// Sets the break command flag in the status register.
    #[inline(always)]
    pub fn set_break_command(&mut self) {
        self.p |= BREAK_COMMAND;
    }

    /// Sets the overflow flag in the status register.
    #[inline(always)]
    pub fn set_overflow_flag(&mut self) {
        self.p |= OVERFLOW_FLAG;
    }

    /// Sets the negative flag in the status register.
    #[inline(always)]
    pub fn set_negative_flag(&mut self) {
        self.p |= NEGATIVE_FLAG;
    }

    /// Unsets the carry flag in the status register.
    #[inline(always)]
    pub fn unset_carry_flag(&mut self) {
        self.p &= !CARRY_FLAG;
    }

    /// Unsets the zero flag in the status register.
    #[inline(always)]
    pub fn unset_zero_flag(&mut self) {
        self.p &= !ZERO_FLAG;
    }

    /// Unsets the interrupt disable flag in the status register.
    #[inline(always)]
    pub fn unset_interrupt_disable(&mut self) {
        self.p &= !INTERRUPT_DISABLE;
    }

    /// Unsets the decimal mode flag in the status register.
    /// NOTE: This flag is disabled in the 2A03 variation of the 6502.
    #[inline(always)]
    pub fn unset_decimal_mode(&mut self) {
        self.p &= !DECIMAL_MODE;
    }

    /// Unsets the break command flag in the status register.
    #[inline(always)]
    pub fn unset_break_command(&mut self) {
        self.p &= !BREAK_COMMAND;
    }

    /// Unsets the overflow flag in the status register.
    #[inline(always)]
    pub fn unset_overflow_flag(&mut self) {
        self.p &= !OVERFLOW_FLAG;
    }

    /// Unsets the negative flag in the status register.
    #[inline(always)]
    pub fn unset_negative_flag(&mut self) {
        self.p &= !NEGATIVE_FLAG;
    }

    /// Sets the carry flag in the status register.
    #[inline(always)]
    pub fn carry_flag_set(&self) -> bool {
        self.p & CARRY_FLAG == CARRY_FLAG
    }

    /// Sets the zero flag in the status register.
    #[inline(always)]
    pub fn zero_flag_set(&self) -> bool {
        self.p & ZERO_FLAG == ZERO_FLAG
    }

    /// Sets the interrupt disable flag in the status register.
    #[inline(always)]
    pub fn interrupt_disable_set(&self) -> bool {
        self.p & INTERRUPT_DISABLE == INTERRUPT_DISABLE
    }

    /// Sets the decimal mode flag in the status register.
    /// NOTE: This flag is disabled in the 2A03 variation of the 6502.
    #[inline(always)]
    pub fn decimal_mode_set(&self) -> bool {
        self.p & DECIMAL_MODE == DECIMAL_MODE
    }

    /// Sets the break command flag in the status register.
    #[inline(always)]
    pub fn break_command_set(&self) -> bool {
        self.p & BREAK_COMMAND == BREAK_COMMAND
    }

    /// Sets the overflow flag in the status register.
    #[inline(always)]
    pub fn overflow_flag_set(&self) -> bool {
        self.p & OVERFLOW_FLAG == OVERFLOW_FLAG
    }

    /// Sets the negative flag in the status register.
    #[inline(always)]
    pub fn negative_flag_set(&self) -> bool {
        self.p & NEGATIVE_FLAG == NEGATIVE_FLAG
    }

    /// Sets the carry flag if the passed overflow is true, otherwise the flag
    /// is unset.
    #[inline(always)]
    pub fn toggle_carry_flag(&mut self, overflow: bool) {
        if overflow {
            self.set_carry_flag();
        } else {
            self.unset_carry_flag();
        }
    }

    /// Sets the zero flag if the value passed (typically a reference to a
    /// register) if the value is zero, otherwise it's unset.
    #[inline(always)]
    pub fn toggle_zero_flag(&mut self, value: u8) {
        if value == 0 {
            self.set_zero_flag();
        } else {
            self.unset_zero_flag();
        }
    }

    /// Sets the negative flag if the value passed (typically a reference to a
    /// register) if the value is negative, otherwise it's unset.
    #[inline(always)]
    pub fn toggle_negative_flag(&mut self, value: u8) {
        if arithmetic::is_negative(value) {
            self.set_negative_flag();
        } else {
            self.unset_negative_flag();
        }
    }

    /// Save the passed execution log which will be used to compare the CPU's
    /// execution to the passed Nintendulator log.
    pub fn begin_testing(&mut self, log: BufReader<File>) {
        self.execution_log = Some(log);
    }

    /// Parse an instruction from memory at the address the program counter
    /// currently points execute it. All instruction logic is in instruction.rs.
    pub fn execute<M: Memory>(&mut self, memory: &mut M) -> u16 {
        let instr = Instruction::parse(self.pc as usize, memory);
        if self.runtime_options.verbose || self.execution_log.is_some() {
            let raw_fragment = instr.log(self, memory);

            // Print the log fragment only if verbose mode is enabled. Logs are
            // formatted like Nintendulator logs.
            if self.runtime_options.verbose {
                log::log("cpu", format!("{}", raw_fragment), &self.runtime_options);
            }

            // Compare the current state of the emulator against a log if one was
            // provided on the command-line.
            if let Some(ref mut execution_log) = self.execution_log {
                // Read the next line from the log.
                let mut log_fragment = String::new();
                execution_log.read_line(&mut log_fragment).unwrap();

                // Parse and compare both of the CPU frames.
                if CPUFrame::parse(raw_fragment.as_str()) != CPUFrame::parse(log_fragment.as_str()) {
                    log::log("error", "FATAL ERROR: Mismatched CPU frames:", &self.runtime_options);
                    log::log("error", format!("Emulator Frame: {}", raw_fragment), &self.runtime_options);
                    log::log("error", format!("Log Frame:      {}", log_fragment), &self.runtime_options);
                    panic!("Mismatched CPU frames");
                }
            }
        }

        // Execute the instruction located at the current PC.
        instr.execute(self, memory);

        // Save the cycle count of the last instruction execution so it may be
        // returned after sleeping through the cycles.
        let old_cycles = self.cycles;

        // The instruction execution should have updated the remaining cycle count in the CPU.
        // Sleep for the clock speed multiplied by the cycle cound.
        //
        // NOTE: When interrupts are implemented, this may have to be changed as some interrupts
        // are delayed by n number of cycles.
        thread::sleep(Duration::new(0, (CLOCK_SPEED * self.cycles as f32) as u32));

        // Reset cycles and set PPU dots for debugging purposes.
        self.ppu_dots = (self.ppu_dots + (self.cycles * 3)) % 341;
        self.cycles = 0;

        // Return the number of cycles that was required to execute the
        // instruction so the PPU can be synchronized.
        old_cycles
    }

    /// Returns "SET" if the passed boolean is true, otherwise "UNSET". This
    /// function is used to display flags when the CPU crashes.
    fn fmt_flag(flag: bool) -> &'static str {
        if flag { "SET" } else { "UNSET" }
    }
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\nCPU Crash State:").unwrap();
        writeln!(f, "    Program Counter: {:#X}", self.pc).unwrap();
        writeln!(f, "    Stack Pointer:   {:#X}", self.sp).unwrap();
        writeln!(f, "    Accumulator:     {:#X}", self.a).unwrap();
        writeln!(f, "    X Register:      {:#X}", self.x).unwrap();
        writeln!(f, "    Y Register:      {:#X}", self.y).unwrap();
        writeln!(f, "").unwrap();
        writeln!(f, "Status Register: {:#X}", self.p).unwrap();
        writeln!(f, "    Carry Flag:        {}", CPU::fmt_flag(self.carry_flag_set())).unwrap();
        writeln!(f, "    Zero Flag:         {}", CPU::fmt_flag(self.zero_flag_set())).unwrap();
        writeln!(f, "    Interrupt Disable: {}", CPU::fmt_flag(self.interrupt_disable_set())).unwrap();
        writeln!(f, "    Decimal Mode:      {}", CPU::fmt_flag(self.decimal_mode_set())).unwrap();
        writeln!(f, "    Break Command:     {}", CPU::fmt_flag(self.break_command_set())).unwrap();
        writeln!(f, "    Overflow Flag:     {}", CPU::fmt_flag(self.overflow_flag_set())).unwrap();
        writeln!(f, "    Negative Flag:     {}", CPU::fmt_flag(self.negative_flag_set()))
    }
}

/// CPU state for use during automated CPU testing. These values are contained
/// inside of Nintendulator logs and used for comparing log frames to test CPU
/// accuracy.
#[derive(Debug, PartialEq)]
struct CPUFrame {
    instruction: Instruction,
    disassembly: String,
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    sp: u8,
    cycles: u16,
}

impl CPUFrame {
    /// Parses a Nintendulator log frame and packs the parsed values into a
    /// structure. The structure can then be compared using the PartialEq trait.
    pub fn parse(frame: &str) -> Result<CPUFrame, ParseIntError> {
        // Nintendulator stores instructions as 8-bit hex in the log frame.
        let instr = Instruction(
            CPUFrame::extract_word(&frame[6..8]),
            CPUFrame::extract_word(&frame[9..11]),
            CPUFrame::extract_word(&frame[12..14]));

        Ok(CPUFrame {
            instruction: instr,
            disassembly: String::from(&frame[16..46]),
            pc:     try!(u16::from_str_radix(&frame[0..4], 16)),
            a:      try!(u8::from_str_radix(&frame[50..52], 16)),
            x:      try!(u8::from_str_radix(&frame[55..57], 16)),
            y:      try!(u8::from_str_radix(&frame[60..62], 16)),
            p:      try!(u8::from_str_radix(&frame[65..67], 16)),
            sp:     try!(u8::from_str_radix(&frame[71..73], 16)),
            cycles: try!(u16::from_str_radix(&frame[78..81].trim(), 10)),
        })
    }

    /// Parses a hex encoded 8-bit integer.
    fn extract_word(slice: &str) -> u8 {
        match u8::from_str_radix(slice, 16) {
            Ok(num) => num,
            Err(_) => 0,
        }
    }
}
