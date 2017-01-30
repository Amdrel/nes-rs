// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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

// Relative addresses of I/O registers handled by the PPU.
const PPUCTRL:    usize = 0x00;
const PPUMASK:    usize = 0x01;
const PPUSTATUS:  usize = 0x02;
const OAMADDR:    usize = 0x03;
const OAMDATA:    usize = 0x04;
const PPUSCROLL:  usize = 0x05;
const PPUADDR:    usize = 0x06;
const PPUDATA:    usize = 0x07;
const OAMDMA:     usize = 0x14;

// Initial register values set at startup.
const INITIAL_PPUCTRL:   u8 = 0b00000000;
const INITIAL_PPUMASK:   u8 = 0b00000000;
const INITIAL_PPUSTATUS: u8 = 0b10100000;
const INITIAL_OAMADDR:   u8 = 0b00000000;
const INITIAL_OAMDATA:   u8 = 0b00000000;
const INITIAL_PPUSCROLL: u8 = 0b00000000;
const INITIAL_PPUADDR:   u8 = 0b00000000;
const INITIAL_PPUDATA:   u8 = 0b00000000;
const INITIAL_OAMDMA:    u8 = 0b00000000;

/// This is an implementation of the 2C02 PPU used in the NES. This piece of
/// hardware is responsible for drawing graphics to the television the console
/// is hooked up to; however in our case we draw to an SDL surface.
///
/// Some comments pertaining to PPU functionality are courtesy of
/// wiki.nesdev.com.
pub struct PPU {
    // Contains various flags used for controlling PPU operation.
    ppu_ctrl: u8,

    // This register controls the rendering of sprites and backgrounds, as well
    // as color effects.
    ppu_mask: u8,

    // This register reflects the state of various functions inside the PPU. It
    // is often used for determining timing.
    ppu_status: u8,

    // Address where OAM starts.
    oam_address: u8,

    // Data to be written to the address of OAMADDR next tick.
    oam_data: u8,

    ppu_scroll: u8,
    ppu_addr: u8,
    ppu_data: u8,

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
            ppu_ctrl: INITIAL_PPUCTRL,
            ppu_mask: INITIAL_PPUMASK,
            ppu_status: INITIAL_PPUSTATUS,
            oam_address: INITIAL_OAMADDR,
            oam_data: INITIAL_OAMDATA,
            ppu_scroll: INITIAL_PPUSCROLL,
            ppu_addr: INITIAL_PPUADDR,
            ppu_data: INITIAL_PPUDATA,
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
    /// TODO: Implement me!
    fn exec_dma(&mut self, register: u8) {
        println!("{:02X}", register);
        panic!("DMA unimplemented");
    }

    /// Reads the contents of the DMA register and executes DMA if written since
    /// the last PPU cycle.
    /// TODO: Implement me!
    fn handle_dma_register(&mut self, index: usize, memory: &mut Memory) {
        let state = memory.misc_ctrl_registers_status[index];
        if state != MiscRegisterStatus::Written {
            return;
        }
        let register = memory.misc_ctrl_registers[index];
        self.exec_dma(register);
    }

    /// Updates the internal PPUCTRL register when the I/O register was written
    /// since the last PPU cycle.
    /// FIXME: Make accurate.
    fn handle_ppu_ctrl(&mut self, index: usize, memory: &mut Memory) {
        let state = memory.ppu_ctrl_registers_status[index];
        if state != PPURegisterStatus::Written || state != PPURegisterStatus::WrittenTwice {
            return;
        }
        self.ppu_ctrl = memory.ppu_ctrl_registers[index];
        memory.ppu_ctrl_registers_status[index] = PPURegisterStatus::Untouched;
    }

    /// Updates the internal PPUMASK register when the I/O register was written
    /// since the last PPU cycle.
    /// FIXME: Make accurate.
    fn handle_ppu_mask(&mut self, index: usize, memory: &mut Memory) {
        let state = memory.ppu_ctrl_registers_status[index];
        if state != PPURegisterStatus::Written || state != PPURegisterStatus::WrittenTwice {
            return;
        }
        self.ppu_mask = memory.ppu_ctrl_registers[index];
        memory.ppu_ctrl_registers_status[index] = PPURegisterStatus::Untouched;
    }

    /// Updates the internal OAMADDR registers with data in the I/O register.
    /// FIXME: Make accurate.
    fn handle_oam_addr(&mut self, index: usize, memory: &mut Memory) {
        let state = memory.ppu_ctrl_registers_status[index];
        if state != PPURegisterStatus::Written || state != PPURegisterStatus::WrittenTwice {
            return;
        }
        self.oam_address = memory.ppu_ctrl_registers[index];
        memory.ppu_ctrl_registers_status[index] = PPURegisterStatus::Untouched;

        panic!("Implement OAMADDR write handling");
    }

    /// Updates the internal OAMADDR registers with data in the I/O register.
    /// FIXME: Make accurate.
    fn handle_oam_data(&mut self, index: usize, memory: &mut Memory) {
        let state = memory.ppu_ctrl_registers_status[index];
        if state != PPURegisterStatus::Written || state != PPURegisterStatus::WrittenTwice {
            return;
        }
        self.oam_data = memory.ppu_ctrl_registers[index];
        self.oam_address = self.oam_address.wrapping_add(1);
        memory.ppu_ctrl_registers_status[index] = PPURegisterStatus::Untouched;

        panic!("Implement OAMDATA write handling");
    }

    /// FIXME: Make accurate.
    fn handle_ppu_scroll(&mut self, index: usize, memory: &mut Memory) {
        let state = memory.ppu_ctrl_registers_status[index];
        if state != PPURegisterStatus::WrittenTwice {
            return;
        }
        panic!("Implement PPUSCROLL write handling");
    }

    /// FIXME: Make accurate.
    fn handle_ppu_address(&mut self, index: usize, memory: &mut Memory) {
        let state = memory.ppu_ctrl_registers_status[index];
        if state != PPURegisterStatus::WrittenTwice {
            return;
        }
        panic!("Implement PPUADDR write handling");
    }

    /// FIXME: Make accurate.
    fn handle_ppu_data(&mut self, index: usize, memory: &mut Memory) {
        let state = memory.ppu_ctrl_registers_status[index];
        if state != PPURegisterStatus::Written || state != PPURegisterStatus::WrittenTwice {
            return;
        }
        panic!("Implement PPUDATA write handling");
    }

    /// Checks the status of PPU I/O registers and executes PPU functionality
    /// depending on their states.
    fn check_ppu_registers(&mut self, memory: &mut Memory) {
        for index in 0x0..0x8 {
            match index {
                PPUCTRL   => self.handle_ppu_ctrl(index, memory),
                PPUMASK   => self.handle_ppu_mask(index, memory),
                PPUSTATUS => {},
                OAMADDR   => self.handle_oam_addr(index, memory),
                OAMDATA   => self.handle_oam_data(index, memory),
                PPUSCROLL => self.handle_ppu_scroll(index, memory),
                PPUADDR   => self.handle_ppu_address(index, memory),
                PPUDATA   => self.handle_ppu_data(index, memory),

                _ => {
                    if memory.ppu_ctrl_registers_status[index] != PPURegisterStatus::Untouched {
                        panic!("Unsupported ppu register touched: 0x{:02X}", index);
                    }
                },
            }
        }
    }

    /// Checks the status of misc I/O registers and executes PPU functionality
    /// depending on their states.
    fn check_misc_registers(&mut self, memory: &mut Memory) {
        for index in 0x0..0x20 {
            match index {
                OAMDMA => self.handle_dma_register(index, memory),

                // FIXME: PPU does not need to handle all misc I/O registers.
                // Remove this panic later.
                _ => {
                    if memory.misc_ctrl_registers_status[index] != MiscRegisterStatus::Untouched {
                        panic!("Unsupported misc register touched: 0x{:02X}", index);
                    }
                },
            };
        }
    }

    /// Executes routine PPU logic and returns stolen cycles from operations
    /// such as DMA transfers if the PPU hogged the main memory bus.
    pub fn step(&mut self, memory: &mut Memory) -> u16 {
        // Check the dirty state of each of the I/O registers used by the PPU.
        self.check_ppu_registers(memory);
        self.check_misc_registers(memory);

        0 // TODO: Throw in DMA cycles.
    }
}
