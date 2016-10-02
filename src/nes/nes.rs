// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use io::binutils::INESHeader;
use io::errors::*;
use nes::cpu::CPU;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::io::stderr;
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
    pub header: INESHeader,
    pub runtime_options: NESRuntimeOptions,
    pub cpu: CPU,
    pub memory: Memory
}

impl NES {
    /// Initializes the NES emulator by dumping the ROM into memory and
    /// initializing the initial hardware state.
    pub fn new(rom: Vec<u8>, header: INESHeader, runtime_options: NESRuntimeOptions) -> NES {
        let cpu = CPU::new();
        let mut memory = Memory::new();

        // An offset is used when copying from the ROM into RAM as the presence
        // of a trainer will shift the locations of other structures.
        let mut cursor: usize = 0x10;

        // Copy the trainer data to 0x7000 if it exists.
        if header.has_trainer() {
            println!("[init] Trainer data found");
            memory.memdump(TRAINER_START, &rom[0x10..0x210]);
            cursor += TRAINER_SIZE;
        }

        println!("[init] Using {:?} mapper", header.mapper());
        println!("[init] Using {:?} mirroring", header.mirror_type());

        // TODO: Mapper handling?

        // Copy PRG-ROM into memory so it can be addressed by the memory mapper.
        if header.prg_rom_size == 2 {
            // There are 2 PRG-ROM banks, copy them to memory.
            println!("[init] 2 PRG-ROM banks detected");
            let prg_rom_1_addr = cursor;
            let prg_rom_2_addr = cursor + PRG_ROM_SIZE;
            memory.memdump(PRG_ROM_1_START, &rom[prg_rom_1_addr..prg_rom_1_addr + PRG_ROM_SIZE]);
            memory.memdump(PRG_ROM_2_START, &rom[prg_rom_2_addr..prg_rom_2_addr + PRG_ROM_SIZE]);
        } else {
            // There is only 1 PRG-ROM bank, make the rom addressable at both
            // 0x8000 and 0xC000.
            println!("[init] 1 PRG-ROM bank detected");
            let prg_rom_1_addr = cursor;
            memory.memdump(PRG_ROM_1_START, &rom[prg_rom_1_addr..prg_rom_1_addr + PRG_ROM_SIZE]);
            memory.memdump(PRG_ROM_2_START, &rom[prg_rom_1_addr..prg_rom_1_addr + PRG_ROM_SIZE]);
        }

        NES {
            header: header,
            runtime_options: runtime_options,
            cpu: cpu,
            memory: memory
        }
    }

    /// Starts the execution loop and starts executing PRG-ROM.
    pub fn run(&mut self) -> i32 {
        // Put the CPU into testing mode if a cpu log was passed in the runtime
        // options. This is done before execution so the log and the CPU state
        // are kept in sync.
        match self.runtime_options.cpu_log {
            Some(ref filename) => {
                match File::open(filename) {
                    Ok(f) => {
                        self.cpu.begin_testing(BufReader::new(f))
                    },
                    Err(e) => {
                        let mut stderr = stderr();
                        writeln!(stderr, "nes-rs: cannot open {}: {}", filename, e).unwrap();
                        return EXIT_CPU_LOG_NOT_FOUND
                    },
                }
            },
            None => {},
        }

        // Start executing the CPU and add a panic catcher so crash information
        // can be shown if the CPU panics.
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            loop {
                self.cpu.execute(&mut self.memory);
            }
        }));
        match result {
            Ok(_) => {
                println!("Shutting down...");
                EXIT_SUCCESS // Success exit code.
            },
            Err(_) => {
                println!("{}", self.cpu);
                EXIT_RUNTIME_FAILURE // Runtime failure exit code.
            }
        }
    }
}

pub struct NESRuntimeOptions {
    pub cpu_log: Option<String>,
}
