// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use nes::cpu::CPU;
use std::io::Cursor;

// Memory partition sizes (physical).
// TODO: Calculate based on ranges below.
pub const RAM_SIZE:                 usize = 0x800;
pub const PPU_CTRL_REGISTERS_SIZE:  usize = 0x8;
pub const MISC_CTRL_REGISTERS_SIZE: usize = 0x20;
pub const EXPANSION_ROM_SIZE:       usize = 0x1FE0;
pub const SRAM_SIZE:                usize = 0x2000;
pub const PRG_ROM_SIZE:             usize = 0x4000;

// Partitioned virtual memory map bounds.
pub const RAM_START_ADDR:                  usize = 0x0;
pub const RAM_END_ADDR:                    usize = 0x7FF;
pub const RAM_MIRROR_START:                usize = 0x800;
pub const RAM_MIRROR_END:                  usize = 0x1FFF;
pub const PPU_CTRL_REGISTERS_START:        usize = 0x2000;
pub const PPU_CTRL_REGISTERS_END:          usize = 0x2007;
pub const PPU_CTRL_REGISTERS_MIRROR_START: usize = 0x2008;
pub const PPU_CTRL_REGISTERS_MIRROR_END:   usize = 0x3FFF;
pub const MISC_CTRL_REGISTERS_START:       usize = 0x4000;
pub const MISC_CTRL_REGISTERS_END:         usize = 0x401F;
pub const EXPANSION_ROM_START:             usize = 0x4020;
pub const EXPANSION_ROM_END:               usize = 0x5FFF;
pub const SRAM_START:                      usize = 0x6000;
pub const SRAM_END:                        usize = 0x7FFF;
pub const PRG_ROM_1_START:                 usize = 0x8000;
pub const PRG_ROM_1_END:                   usize = 0xBFFF;
pub const PRG_ROM_2_START:                 usize = 0xC000;
pub const PRG_ROM_2_END:                   usize = 0xFFFF;

// Constants for additional structures.
pub const TRAINER_START: usize = 0x7000;
pub const TRAINER_SIZE:  usize = 512;

// Location of the DMA register for copying sprite data to the PPU.
pub const DMA_REGISTER: usize = 0x4014;

// Location of the first byte on the bottom of the stack. The stack starts on
// memory page 2 (0x100).
const STACK_OFFSET: usize = 0x100;

/// Different operation that can be performed on memory.
///
/// This enum is used with the mapping function so the PPU is informed of writes
/// to it's I/O registers over the virtual "bus".
#[derive(PartialEq)]
pub enum MemoryOperation {
    Read,
    Write,
    Nop,
}

/// Possible states of the PPU registers.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PPURegisterStatus {
    Read,
    Written,
    WrittenTwice,
    Untouched,
}

/// Possible states of the misc registers.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MiscRegisterStatus {
    Read,
    Written,
    Untouched,
}

/// Partitioned physical memory layout for CPU memory. These fields are not
/// meant to be accessed directly by the CPU implementation and are instead
/// accessed through a read function that handles memory mapping.
///
/// NOTE: Currently all memory is allocated on the stack. This may not work well
/// for systems with a small stack and slices should be boxed up.
pub struct Memory {
    // 2kB of internal RAM which contains zero page, the stack, and general
    // purpose memory.
    ram: [u8; RAM_SIZE],

    // Contains PPU registers that allow the running application to communicate
    // with the PPU.
    pub ppu_ctrl_registers: [u8; PPU_CTRL_REGISTERS_SIZE],

    // Current read / write status of all PPU registers stored in memory.
    pub ppu_ctrl_registers_status: [PPURegisterStatus; PPU_CTRL_REGISTERS_SIZE],

    // Contains NES APU and I/O registers. Also allows use of APU and I/O
    // functionality that is normally disabled.
    pub misc_ctrl_registers: [u8; MISC_CTRL_REGISTERS_SIZE],

    // Current read / write status of all misc registers stored in memory.
    pub misc_ctrl_registers_status: [MiscRegisterStatus; MISC_CTRL_REGISTERS_SIZE],

    // TODO: Add ring buffer for double write register values.

    expansion_rom: [u8; EXPANSION_ROM_SIZE],
    sram: [u8; SRAM_SIZE],

    // Read-only ROM which contains executable code and assets.
    prg_rom_1: [u8; PRG_ROM_SIZE],
    prg_rom_2: [u8; PRG_ROM_SIZE]
}

impl Memory {
    /// Returns an instance of memory with all banks initialized.
    pub fn new() -> Self {
        Memory {
            ram: [0; RAM_SIZE],
            ppu_ctrl_registers: [0; PPU_CTRL_REGISTERS_SIZE],
            ppu_ctrl_registers_status: [PPURegisterStatus::Untouched; PPU_CTRL_REGISTERS_SIZE],
            misc_ctrl_registers: [0; MISC_CTRL_REGISTERS_SIZE],
            misc_ctrl_registers_status: [MiscRegisterStatus::Untouched; MISC_CTRL_REGISTERS_SIZE],
            expansion_rom: [0; EXPANSION_ROM_SIZE],
            sram: [0; SRAM_SIZE],
            prg_rom_1: [0; PRG_ROM_SIZE],
            prg_rom_2: [0; PRG_ROM_SIZE],
        }
    }

    /// Reads an unsigned 8-bit byte value located at the given virtual address.
    #[inline(always)]
    pub fn read_u8(&mut self, addr: usize) -> u8 {
        let (bank, idx, readable, _) = self.map(addr, MemoryOperation::Read);
        if readable {
            bank[idx]
        } else {
            0
        }
    }

    /// Writes an unsigned 8-bit byte value to the given virtual address.
    #[inline(always)]
    pub fn write_u8(&mut self, addr: usize, val: u8) {
        let (bank, idx, _, writable) = self.map(addr, MemoryOperation::Write);
        if writable {
            bank[idx] = val;
        }
    }

    /// Reads an unsigned 8-bit byte value located at the given virtual address.
    #[inline(always)]
    pub fn read_u8_unrestricted(&mut self, addr: usize) -> u8 {
        let (bank, idx, _, _) = self.map(addr, MemoryOperation::Nop);
        bank[idx]
    }

    /// Writes an unsigned 8-bit byte value to the given virtual address.
    #[inline(always)]
    pub fn write_u8_unrestricted(&mut self, addr: usize, val: u8) {
        let (bank, idx, _, _) = self.map(addr, MemoryOperation::Nop);
        bank[idx] = val;
    }

    /// Reads an unsigned 16-bit byte value at the given virtual address
    /// (little-endian).
    #[inline(always)]
    pub fn read_u16(&mut self, addr: usize) -> u16 {
        // Reads two bytes starting at the given address and parses them.
        let mut reader = Cursor::new(vec![
            self.read_u8(addr),
            self.read_u8(addr + 1)
        ]);
        reader.read_u16::<LittleEndian>().unwrap()
    }

    /// Reads an unsigned 16-bit byte value at the given virtual address
    /// (little-endian).
    #[inline(always)]
    pub fn read_u16_alt(&mut self, addr: usize) -> u16 {
        // Reads two bytes starting at the given address and parses them.
        let mut reader = Cursor::new(vec![
            self.read_u8(addr - 1),
            self.read_u8(addr)
        ]);
        reader.read_u16::<LittleEndian>().unwrap()
    }

    /// Reads an unsigned 16-bit byte value at the given virtual address
    /// (little-endian) where the MSB is read at page start if the LSB is at
    /// the end of a page. This exists to properly emulate a hardware bug in the
    /// 2A03 where indirect jumps cannot fetch addresses outside it's own page.
    #[inline(always)]
    pub fn read_u16_wrapped_msb(&mut self, addr: usize) -> u16 {
        let lsb = self.read_u8(addr);
        let msb = if addr & 0xFF == 0xFF {
            self.read_u8(addr - 0xFF)
        } else {
            self.read_u8(addr + 1)
        };

        // Reads two bytes starting at the given address and parses them.
        let mut reader = Cursor::new(vec![lsb, msb]);
        reader.read_u16::<LittleEndian>().unwrap()
    }

    /// Reads an unsigned 16-bit byte value at the given virtual address
    /// (little-endian) where the MSB is read at page start if the LSB is at
    /// the end of a page. This exists to properly emulate a hardware bug in the
    /// 2A03 where indirect jumps cannot fetch addresses outside it's own page.
    #[inline(always)]
    pub fn read_u16_wrapped_msb_alt(&mut self, addr: usize) -> u16 {
        let lsb = self.read_u8(addr - 1);
        let msb = if addr & 0xFF == 0xFF {
            self.read_u8(addr - 0xFF)
        } else {
            self.read_u8(addr)
        };

        // Reads two bytes starting at the given address and parses them.
        let mut reader = Cursor::new(vec![lsb, msb]);
        reader.read_u16::<LittleEndian>().unwrap()
    }

    /// Writes an unsigned 16-bit byte value to the given virtual address
    /// (little-endian)
    #[inline(always)]
    pub fn write_u16(&mut self, addr: usize, val: u16) {
        let mut writer = vec![];
        writer.write_u16::<LittleEndian>(val).unwrap();
        self.write_u8(addr, writer[0]);
        self.write_u8(addr + 1, writer[1]);
    }

    /// Writes an unsigned 16-bit byte value to the given virtual address
    /// (little-endian)
    #[inline(always)]
    pub fn write_u16_alt(&mut self, addr: usize, val: u16) {
        let mut writer = vec![];
        writer.write_u16::<LittleEndian>(val).unwrap();
        self.write_u8(addr - 1, writer[0]);
        self.write_u8(addr, writer[1]);
    }

    /// Dumps the contents of a slice starting at a given address.
    pub fn memdump(&mut self, addr: usize, buf: &[u8]) {
        for i in 0..buf.len() {
            self.write_u8_unrestricted(addr + i, buf[i]);
        }
    }

    // Utility functions for managing the stack.

    /// Pushes an 8-bit number onto the stack.
    pub fn stack_push_u8(&mut self, cpu: &mut CPU, value: u8) {
        self.write_u8(STACK_OFFSET + cpu.sp as usize, value);
        cpu.sp = cpu.sp.wrapping_sub(1);
    }

    /// Pops an 8-bit number off the stack.
    pub fn stack_pop_u8(&mut self, cpu: &mut CPU) -> u8 {
        cpu.sp = cpu.sp.wrapping_add(1);
        self.read_u8(STACK_OFFSET + cpu.sp as usize)
    }

    /// Pushes a 16-bit number (usually an address) onto the stack.
    pub fn stack_push_u16(&mut self, cpu: &mut CPU, value: u16) {
        self.write_u16_alt(STACK_OFFSET + cpu.sp as usize, value);
        cpu.sp = cpu.sp.wrapping_sub(2);
    }

    /// Pops a 16-bit number (usually an address) off the stack.
    pub fn stack_pop_u16(&mut self, cpu: &mut CPU) -> u16 {
        cpu.sp = cpu.sp.wrapping_add(2);
        self.read_u16_alt(STACK_OFFSET + cpu.sp as usize)
    }

    /// Update the register status so the PPU knows which registers were touched
    /// after the memory operation. Reads to registers marked in any written
    /// state do not override the written flag for that register.
    ///
    /// In the event that the PPU register has already been written to and is
    /// being written to again, set the status to WrittenTwice.
    fn update_ppu_register_status(&mut self, addr: usize, operation: MemoryOperation) {
        let registers_status = &mut self.ppu_ctrl_registers_status;
        registers_status[addr] = if registers_status[addr] == PPURegisterStatus::Written && operation == MemoryOperation::Write {
                PPURegisterStatus::WrittenTwice
        } else if registers_status[addr] != PPURegisterStatus::Written && registers_status[addr] != PPURegisterStatus::WrittenTwice {
            match operation {
                MemoryOperation::Read  => PPURegisterStatus::Read,
                MemoryOperation::Write => PPURegisterStatus::Written,
                MemoryOperation::Nop   => registers_status[addr],
            }
        } else {
            registers_status[addr]
        };
    }

    /// Returns PPU register read/write permissions for use with the I/O
    /// functions. Register status is also updated depending on the operation.
    fn map_ppu_registers(&mut self, addr: usize, operation: MemoryOperation) -> (&mut [u8], usize, bool, bool) {
        self.update_ppu_register_status(addr, operation);

        let registers = &mut self.ppu_ctrl_registers;
        match addr {
            0 => (registers, addr, false, true),
            1 => (registers, addr, false, true),
            2 => (registers, addr, true, false),
            3 => (registers, addr, false, true),
            4 => (registers, addr, true, true),
            5 => (registers, addr, false, true), // Twice
            6 => (registers, addr, false, true), // Twice
            7 => (registers, addr, true, true),
            _ => (registers, addr, true, true),
        }
    }

    /// Update the register status so the we know which registers were touched
    /// after the memory operation. Reads to registers marked in any written
    /// state do not override the written flag for that register.
    ///
    /// In the event that the PPU register has already been written to and is
    /// being written to again, set the status to WrittenTwice.
    fn update_misc_register_status(&mut self, addr: usize, operation: MemoryOperation) {
        let registers_status = &mut self.misc_ctrl_registers_status;
        registers_status[addr] = match operation {
            MemoryOperation::Read  => MiscRegisterStatus::Read,
            MemoryOperation::Write => MiscRegisterStatus::Written,
            MemoryOperation::Nop   => registers_status[addr],
        };
    }

    /// Returns misc register read/write permissions for use with the I/O
    /// functions. Register status is also updated depending on the operation.
    fn map_misc_registers(&mut self, addr: usize, operation: MemoryOperation) -> (&mut [u8], usize, bool, bool) {
        self.update_misc_register_status(addr, operation);

        // FIXME: Double-check permissions on these I/O registers.
        let registers = &mut self.misc_ctrl_registers;
        match addr {
            0x14 => (registers, addr, false, true),
            _    => (registers, addr, true, true),
        }
    }

    /// Maps a given virtual address to a physical address internal to the
    /// emulator. Returns a memory buffer and index for physical memory access.
    ///
    /// TODO: Switch all references to struct members to functions so this
    /// mapper implementation can be shared between ROM mappers.
    fn map(&mut self, addr: usize, operation: MemoryOperation) -> (&mut [u8], usize, bool, bool) {
        match addr {
            RAM_START_ADDR...RAM_END_ADDR =>
                (&mut self.ram, addr, true, true),
            RAM_MIRROR_START...RAM_MIRROR_END =>
                (&mut self.ram, addr % RAM_SIZE, true, true),
            PPU_CTRL_REGISTERS_START...PPU_CTRL_REGISTERS_END =>
                self.map_ppu_registers(addr - PPU_CTRL_REGISTERS_START, operation),
            PPU_CTRL_REGISTERS_MIRROR_START...PPU_CTRL_REGISTERS_MIRROR_END =>
                self.map_ppu_registers((addr - PPU_CTRL_REGISTERS_START) % PPU_CTRL_REGISTERS_SIZE, operation),
            MISC_CTRL_REGISTERS_START...MISC_CTRL_REGISTERS_END =>
                self.map_misc_registers(addr - MISC_CTRL_REGISTERS_START, operation),
            EXPANSION_ROM_START...EXPANSION_ROM_END =>
                (&mut self.expansion_rom, addr - EXPANSION_ROM_START, true, false),
            SRAM_START...SRAM_END =>
                (&mut self.sram, addr - SRAM_START, true, true),
            PRG_ROM_1_START...PRG_ROM_1_END =>
                (&mut self.prg_rom_1, addr - PRG_ROM_1_START, true, false),
            PRG_ROM_2_START...PRG_ROM_2_END =>
                (&mut self.prg_rom_2, addr - PRG_ROM_2_START, true, false),
            _ => { panic!("Unable to map virtual address {:#X} to any physical address", addr) },
        }
    }
}
