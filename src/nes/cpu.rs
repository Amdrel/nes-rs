// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use nes::instruction::Instruction;
use nes::memory::Memory;
use std::fmt;

// Flag constants that allow easy bitwise getting and setting of flag values.
pub const CARRY_FLAG       : u8 = 0x1;
pub const ZERO_FLAG        : u8 = 0x2;
pub const INTERRUPT_DISABLE: u8 = 0x4;
pub const DECIMAL_MODE     : u8 = 0x8;
pub const BREAK_COMMAND    : u8 = 0x10;
pub const OVERFLOW_FLAG    : u8 = 0x40;
pub const NEGATIVE_FLAG    : u8 = 0x80;

/// This is an implementation of 2A03 processor used in the NES. The 2A03 is
/// based off the 6502 processor with some minor changes such as having no
/// binary-coded decimal mode. Currently only the NTSC variant of the chip is
/// planned to be implemented.
///
/// Much of the information and comments are due credit to www.obelisk.me.uk,
/// which has really good information about the 6502 processor. If you're
/// interested in diving further, I recommend you give that site a visit.
#[derive(Debug)]
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
    pub cycles: u16
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            pc: 0xC000,
            sp: 0xFD,
            a: 0,
            x: 0,
            y: 0,
            p: 0x24,
            cycles: 0
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

    /// Parse an instruction from memory at the address the program counter
    /// currently points execute it. All instruction logic is in instruction.rs.
    pub fn execute(&mut self, memory: &mut Memory) {
        // NOTE: At this time, some parsing logic is done twice for the sake of
        // code simplicity. In the future I may rework the function arguments to
        // reuse as much data as possible since this is high-performance code.
        let instr = Instruction::parse(self.pc as usize, memory);
        instr.log(self, memory);
        instr.execute(self, memory);
    }

    fn fmt_flag(&self, flag: bool) -> &'static str {
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
        writeln!(f, "    Carry Flag:        {}", self.fmt_flag(self.carry_flag_set())).unwrap();
        writeln!(f, "    Zero Flag:         {}", self.fmt_flag(self.zero_flag_set())).unwrap();
        writeln!(f, "    Interrupt Disable: {}", self.fmt_flag(self.interrupt_disable_set())).unwrap();
        writeln!(f, "    Decimal Mode:      {}", self.fmt_flag(self.decimal_mode_set())).unwrap();
        writeln!(f, "    Break Command:     {}", self.fmt_flag(self.break_command_set())).unwrap();
        writeln!(f, "    Overflow Flag:     {}", self.fmt_flag(self.overflow_flag_set())).unwrap();
        writeln!(f, "    Negative Flag:     {}", self.fmt_flag(self.negative_flag_set()))
    }
}
