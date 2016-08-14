// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

// Memory partition sizes (physical).
// TODO: Calculate based on ranges below.
pub const RAM_SIZE                : usize = 0x800;
pub const PPU_CTRL_REGISTERS_SIZE : usize = 0x8;
pub const MISC_CTRL_REGISTERS_SIZE: usize = 0x20;
pub const EXPANSION_ROM_SIZE      : usize = 0x1FE0;
pub const SRAM_SIZE               : usize = 0x2000;
pub const PRG_ROM_SIZE            : usize = 0x4000;

// Partitioned virtual memory map bounds.
pub const RAM_START_ADDR                 : usize = 0x0;
pub const RAM_END_ADDR                   : usize = 0x7FF;
pub const RAM_MIRROR_START               : usize = 0x800;
pub const RAM_MIRROR_END                 : usize = 0x1FFF;
pub const PPU_CTRL_REGISTERS_START       : usize = 0x2000;
pub const PPU_CTRL_REGISTERS_END         : usize = 0x2007;
pub const PPU_CTRL_REGISTERS_MIRROR_START: usize = 0x2008;
pub const PPU_CTRL_REGISTERS_MIRROR_END  : usize = 0x3FFF;
pub const MISC_CTRL_REGISTERS_START      : usize = 0x4000;
pub const MISC_CTRL_REGISTERS_END        : usize = 0x401F;
pub const EXPANSION_ROM_START            : usize = 0x4020;
pub const EXPANSION_ROM_END              : usize = 0x5FFF;
pub const SRAM_START                     : usize = 0x6000;
pub const SRAM_END                       : usize = 0x7FFF;
pub const PRG_ROM_1_START                : usize = 0x8000;
pub const PRG_ROM_1_END                  : usize = 0xBFFF;
pub const PRG_ROM_2_START                : usize = 0xC000;
pub const PRG_ROM_2_END                  : usize = 0xFFFF;

// Constants for additional structures.
pub const TRAINER_START: usize = 0x7000;
pub const TRAINER_SIZE : usize = 512;

/// Partitioned physical memory layout for CPU memory. These fields are not
/// meant to be accessed directly by the CPU implementation and are instead
/// accessed through a read function that handles memory mapping.
///
/// NOTE: Currently all memory is allocated on the stack. This may not work well
/// for systems with a small stack and slices should be boxed up.
pub struct Memory {
    // 2kB of internal RAM for which it's use is entirely up to the programmer.
    ram: [u8; RAM_SIZE],

    // Contains PPU registers that allow the running application to communicate
    // with the PPU.
    ppu_ctrl_registers: [u8; PPU_CTRL_REGISTERS_SIZE],

    // Contains NES APU and I/O registers. Also allows use of APU and I/O
    // functionality that is normally disabled.
    misc_ctrl_registers: [u8; MISC_CTRL_REGISTERS_SIZE],

    expansion_rom: [u8; EXPANSION_ROM_SIZE],

    // 8kB of static RAM.
    sram: [u8; SRAM_SIZE],

    // PRG-ROM bank 1.
    prg_rom_1: [u8; PRG_ROM_SIZE],

    // PRG-ROM bank 2. Execution starts here.
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

    /// Reads an unsigned 8-bit byte value located at the given virtual address.
    pub fn read_u8(&mut self, addr: usize) -> u8 {
        let (bank, idx) = self.map(addr);
        bank[idx]
    }

    /// Writes an unsigned 8-bit byte value to the given virtual address.
    pub fn write_u8(&mut self, addr: usize, val: u8) {
        let (bank, idx) = self.map(addr);
        bank[idx] = val;
    }

    /// Reads an unsigned 16-bit byte value at the given virtual address
    /// (little-endian).
    pub fn read_u16(&mut self, addr: usize) -> u16 {
        // Reads two bytes starting at the given address and parses them.
        let mut reader = Cursor::new(vec![
            self.read_u8(addr),
            self.read_u8(addr + 1)
        ]);
        reader.read_u16::<LittleEndian>().unwrap()
    }

    /// Writes an unsigned 16-bit byte value to the given virtual address
    /// (little-endian)
    pub fn write_u16(&mut self, addr: usize, val: u16) {
        let mut writer = vec![];
        writer.write_u16::<LittleEndian>(val).unwrap();
        self.write_u8(addr, writer[0]);
        self.write_u8(addr + 1, writer[1]);
    }

    /// Dumps the contents of a slice starting at a given address.
    pub fn memdump(&mut self, addr: usize, buf: &[u8]) {
        for i in 0..buf.len() {
            self.write_u8(addr + i, buf[i]);
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
    ///
    /// TODO: Add permissions flag for special addresses?
    fn map(&mut self, addr: usize) -> (&mut [u8], usize) {
        // Address translation for accessing system memory. No modifications
        // need to be done to the address as it's at the start of addressable
        // memory.
        if self.addr_in_range(addr, RAM_START_ADDR, RAM_END_ADDR) {
            return (&mut self.ram, addr)
        }

        // Address translation for mirroring of system memory. System memory at
        // $0000-$07FF is mirrored at $0800-$0FFF, $1000-$17FF, and $1800-$1FFF
        // - attempting to access memory at, for example, $0173 is the same as
        // accessing memory at $0973, $1173, or $1973.
        if self.addr_in_range(addr, RAM_MIRROR_START, RAM_MIRROR_END) {
            let new_addr = addr % RAM_SIZE;
            return (&mut self.ram, new_addr)
        }

        // Address translation for accessing the PPU control registers.
        if self.addr_in_range(addr, PPU_CTRL_REGISTERS_START,
                              PPU_CTRL_REGISTERS_END) {
            let new_addr = addr - PPU_CTRL_REGISTERS_START;
            return (&mut self.ppu_ctrl_registers, new_addr)
        }

        // Address translation for mirroring of the PPU control registers. PPU
        // control at $2000-$2007 is mirrored 1023 times at $2008-$3FFF.
        if self.addr_in_range(addr, PPU_CTRL_REGISTERS_MIRROR_START,
                              PPU_CTRL_REGISTERS_MIRROR_END) {
            let new_addr = (addr - PPU_CTRL_REGISTERS_START) %
                PPU_CTRL_REGISTERS_SIZE;
            return (&mut self.ppu_ctrl_registers, new_addr)
        }

        // Address translation for miscellaneous registers (such as APU etc).
        if self.addr_in_range(addr, MISC_CTRL_REGISTERS_START,
                              MISC_CTRL_REGISTERS_END) {
            let new_addr = addr - MISC_CTRL_REGISTERS_START;
            return (&mut self.misc_ctrl_registers, new_addr)
        }

        // Address translation for accessing cartridge expansion ROM.
        if self.addr_in_range(addr, EXPANSION_ROM_START, EXPANSION_ROM_END) {
            let new_addr = addr - EXPANSION_ROM_START;
            return (&mut self.expansion_rom, new_addr)
        }

        // Address translation for accessing SRAM.
        if self.addr_in_range(addr, SRAM_START, SRAM_END) {
            let new_addr = addr - SRAM_START;
            return (&mut self.sram, new_addr)
        }

        // Address translation for accessing the program rom (bank 1).
        if self.addr_in_range(addr, PRG_ROM_1_START, PRG_ROM_1_END) {
            let new_addr = addr - PRG_ROM_1_START;
            return (&mut self.prg_rom_1, new_addr)
        }

        // Address translation for accessing the program rom (bank 2).
        if self.addr_in_range(addr, PRG_ROM_2_START, PRG_ROM_2_END) {
            let new_addr = addr - PRG_ROM_2_START;
            return (&mut self.prg_rom_2, new_addr)
        }

        panic!("Unable to map virtual address {:#X} to any physical address",
               addr);
    }
}
