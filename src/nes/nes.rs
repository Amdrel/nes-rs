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

    pub fn test(&mut self) {
        // FIXME: Temporary code, please remove at some point!
        let (bank, idx) = self.memory.map(0xFFFF);
        println!("{:?}, index: {:#X}", bank, idx);
    }
}
