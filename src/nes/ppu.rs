// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

const PATTERN_TABLES_SIZE: usize = 0x2000;
const NAME_TABLES_SIZE:    usize = 0x1F00;
const PALETTES_SIZE:       usize = 0x0100;
const SPR_RAM_SIZE:        usize = 0x00FF;

/// This is an implementation of the 2C02 PPU used in the NES. This piece of
/// hardware is responsible for drawing graphics to the television the console
/// is hooked up to; however in our case we draw to an SDL surface.
pub struct PPU {
    // TODO: Document these memory banks after grokking documentation.
    pattern_tables: [u8; PATTERN_TABLES_SIZE],
    name_tables: [u8; NAME_TABLES_SIZE],
    palettes: [u8; PALETTES_SIZE],
    spr_ram: [u8; SPR_RAM_SIZE],
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            pattern_tables: [0; PATTERN_TABLES_SIZE],
            name_tables: [0; NAME_TABLES_SIZE],
            palettes: [0; PALETTES_SIZE],
            spr_ram: [0; SPR_RAM_SIZE],
        }
    }
}
