// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub struct CPU {
    // The program counter is a 16 bit register which points to the next
    // instruction to be executed. The value of program counter is modified
    // automatically as instructions are executed.
    //
    // The value of the program counter can be modified by executing a jump, a
    // relative branch or a subroutine call to another memory address or by
    // returning from a subroutine or interrupt.
    pc: u16,

    sp: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8
}
