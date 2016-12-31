// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use io::binutils::INESHeader;
use io::errors::*;
use io::log;
use nes::cpu::CPU;
use nes::ppu::PPU;
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

/// The NES struct owns all hardware peripherals and lends them when needed. The
/// runtime cost of this should be removed with optimized builds (untested).
pub struct NES<M: Memory> {
    pub header: INESHeader,
    pub runtime_options: NESRuntimeOptions,
    pub cpu: CPU,
    pub ppu: PPU,
    pub memory: M,
}

impl<M: Memory> NES<M> {
    /// Initializes the NES emulator by dumping the ROM into memory and
    /// initializing the initial hardware state.
    pub fn new(rom: Vec<u8>, header: INESHeader, runtime_options: NESRuntimeOptions) -> Self {
        // An offset is used when copying from the ROM into RAM as the presence
        // of a trainer will shift the locations of other structures.
        let mut cursor: usize = 0x10;

        // Spew out some useful metadata about the rom when verbose is on.
        log::log("init", format!("Using {:?} mapper", header.mapper()), &runtime_options);
        log::log("init", format!("Using {:?} mirroring", header.mirror_type()), &runtime_options);

        // Copy the trainer data to 0x7000 if it exists and adjust the cursor
        // size to accommodate. Trainer data will offset the location of ROM
        // data in the INES ROM file.
        let mut memory = M::new();
        if header.has_trainer() {
            log::log("init", "Trainer data found", &runtime_options);
            memory.memdump(TRAINER_START, &rom[0x10..0x210]);
            cursor += TRAINER_SIZE;
        }

        // Copy PRG-ROM into memory so it can be addressed by the chosen memory
        // mapper. PRG-ROM bank 1 begins at 0x8000 and bank 2 begins at 0xC000.
        //
        // In the event that there are 2 PRG-ROM banks, make both banks
        // addressable at their respective locations. However if there's only
        // one bank, make PRG-ROM bank 1 addressable starting from both
        // addresses.
        //
        // NOTE: Should this be moved to mapper code?
        if header.prg_rom_size == 2 {
            log::log("init", "2 PRG-ROM banks detected", &runtime_options);
            let prg_rom_1_addr = cursor;
            let prg_rom_2_addr = cursor + PRG_ROM_SIZE;
            memory.memdump(PRG_ROM_1_START, &rom[prg_rom_1_addr..prg_rom_1_addr + PRG_ROM_SIZE]);
            memory.memdump(PRG_ROM_2_START, &rom[prg_rom_2_addr..prg_rom_2_addr + PRG_ROM_SIZE]);
        } else {
            log::log("init", "1 PRG-ROM bank detected", &runtime_options);
            let prg_rom_1_addr = cursor;
            memory.memdump(PRG_ROM_1_START, &rom[prg_rom_1_addr..prg_rom_1_addr + PRG_ROM_SIZE]);
            memory.memdump(PRG_ROM_2_START, &rom[prg_rom_1_addr..prg_rom_1_addr + PRG_ROM_SIZE]);
        }

        // Set the initial program counter to the address stored at 0xFFFC (this
        // allows ROMs to specify entry point). If a program counter was
        // specified on the command-line, use that one instead.
        let pc = match runtime_options.program_counter {
            Some(pc) => pc,
            None => {
                memory.read_u16(0xFFFC)
            },
        };

        NES {
            header: header,
            cpu: CPU::new(runtime_options.clone(), pc),
            ppu: PPU::new(runtime_options.clone()),
            runtime_options: runtime_options,
            memory: memory,
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

        // Start cycling the CPU and PPU and add a panic catcher so crash
        // information can be shown if the CPU panics.
        //
        // The PPU cycles every 3 CPU cycles, though there may need to be
        // changes made for PAL (currently assumes NTSC PPU clock speed).
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            let mut cycles: u16 = 0;
            loop {
                cycles += self.cpu.execute(&mut self.memory);
                while cycles >= 3 {
                    self.ppu.execute();
                    cycles -= 3;
                }
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

/// Flags and other information set through command-line arguments.
#[derive(Clone)]
pub struct NESRuntimeOptions {
    pub cpu_log: Option<String>,
    pub program_counter: Option<u16>,
    pub verbose: bool,
}
