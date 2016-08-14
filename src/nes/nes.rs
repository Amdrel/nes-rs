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

// Constants for additional structures.
const TRAINER_START: usize = 0x7000;
const TRAINER_SIZE : usize = 512;

pub struct NES {
    header: INESHeader,
    cpu: CPU,
    memory: Memory
}

impl NES {
    pub fn new(header: INESHeader, rom: Vec<u8>) -> NES {
        let cpu = CPU::new();
        let mut memory = Memory::new();

        // An offset is used when copying from the ROM into RAM as the presence
        // of a trainer will shift the locations of other structures.
        let mut offset: usize = 0;

        // Copy the trainer data to 0x7000 if it exists.
        if header.has_trainer() {
            println!("Trainer data found");
            memory.memdump(TRAINER_START, &rom[0x10..0x210]);
            offset += TRAINER_SIZE;
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
        self.memory.write_u16(0x1000, 1000);
        println!("{}", self.memory.read_u16(0x0));
    }
}
