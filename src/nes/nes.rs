// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use io::binutils::INESHeader;
use nes::cpu::CPU;
use nes::memory::Memory;

pub struct NES {
    header: INESHeader,
    cpu: CPU,
    memory: Memory
}

impl NES {
    pub fn new(header: INESHeader, rom: Vec<u8>) -> NES {
        let cpu = CPU::new();
        let memory = Memory::new();

        // An offset is used when copying from the ROM into RAM as the presence
        // of a trainer will shift the locations of other structures.
        let mut offset = 0x0;

        if header.has_trainer() {
            offset += 512;
            println!("Trainer data found, copying to 0x7000...");
        }

        if header.prg_rom_size == 2 {
            println!("2 PRG-ROM banks detected");
        } else {
            println!("1 PRG-ROM bank detected");
        }

        NES {
            header: header,
            cpu: cpu,
            memory: memory
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

        self.memory.write_u16(0x1000, 1000);
        println!("{}", self.memory.read_u16(0x0));
    }
}
