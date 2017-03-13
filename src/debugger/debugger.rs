// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use debugger::parser;
use nes::nes::NES;
use std::sync::mpsc::Receiver;
use std::io::{self, Write};

enum Command {
    Stop,
    Continue,
    Dump,
}

pub struct Debugger {
    receiver: Receiver<String>,
    stepping: bool,
}

impl Debugger {
    pub fn new(receiver: Receiver<String>) -> Self {
        Debugger {
            receiver: receiver,
            stepping: true,
        }
    }

    pub fn step(&mut self, nes: &mut NES) {
        match self.receiver.try_recv() {
            Ok(input) => {
                self.interpret(input);
            },
            Err(e) => {}, // Ignore empty and disconnect errors.
        };

        if self.stepping {
            nes.step();
        }
    }

    fn interpret(&self, input: String) {
        let args = match parser::input_to_arguments(input) {
            Ok(args) => args,
            Err(e) => {
                let mut stderr = io::stderr();
                writeln!(stderr, "nes-rs: {}", e).unwrap();
                return;
            },
        };
        println!("{:?}", args);
    }
}
