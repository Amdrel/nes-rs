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

/// Reads a binary file at a given path and stores it in a vector of bytes.
pub fn read_bin<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Error> {
    let mut buffer: Vec<u8> = Vec::new();
    match File::open(path) {
        Ok(mut file) => {
            file.read_to_end(&mut buffer).unwrap();
        },
        Err(e) => return Err(e)
    }
    Ok(buffer)
}

/// Parses the header of a rom (assumed to be in the iNES format).
///
/// The first 16 bytes of the rom contain the header. The iNES format is
/// identified by the literal byte string "NES<0x1A>". If the rom is not in the
/// iNES format, then it cannot be executed by the emulator.
pub fn parse_rom_header(rom: &[u8]) -> Result<(), &str> {
    // Validate that the rom is formatted in the iNES format.
    if &rom[0x0..0x4] != [0x4E, 0x45, 0x53, 0x1A] {
        return Err("rom does not contain iNES identifier and is invalid")
    }
    Ok(())
}
