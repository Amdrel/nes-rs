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

/// Structure that represents the 16 byte header of an iNES rom. Only missing
/// the zero fill as it's unused space.
#[derive(Debug)]
pub struct INESHeader {
    identifier: [u8; 4], // File format identifier.
    prg_rom_size: u8,    // Size of PRG ROM in 16 KB units.
    chr_rom_size: u8,    // Size of CHR ROM in 8 KB units.
    flags_6: u8,
    flags_7: u8,
    prg_ram_size: u8,    // Size of PRG RAM in 8 KB units (0 infers 8 KB for
                         // compatibility).
    flags_9: u8,
    flags_10: u8         // Unofficial, unused by most emulators.
}

/// Reads a binary file at a given path and stores it in a vector of bytes.
pub fn read_bin<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Error> {
    let mut buffer: Vec<u8> = Vec::new();
    let mut file = try!(File::open(path));
    try!(file.read_to_end(&mut buffer));
    Ok(buffer)
}

/// Parses the header of a rom (assumed to be in the iNES format).
///
/// The first 16 bytes of the rom contain the header. The iNES format is
/// identified by the literal byte string "NES<0x1A>". If the rom is not in the
/// iNES format, then it cannot be executed by the emulator.
pub fn parse_rom_header(rom: &[u8]) -> Result<INESHeader, &str> {
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
