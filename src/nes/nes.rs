// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use nes::cpu::CPU;
use nes::memory::Memory;

pub struct NES {
    cpu: CPU,
    memory: Memory
}

impl NES {
    pub fn new(rom: Vec<u8>) -> NES {
        NES {
            cpu: CPU::new(),
            memory: Memory::new()
        }
    }

    /// FIXME: Temporary code, please remove at some point!
    pub fn test(&mut self) {
        //let (bank, idx) = self.memory.map(0xFFFF);
        //println!("{:?}, index: {:#X}", bank, idx);

        //let read1 = self.memory.read_u8(0x0);
        //println!("{}", read1);
        //self.memory.write_u8(0x800, 5);
        //let read2 = self.memory.read_u8(0x0);
        //println!("{}", read2);

        self.memory.write_u16(0x7FF, 1000);
        println!("{}", self.memory.read_u16(0x7FF));
    }
}
