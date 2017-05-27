// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use debugger::parser;
use io::log;
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

struct CommandWithArguments {
    command: Command,
    args: Vec<String>,
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
            Ok(input) => {
                match self.interpret(input) {
                    Some(command) => {
                        self.execute_command(command, nes);
                    },
                    None => {
                        let mut stderr = io::stderr();
                        writeln!(stderr, "nes-rs: unknown command specified").unwrap();
                    },
                };
            },
            Err(_) => {}, // Ignore empty and disconnect errors.
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

    /// Parse a raw input string into a list of arguments and a command. This
    /// function also maps command names to their respective enums.
    fn interpret(&self, input: String) -> Option<CommandWithArguments> {
        let mut stderr = io::stderr();
        let args = match parser::input_to_arguments(input) {
            Ok(args) => args,
            Err(e) => {
                writeln!(stderr, "nes-rs: {}", e).unwrap();
                return None;
            },
        };

        let command = {
            let raw_command = if args.len() > 0 {
                &args[0]
            } else {
                writeln!(stderr, "nes-rs: no command specified").unwrap();
                return None;
            };

            // Map command strings to the command enum type.
            match raw_command.to_lowercase().as_str() {
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
                    return None;
                },
            }
        };

        Some({
            CommandWithArguments {
                command: command,
                args: args,
            }
        })
    }

    /// Executes the correct debugger command based on the enum passed.
    fn execute_command(&mut self, command: CommandWithArguments, nes: &mut NES) {
        match command.command {
            Command::Stop => self.execute_stop(nes),
            Command::Continue => self.execute_continue(nes),
            Command::Dump => self.execute_dump(nes, &command.args),
        };
    }

    /// Stops execution of the CPU and PPU to allow the human some time to debug
    /// a problem or stare at hex codes all day to look like a l33t haxor.
    fn execute_stop(&mut self, nes: &mut NES) {
        log::log("debugger", "Stopping execution now...", &nes.runtime_options);
        self.stepping = false;
    }

    /// Starts execution if it's stopped.
    fn execute_continue(&mut self, nes: &mut NES) {
        log::log("debugger", "Starting execution now...", &nes.runtime_options);
        self.stepping = true;
    }

    /// Allows dumping memory or program code at a specified memory address.
    fn execute_dump(&mut self, nes: &mut NES, args: &Vec<String>) {
        log::log("debugger", "TODO: Dump truck time!", &nes.runtime_options);
    }
}
