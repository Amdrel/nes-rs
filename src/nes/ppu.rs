// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use io::log;
use nes::memory::Memory;
use nes::nes::NESRuntimeOptions;

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
    pub fn new(runtime_options: NESRuntimeOptions) -> Self {
        PPU {
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

    pub fn execute<M: Memory>(&mut self, memory: &mut M) {
        {
            let (bank, addr) = self.map(NAME_TABLES_START-1);
            println!("{:04X}", addr);
        }

        log::log("ppu", "PPU cycle complete", &self.runtime_options);
    }
}
