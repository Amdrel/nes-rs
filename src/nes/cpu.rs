// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Flag constants that allow easy bitwise getting and setting of flag values.
const CARRY_FLAG       : u8 = 0x1;
const ZERO_FLAG        : u8 = 0x2;
const INTERRUPT_DISABLE: u8 = 0x4;
const DECIMAL_MODE     : u8 = 0x8;
const BREAK_COMMAND    : u8 = 0x10;
const OVERFLOW_FLAG    : u8 = 0x40;
const NEGATIVE_FLAG    : u8 = 0x80;

/// This is an implementation of 2A03 processor used in the NES. The 2A03 is
/// based off the 6502 processor with some minor changes such as having no
/// binary-coded decimal mode. Currently only the NTSC variant of the chip is
/// planned to be implemented.
///
/// Much of the information and comments are due credit to www.obelisk.me.uk,
/// which has really good information about the 6502 processor. If you're
/// interested in diving further, I recommend you give that site a visit.
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

    pub fn execute() {
    }
}
