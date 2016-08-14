// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::path::Path;
use std::result::Result;

// Used to identify a rom as being in the iNES format. This byte sequence should
// be at the start of every rom.
const INES_IDENTIFIER: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

const MIRROR_TYPE    : u8 = 0x1;
const PERSISTENT_FLAG: u8 = 0x2;
const TRAINER_FLAG   : u8 = 0x4;
const MIRROR_4_SCREEN: u8 = 0x8;
const MAPPER_NUMBER  : u8 = 0xF0;

#[derive(Debug)]
pub enum MirrorType {
    Horizontal,
    Vertical,
    Both
}

#[derive(Debug)]
pub enum Mapper {
    NROM
}

/// Structure that represents the 16 byte header of an iNES rom. Only missing
/// the zero fill as it's unused space.
#[derive(Debug)]
pub struct INESHeader {
    // File format identifier for the iNES format.
    pub identifier: [u8; 4],

    // Size of PRG ROM in 16 KB units.
    pub prg_rom_size: u8,

    // Size of CHR ROM in 8 KB units.
    pub chr_rom_size: u8,

    // Size of PRG RAM in 8 KB units (0 infers 8 KB for compatibility).
    pub prg_ram_size: u8,

    flags_6: u8,
    flags_7: u8,
    flags_9: u8,
    flags_10: u8 // Unofficial, unused by most emulators.
}

impl INESHeader {
    /// Parses the header of a rom (assumed to be in the iNES format).
    ///
    /// The first 16 bytes of the rom contain the header. The iNES format is
    /// identified by the literal byte string "NES<0x1A>". If the rom is not in the
    /// iNES format, then it cannot be executed by the emulator.
    pub fn new(rom: &[u8]) -> Result<INESHeader, &str> {
        // The header takes at least 0x10 bytes of space at the start of the rom.
        let invalid_header = "rom does not contain iNES identifier and is invalid";
        if rom.len() < 0x10 {
            return Err(invalid_header)
        }

        // Validate that the rom is formatted in the iNES format.
        let identifier = &rom[0x0..0x4];
        if identifier != INES_IDENTIFIER {
            return Err(invalid_header)
        }

        // Copy the identifier from the rom for placement in the header.
        let mut new_identifier: [u8; 4] = [0; 4];
        new_identifier.copy_from_slice(identifier);

        // Return an iNES header containing fields filled in from the rom.
        Ok(INESHeader {
            identifier: new_identifier,
            prg_rom_size: rom[0x4],
            chr_rom_size: rom[0x5],
            flags_6: rom[0x6],
            flags_7: rom[0x7],
            prg_ram_size: rom[0x8],
            flags_9: rom[0x9],
            flags_10: rom[0xA]
        })
    }

    /// Returns mirroring type used by the ROM.
    #[inline(always)]
    pub fn mirror_type(&self) -> MirrorType {
        if self.flags_6 & MIRROR_4_SCREEN == MIRROR_4_SCREEN {
            return MirrorType::Both
        } else if self.flags_6 & MIRROR_TYPE == MIRROR_TYPE {
            return MirrorType::Vertical
        } else {
            return MirrorType::Horizontal
        }
    }

    /// Returns true if persistent RAM is used by the ROM.
    #[inline(always)]
    pub fn has_persistent_ram(&self) -> bool {
        self.flags_6 & PERSISTENT_FLAG == PERSISTENT_FLAG
    }

    /// Returns true if there is trainer data inside the ROM.
    #[inline(always)]
    pub fn has_trainer(&self) -> bool {
        self.flags_6 & TRAINER_FLAG == TRAINER_FLAG
    }

    /// Returns the mapper number that signifies which mapper is in use by the
    /// cartridge. The lower nybble is stored in bits 4-7 in flag 6 while the
    /// upper nybble is stored in bits 4-7 in flag 7 (same bitmask). The results
    /// are then OR'd together to create the final 8-bit number.
    #[inline(always)]
    pub fn mapper(&self) -> Mapper {
        let lower = (self.flags_6 & MAPPER_NUMBER) >> 4;
        let upper = self.flags_7 & MAPPER_NUMBER;
        let mapper = lower | upper;

        match mapper {
            0 => Mapper::NROM,
            _ => {
                panic!("ROM uses unimplemented mapper: {}", mapper);
            }
        }
    }
}

/// Reads a binary file at a given path and stores it in a vector of bytes.
pub fn read_bin<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Error> {
    let mut buffer: Vec<u8> = Vec::new();
    let mut file = try!(File::open(path));
    try!(file.read_to_end(&mut buffer));
    Ok(buffer)
}
