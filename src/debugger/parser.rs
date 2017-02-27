// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

enum ParseState {
    ScanningForArguments,
    ScanningArgument,
    ScanningQuotedArgument,
}

/// Returns true if the character passed is a whitespace character. Both spaces
/// and tabs are considered whitespace characters.
fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}

/// Returns true if the character passed is a quote.
fn is_quote(c: char) -> bool {
    c == '"'
}

/// Parses raw command-line input into a list of separate arguments. Arguments
/// are separated by whitespace, can be quoted, and can have escaped characters
/// inside of them.
///
/// The vector of strings returned can be parsed by a library such as getopts.
pub fn parse_raw_input(input: String) -> Result<Vec<String>, &'static str> {
    let mut state = ParseState::ScanningForArguments;
    let mut args: Vec<String> = Vec::new();
    let mut arg_start: usize = 0;
    let mut offset: usize = 0;

    // Iterate over each character in the input and separate different arguments
    // and put the in a list of arguments.
    for c in input.chars() {
        println!("{}", offset);

        match state {
            ParseState::ScanningForArguments => {
                // Determine a scanning state depending on the first
                // non-whitespace character and set the index of the first
                // character in the argument.
                if !is_whitespace(c) {
                    if is_quote(c) {
                        state = ParseState::ScanningQuotedArgument;
                        if offset < input.len() - 1 {
                            arg_start = offset + 1; // The start of the argument begins here.
                        } else {
                            return Err("quoted arg does not close");
                        }
                    } else {
                        state = ParseState::ScanningArgument;
                        arg_start = offset; // The start of the argument begins here.
                    }
                }
            },
            ParseState::ScanningArgument => {
                // Switch back to scanning if whitespace is encountered and push
                // the argument currently being parsed to the argument list.
                if is_whitespace(c) {
                    let arg = String::from(&input[arg_start..offset]);
                    args.push(arg);
                    arg_start = 0;
                    state = ParseState::ScanningForArguments;
                } else if offset == input.len() - 1 {
                    let arg = String::from(&input[arg_start..input.len()]);
                    args.push(arg);
                    arg_start = 0;
                    state = ParseState::ScanningForArguments;
                }
            },
            ParseState::ScanningQuotedArgument => {

            },
        }

        offset += c.to_string().len();
    }

    Ok(args)
}
