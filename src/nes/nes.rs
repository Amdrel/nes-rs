// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use io::binutils::INESHeader;
use nes::cpu::CPU;
use std::panic;

use nes::memory::{
    Memory,
    TRAINER_START,
    TRAINER_SIZE,
    PRG_ROM_1_START,
    PRG_ROM_2_START,
    PRG_ROM_SIZE
};

/// The NES struct owns all hardware peripherals and lends them when needed.
pub struct NES {
    header: INESHeader,
    pub cpu: CPU,
    pub memory: Memory
}

impl NES {
    pub fn new(header: INESHeader, rom: Vec<u8>) -> NES {
        let cpu = CPU::new();
        let mut memory = Memory::new();

        // An offset is used when copying from the ROM into RAM as the presence
        // of a trainer will shift the locations of other structures.
        let mut cursor: usize = 0x10;

        // Copy the trainer data to 0x7000 if it exists.
        if header.has_trainer() {
            println!("Trainer data found");
            memory.memdump(TRAINER_START, &rom[0x10..0x210]);
            cursor += TRAINER_SIZE;
        }

        println!("{:?}", header);
        println!("Using {:?} mapper", header.mapper());
        println!("Using {:?} mirroring", header.mirror_type());

        // TODO: Mapper handling?

        // Copy PRG-ROM into memory so it can be addressed by the memory mapper.
        if header.prg_rom_size == 2 {
            // There are 2 PRG-ROM banks, copy them to memory.
            println!("2 PRG-ROM banks detected");
            let prg_rom_1_addr = cursor;
            let prg_rom_2_addr = cursor + PRG_ROM_SIZE;
            memory.memdump(PRG_ROM_1_START, &rom[prg_rom_1_addr..prg_rom_1_addr + PRG_ROM_SIZE]);
            memory.memdump(PRG_ROM_2_START, &rom[prg_rom_2_addr..prg_rom_2_addr + PRG_ROM_SIZE]);
        } else {
            // There is only 1 PRG-ROM bank, make the rom addressable at both
            // 0x8000 and 0xC000.
            println!("1 PRG-ROM bank detected");
            let prg_rom_1_addr = cursor;
            memory.memdump(PRG_ROM_1_START, &rom[prg_rom_1_addr..prg_rom_1_addr + PRG_ROM_SIZE]);
            memory.memdump(PRG_ROM_2_START, &rom[prg_rom_1_addr..prg_rom_1_addr + PRG_ROM_SIZE]);
        }

        NES {
            header: header,
            cpu: cpu,
            memory: memory
        }
    }

    /// Starts the execution loop and starts executing PRG-ROM.
    pub fn run(&mut self) -> i32 {
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            loop {
                self.cpu.execute(&mut self.memory);
            }
        }));

        // Log crash information if the execution loop panicked. If there was no
        // panic, return a success exit code and call it quits.
        match result {
            Ok(_) => {
                println!("Shutting down...");
                0 // Success exit code.
            },
            Err(_) => {
                println!("CPU Crash State: {:?}", self.cpu);
                101 // Runtime failure exit code.
            }
        }
    }
}
