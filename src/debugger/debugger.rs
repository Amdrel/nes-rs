// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use debugger::parser;
use getopts::Options;
use io::log;
use nes::nes::NES;
use utils::arithmetic;
use std::io::{self, Write, stderr, stdout};
use std::sync::mpsc::{SyncSender, Receiver};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum Command {
    Help,
    Exit,
    Stop,
    Continue,
    Dump,
    ObjDump,
}

struct CommandWithArguments {
    command: Command,
    args: Vec<String>,
}

pub struct Debugger {
    sender:   SyncSender<u8>,
    receiver: Receiver<String>,
    stepping: bool,
    shutdown: bool,
}

impl Debugger {
    pub fn new(sender: SyncSender<u8>, receiver: Receiver<String>) -> Self {
        Self {
            sender: sender,
            receiver: receiver,
            stepping: true,
            shutdown: false,
        }
    }

    /// Steps the CPU forward a single instruction, as well as executing any PPU
    /// and sound functionality that happens in-between.
    pub fn step(&mut self, nes: &mut NES) -> bool {
        // Input is received from another thread so the emulator can run without
        // the debugger prompt blocking it.
        match self.receiver.try_recv() {
            Ok(input) => {
                if let Some(command) = self.interpret(input.clone()) {
                    self.execute_command(command, nes);
                } else {
                    if input.len() > 0 {
                        writeln!(stderr(), "nes-rs: unknown command specified").unwrap();
                    }
                }

                // Tell the input thread to continue and display prompt.
                if let Err(_) = self.sender.send(0) {
                }
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

        self.shutdown
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
                return None;
            };

            // Map command strings to the command enum type.
            match raw_command.to_lowercase().as_str() {
                // Full commands.
                "help"     => Command::Help,
                "exit"     => Command::Exit,
                "stop"     => Command::Stop,
                "continue" => Command::Continue,
                "dump"     => Command::Dump,
                "objdump"  => Command::ObjDump,
                // Aliases.
                "s"  => Command::Stop,
                "c"  => Command::Continue,
                "d"  => Command::Dump,
                "od" => Command::ObjDump,
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
            Command::Help => self.execute_help(),
            Command::Exit => self.execute_exit(),
            Command::Stop => self.execute_stop(nes),
            Command::Continue => self.execute_continue(nes),
            Command::Dump => self.execute_dump(nes, &command.args),
            Command::ObjDump => self.execute_objdump(nes, &command.args),
        };
    }

    /// Shows friendly help text for information about using the debugger.
    fn execute_help(&self) {
        writeln!(stderr(), "
Welcome to the nes-rs debugger!

This subshell provides access to a few different commands that allow you to
modify and observe the state of the virtual machine. At the moment there is a
very limited set of commands and more may be added in the future.

Supported commands: help | exit | stop | continue | dump | objdump
"
        ).unwrap();
    }

    /// Stops the virtual machine by setting the shutdown flag.
    fn execute_exit(&mut self) {
        self.shutdown = true;

        // Let the input thread know we're shutting things down so it can save
        // the input history for the next run.
        if let Err(_) = self.sender.send(1) {
        }
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

    /// Allows dumping memory or program code at a specified memory address. All
    /// values are assumed to be 8-bit words.
    fn execute_dump(&mut self, nes: &mut NES, args: &Vec<String>) {
        const USAGE: &'static str = "Usage: dump [OPTION]... [ADDRESS]";

        let mut opts = Options::new();
        opts.optopt("p", "peek", "how far forward should memory be dumped", "NUMBER");

        let matches = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => {
                writeln!(stderr(), "dump: {}", f).unwrap();
                writeln!(stderr(), "{}", opts.usage(USAGE)).unwrap();
                return;
            },
        };

        // Peek allows specifying how much information to dump.
        let peek = match matches.opt_str("peek") {
            Some(arg) => {
                match arg.parse::<u16>() {
                    Ok(p) => p,
                    Err(e) => {
                        writeln!(stderr(), "dump: {}", e).unwrap();
                        writeln!(stderr(), "{}", opts.usage(USAGE)).unwrap();
                        return;
                    },
                }
            },
            None => 10,
        };

        // Parse hex representation of a memory address at free argument if
        // available, otherwise the address will be the program counter.
        let addr = if !matches.free.is_empty() {
            let arg = matches.free[0].clone();
            if let Some(hex) = arithmetic::hex_to_u16(&arg) {
                hex
            } else {
                writeln!(stderr(), "dump: cannot parse address: {}", arg).unwrap();
                return;
            }
        } else {
            nes.cpu.pc
        };

        for idx in 0..peek {
            let peek_offset = addr.wrapping_add(idx * 16);
            let mut bytes: [u8; 16] = [0; 16];
            for offset in 0..16 {
                let current_addr = (peek_offset.wrapping_add(offset)) as usize;
                let value = nes.memory.read_u8_unrestricted(current_addr);
                bytes[offset as usize] = value;
            }

            // Print the memory address for for the first byte in the line.
            print!("{:04x}  ", peek_offset);

            // Print out 2 lines of 8-bit bytes showing 8 bytes each.
            for offset in 0..8 {
                print!("{:02x} ", bytes[offset]);
            }
            print!(" ");
            for offset in 0..8 {
                print!("{:02x} ", bytes[offset + 8]);
            }

            // Print out an ASCII representation of the bytes.
            print!(" ");
            for offset in 0..16 {
                let value = bytes[offset];
                let human_char = if value >= 0x20 && value <= 0x7E {
                    value as char
                } else {
                    '.'
                };
                print!("{}", human_char);
            }
            print!("\n");

            stdout().flush().unwrap();
        }
    }

    /// Similar to dump, but will interpret data as instructions.
    fn execute_objdump(&mut self, nes: &mut NES, args: &Vec<String>) {
        const USAGE: &'static str = "Usage: objdump [OPTION]... [ADDRESS]";

        let mut opts = Options::new();
        opts.optopt("p", "peek", "how far forward should memory be dumped", "NUMBER");

        let matches = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => {
                writeln!(stderr(), "dump: {}", f).unwrap();
                writeln!(stderr(), "{}", opts.usage(USAGE)).unwrap();
                return;
            },
        };

        // Peek allows specifying how much information to dump.
        let peek = match matches.opt_str("peek") {
            Some(arg) => {
                match arg.parse::<u16>() {
                    Ok(p) => p,
                    Err(e) => {
                        writeln!(stderr(), "dump: {}", e).unwrap();
                        writeln!(stderr(), "{}", opts.usage(USAGE)).unwrap();
                        return;
                    },
                }
            },
            None => 10,
        };

        // Parse hex representation of a memory address at free argument if
        // available, otherwise the address will be the program counter.
        let addr = if !matches.free.is_empty() {
            let arg = matches.free[0].clone();
            if let Some(hex) = arithmetic::hex_to_u16(&arg) {
                hex
            } else {
                writeln!(stderr(), "dump: cannot parse address: {}", arg).unwrap();
                return;
            }
        } else {
            nes.cpu.pc
        };

        println!("Got ya good!");
    }
}
