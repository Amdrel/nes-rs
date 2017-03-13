// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use debugger::parser;
use nes::nes::NES;
use std::io::{self, Write};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
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

    /// Steps the CPU forward a single instruction, as well as executing any PPU
    /// and sound functionality that happens in-between.
    pub fn step(&mut self, nes: &mut NES) {
        // Input is received from another thread so the emulator can run without
        // the debugger prompt blocking it.
        match self.receiver.try_recv() {
            Ok(input) => self.interpret(input),
            Err(e) => {}, // Ignore empty and disconnect errors.
        };

        // If the debugger is in stepping mode, continue execution like normal,
        // otherwise the CPU and other peripherals should not update. In the
        // meantime, sleep the host CPU while we wait for input.
        if self.stepping {
            nes.step();
        } else {
            thread::sleep(Duration::from_millis(16));
        }
    }

    /// Parse a raw input string into a list of arguments and a command, then
    /// execute the command's functionality.
    fn interpret(&mut self, input: String) {
        // Parse the input from stdin and receive an array of arguments.
        let mut stderr = io::stderr();
        let args = match parser::input_to_arguments(input) {
            Ok(args) => args,
            Err(e) => {
                writeln!(stderr, "nes-rs: {}", e).unwrap();
                return;
            },
        };

        // Determine which command to execute given the first argument and
        // execute it's routine.
        let raw_command = if args.len() > 0 {
            &args[0]
        } else {
            writeln!(stderr, "nes-rs: no command specified").unwrap();
            return;
        };
        let command = match raw_command.to_lowercase().as_str() {
            // Full commands.
            "stop"     => Command::Stop,
            "continue" => Command::Continue,
            "dump"     => Command::Dump,
            // Aliases.
            "s" => Command::Stop,
            "c" => Command::Continue,
            "d" => Command::Dump,
            // Unknown command.
            _ => {
                writeln!(stderr, "nes-rs: unknown command specified").unwrap();
                return;
            },
        };
        self.execute_command(command, &args[1..]);
    }

    /// Executes a debugger command.
    fn execute_command(&mut self, command: Command, args: &[String]) {
        println!("{:?} => {:?}", command, args);
    }
}
