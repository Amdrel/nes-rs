// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use io::log;
use nes::nes::NESRuntimeOptions;

const PATTERN_TABLES_SIZE: usize = 0x2000;
const NAME_TABLES_SIZE:    usize = 0x1F00;
const PALETTES_SIZE:       usize = 0x0100;
const SPR_RAM_SIZE:        usize = 0x00FF;

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

    pub fn execute(&self) {
        log::log("ppu", "PPU cycle complete", &self.runtime_options);
    }
}
