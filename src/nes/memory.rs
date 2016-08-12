// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Memory partition sizes (physical).
const RAM_SIZE: usize = 0x800;
const PPU_CTRL_REGISTERS_SIZE: usize = 0x8;
const MISC_CTRL_REGISTERS_SIZE: usize = 0x20;
const EXPANSION_ROM_SIZE: usize = 0x1FDF;
const SRAM_SIZE: usize = 0x2000;
const PRG_ROM_SIZE: usize = 0x4000;

// Virtual memory map bounds.
const RAM_START_ADDR: usize = 0x0;
const RAM_END_ADDR: usize = 0x7FF;

/// Partitioned physical memory layout for CPU memory. These fields are not
/// meant to be accessed directly by the cpu implementation and are instead
/// accessed through a read function that handles memory mapping.
pub struct Memory {
    ram: [u8; RAM_SIZE],
    ppu_ctrl_registers: [u8; PPU_CTRL_REGISTERS_SIZE],
    misc_ctrl_registers: [u8; MISC_CTRL_REGISTERS_SIZE],
    expansion_rom: [u8; EXPANSION_ROM_SIZE],
    sram: [u8; SRAM_SIZE],
    prg_rom_1: [u8; PRG_ROM_SIZE],
    prg_rom_2: [u8; PRG_ROM_SIZE]
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            ram: [0; RAM_SIZE],
            ppu_ctrl_registers: [0; PPU_CTRL_REGISTERS_SIZE],
            misc_ctrl_registers: [0; MISC_CTRL_REGISTERS_SIZE],
            expansion_rom: [0; EXPANSION_ROM_SIZE],
            sram: [0; SRAM_SIZE],
            prg_rom_1: [0; PRG_ROM_SIZE],
            prg_rom_2: [0; PRG_ROM_SIZE]
        }
    }

    /// Returns true when the provided address is in the provided range
    /// (inclusive).
    ///
    /// NOTE: Inclusive ranges are in unstable rust and this function can be
    /// replaced once it lands in stable (RFC 1192).
    fn addr_in_range(&self, addr: usize, lower: usize, upper: usize) -> bool {
        addr >= lower && addr <= upper
    }

    /// Maps a given virtual address to a physical address internal to the
    /// emulator. Returns a memory buffer and index for physical memory access.
    pub fn map(&mut self, addr: usize) -> (&mut [u8], usize) {
        // Work ram memory mapping.
        if self.addr_in_range(addr, RAM_START_ADDR, RAM_END_ADDR) {
            return (&mut self.ram, addr)
        }

        panic!("Unable to map virtual address {:#X} to any physical address", addr);
    }
}
