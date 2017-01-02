// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use io::log;
use nes::memory::Memory;
use nes::memory::MiscRegisterStatus;
use nes::memory::PPURegisterStatus;
use nes::nes::NESRuntimeOptions;

use nes::memory::{
    PPU_CTRL_REGISTERS_SIZE,
    MISC_CTRL_REGISTERS_SIZE,
};

const SPR_RAM_SIZE: usize = 0x00FF;

// Memory map section sizes.
const PATTERN_TABLES_SIZE: usize = 0x2000;
const NAME_TABLES_SIZE:    usize = 0x1000;
const PALETTES_SIZE:       usize = 0x0020;

// Memory map bounds.
const PATTERN_TABLES_START:     usize = 0x0000;
const PATTERN_TABLES_END:       usize = 0x1FFF;
const NAME_TABLES_START:        usize = 0x2000;
const NAME_TABLES_END:          usize = 0x2FFF;
const NAME_TABLES_MIRROR_START: usize = 0x3000;
const NAME_TABLES_MIRROR_END:   usize = 0x3EFF;
const PALETTES_START:           usize = 0x3F00;
const PALETTES_END:             usize = 0x3F1F;
const PALETTES_MIRROR_START:    usize = 0x3F20;
const PALETTES_MIRROR_END:      usize = 0x3FFF;
const MIRROR_START:             usize = 0x4000;
const MIRROR_END:               usize = 0xFFFF;

/// This is an implementation of the 2C02 PPU used in the NES. This piece of
/// hardware is responsible for drawing graphics to the television the console
/// is hooked up to; however in our case we draw to an SDL surface.
pub struct PPU {
    ppu_ctrl: u8,
    ppu_mask: u8,
    ppu_status: u8,

    // The runtime options contain some useful information such as television
    // standard which affect the clock rate of the PPU.
    runtime_options: NESRuntimeOptions,

    // The PPU has 2 pattern tables which store 8x8 pixel tiles which can be
    // drawn to the screen.
    pattern_tables: [u8; PATTERN_TABLES_SIZE],

    // The name tables are matrices of numbers that point to tiles stored in the
    // pattern tables. Each name table has an associated attribute table, which
    // contains the upper 2 bits of colors for each of the associated tiles.
    name_tables: [u8; NAME_TABLES_SIZE],

    // The PPU has 2 color palettes each containing 16 entires selected from the
    // PPU total selection of 52 colors. Because of this all possible colors the
    // PPU can create cannot be shown at once.
    //
    // Another thing to note is that the background color entry is mirrored
    // every 4 bytes so the effective number of color entries is reduced to 13
    // rather than 16.
    palettes: [u8; PALETTES_SIZE],

    // Where sprites are stored (different bus).
    spr_ram: [u8; SPR_RAM_SIZE],
}

impl PPU {
    /// Initializes the PPU and it's internal memory.
    pub fn new(runtime_options: NESRuntimeOptions) -> Self {
        PPU {
            ppu_ctrl: 0,
            ppu_mask: 0,
            ppu_status: 0,

            runtime_options: runtime_options,
            pattern_tables: [0; PATTERN_TABLES_SIZE],
            name_tables: [0; NAME_TABLES_SIZE],
            palettes: [0; PALETTES_SIZE],
            spr_ram: [0; SPR_RAM_SIZE],
        }
    }

    /// Maps a PPU virtual addresses to a physical address used internally by
    /// the PPU emulator.
    fn map(&mut self, addr: usize) -> (&mut [u8], usize) {
        match addr {
            PATTERN_TABLES_START...PATTERN_TABLES_END =>
                (&mut self.pattern_tables, addr),
            NAME_TABLES_START...NAME_TABLES_END =>
                (&mut self.name_tables, addr - NAME_TABLES_START),
            NAME_TABLES_MIRROR_START...NAME_TABLES_MIRROR_END =>
                (&mut self.name_tables, (addr - NAME_TABLES_START) % NAME_TABLES_SIZE),
            PALETTES_START...PALETTES_END =>
                (&mut self.palettes, addr - PALETTES_START),
            PALETTES_MIRROR_START...PALETTES_MIRROR_END =>
                (&mut self.palettes, (addr - PALETTES_START) % PALETTES_SIZE),
            MIRROR_START...MIRROR_END =>
                self.map(addr - MIRROR_START), // Lazy recursion to share nested mirror logic ^^^.
            _ => { panic!("Unable to map virtual address {:#X} to any physical address", addr) },
        }
    }

    /// Reads a byte from PPU memory at the given virtual address.
    fn read_u8(&mut self, addr: usize) -> u8 {
        let (bank, addr) = self.map(addr);
        bank[addr]
    }

    /// Writes a byte to PPU memory at the given virtual address.
    fn write_u8(&mut self, addr: usize, value: u8) {
        let (bank, addr) = self.map(addr);
        bank[addr] = value;
    }

    /// Copy data from main memory to the PPU's internal sprite memory.
    fn exec_dma(&mut self) {
    }

    /// Reads the contents of the DMA register and executes DMA if written since
    /// the last PPU cycle.
    fn handle_dma_register<M: Memory>(&mut self, index: usize, state: MiscRegisterStatus, memory: &mut M) {
        if state != MiscRegisterStatus::Written {
            return;
        }
        let register = memory.misc_ctrl_registers()[index];
        println!("{:02X}", register);
        panic!("Implement DMA to continue!");
    }

    fn handle_ppu_ctrl<M: Memory>(&mut self, index: usize, state: PPURegisterStatus, memory: &mut M) {
        if state != PPURegisterStatus::Written {
            return;
        }
        let register = memory.ppu_ctrl_registers()[index];
        println!("{:02X}", register);
        panic!("Implement PPU CTRL to continue!");
    }

    /// Checks the status of PPU I/O registers and executes PPU functionality
    /// depending on their states.
    fn check_ppu_registers<M: Memory>(&mut self, memory: &mut M) {
        let mut io_registers_state = [PPURegisterStatus::Untouched; PPU_CTRL_REGISTERS_SIZE];
        io_registers_state.clone_from_slice(memory.ppu_ctrl_registers_status());

        for (index, state) in io_registers_state.iter().enumerate() {
            println!("PPU REGISTERS :: index: 0x{:02X}, state: {:?}", index, state);
            match index {
                0x00 => self.handle_ppu_ctrl(index, state.clone(), memory),
                _ => {
                    if state.clone() != PPURegisterStatus::Untouched {
                        panic!("Unsupported register modified");
                    }
                },
            }
        }
    }

    /// Checks the status of misc I/O registers and executes PPU functionality
    /// depending on their states.
    fn check_misc_registers<M: Memory>(&mut self, memory: &mut M) {
        let mut io_registers_state = [MiscRegisterStatus::Untouched; MISC_CTRL_REGISTERS_SIZE];
        io_registers_state.clone_from_slice(memory.misc_ctrl_registers_status());

        for (index, state) in io_registers_state.iter().enumerate() {
            println!("MISC REGISTERS :: index: 0x{:02X}, state: {:?}", index, state);
            match index {
                0x14 => self.handle_dma_register(index, state.clone(), memory),
                _ => {
                    if state.clone() != MiscRegisterStatus::Untouched {
                        panic!("Unsupported register modified");
                    }
                },
            };
        }
    }

    /// Executes routine PPU logic and returns stolen cycles from operations
    /// such as DMA transfers if the PPU hogged the main memory bus.
    pub fn execute<M: Memory>(&mut self, memory: &mut M) -> u16 {
        self.check_ppu_registers(memory);
        self.check_misc_registers(memory);

        log::log("ppu", "PPU cycle complete", &self.runtime_options);
        0
    }
}
