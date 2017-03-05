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

/// Parses raw command-line input into a list of separate arguments. Arguments
/// are separated by whitespace, can be quoted, and can have escaped characters
/// inside of them.
///
/// The vector of strings returned can be parsed by a library such as getopts.
pub fn parse_raw_input(input: String) -> Result<Vec<String>, &'static str> {
    let mut args: Vec<String> = Vec::new(); // Output genned from input.
    let mut control_active = false;         // True when on control code.

    // This parser uses a basic state machine that can be in a few different
    // states. The parser can be searching for new arguments, stepping through
    // an argument, or stepping through a quoted argument.
    let mut state = ParseState::ScanningForArguments;

    // Argument parsing offsets used when extracting arguments out of the input
    // string. These values represent binary offsets rather than character
    // indices as unicode values can vary in length.
    let mut arg_start: usize = 0;
    let mut offset: usize = 0;

    // Iterate over each character in the input and separate different arguments
    // and put the in a list of arguments. The index from the iterator is not
    // used since it's not a binary offset of the character currently being
    // parsed.
    for c in input.chars() {
        match state {
            // Determine a scanning state depending on the first non-whitespace
            // character encountered and set the index of the first character in
            // the argument.
            ParseState::ScanningForArguments => {
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

            // Scans until a whitespace character or the end of the input is
            // reached. Once whitespace is encountered, the argument currently
            // being parsed is pushed to the argument list.
            ParseState::ScanningArgument => {
                let mut ignore_next = false;
                if is_control_char(c) && !control_active {
                    control_active = true;
                    ignore_next = true;
                } else if is_control_char(c) {
                    ignore_next = false;
                    control_active = false;
                }

                if is_whitespace(c) && !ignore_next {
                    if control_active {
                        control_active = false;
                    } else {
                        let arg = String::from(&input[arg_start..offset]);
                        args.push(arg);
                        arg_start = 0;
                        state = ParseState::ScanningForArguments;
                    }
                }
                if offset == input.len() - 1 {
                    let arg = String::from(&input[arg_start..input.len()]);
                    args.push(arg);
                    arg_start = 0;
                    state = ParseState::ScanningForArguments;
                }
            },

            // Scan until a non escaped quote is reached. Once encountered, the
            // argument currently being parsed is pushed to the argument list.
            ParseState::ScanningQuotedArgument => {
                let mut ignore_next = false;
                if is_control_char(c) && !control_active {
                    control_active = true;
                    ignore_next = true;
                } else if is_control_char(c) {
                    ignore_next = false;
                    control_active = false;
                }

                if is_quote(c) && !ignore_next {
                    if control_active {
                        control_active = false;
                    } else {
                        let arg = String::from(&input[arg_start..offset]);
                        args.push(arg);
                        arg_start = 0;
                        state = ParseState::ScanningForArguments;
                    }
                } else if offset == input.len() - 1 {
                    return Err("quoted arg does not close");
                }
            },
        }
        offset += c.to_string().len();
    }

    Ok(args)
}

/// Returns true if the character passed is a control character.
fn is_control_char(c: char) -> bool {
    c == '\\'
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
