// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

const UNCLOSING_QUOTE: &'static str = "quoted arg does not close";

/// Converts raw command-line input into a list of separate arguments. The vector
/// of strings returned can be parsed by a library such as getopts.
pub fn input_to_arguments(input: String) -> Result<Vec<String>, &'static str> {
    let mut args: Vec<String> = Vec::new();
    let mut start: usize = 0;
    let mut end: usize = 0;
    let mut last_was_whitespace = false;

    for c in input.chars() {
        let charlen = c.to_string().len();

        if is_whitespace(c) {
            if !last_was_whitespace {
                args.push(String::from(&input[start..end]));
            }

            end += charlen;
            start = end;
            last_was_whitespace = true;
        } else {
            end += charlen;
            last_was_whitespace = false;
        }
    }

    if start != end {
        args.push(String::from(&input[start..end]));
    }

    Ok(args)
}

/// Returns true if the character passed is a whitespace character. Both spaces
/// and tabs are considered whitespace characters.
fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}
