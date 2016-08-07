// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fs::File;
use std::io::Read;
use std::io::Result;
use std::path::Path;

/// Reads a binary file at a given path and stores it in a vector of bytes.
pub fn read_bin<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::new();
    match File::open(path) {
        Ok(mut file) => {
            file.read_to_end(&mut buffer).unwrap();
        },
        Err(e) => return Err(e)
    }

    Ok(buffer)
}
