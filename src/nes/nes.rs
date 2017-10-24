// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use sdl2;
use sdl2::EventPump;
use sdl2::render;
use sdl2::render::Canvas;
use sdl2::pixels::Color;
use sdl2::video::Window;
use sdl2::event::Event;
use debugger::debugger::Debugger;
use io::binutils::INESHeader;
use io::errors::*;
use io::log;
use nes::cpu::CPU;
use nes::ppu::PPU;
use std::fs::File;
use std::io::{self, stdin, Read, Write, BufReader, BufRead};
use std::sync::mpsc::{self, SyncSender, Receiver};
use std::{thread, panic};
use std::time::Duration;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use nes::memory::{
    Memory,
    TRAINER_START,
    TRAINER_SIZE,
    PRG_ROM_1_START,
    PRG_ROM_2_START,
    PRG_ROM_SIZE
};

const HISTORY_FILE: &'static str = ".nes-rs-history.txt";

/// The NES struct owns all hardware peripherals and lends them when needed. The
/// runtime cost of this should be removed with optimized builds (untested).
pub struct NES {
    pub header: INESHeader,
    pub runtime_options: NESRuntimeOptions,

    pub cpu: CPU,
    pub ppu: PPU,
    pub memory: Memory,

    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,
}

impl NES {
    /// Initializes the NES emulator by dumping the ROM into memory and
    /// initializing the initial hardware state.
    pub fn new(rom: Vec<u8>, header: INESHeader, runtime_options: NESRuntimeOptions) -> Self {
        // An offset is used when copying from the ROM into RAM as the presence
        // of a trainer will shift the locations of other structures.
        let mut cursor: usize = 0x10;

        // Spew out some useful metadata about the rom when verbose is on.
        log::log("init", format!("Using {:?} mapper", header.mapper()), &runtime_options);
        log::log("init", format!("Using {:?} mirroring", header.mirror_type()), &runtime_options);

        // Copy the trainer data to 0x7000 if it exists and adjust the cursor
        // size to accommodate. Trainer data will offset the location of ROM
        // data in the INES ROM file.
        let mut memory = Memory::new();
        if header.has_trainer() {
            log::log("init", "Trainer data found", &runtime_options);
            memory.memdump(TRAINER_START, &rom[0x10..0x210]);
            cursor += TRAINER_SIZE;
        }

        // Copy PRG-ROM into memory so it can be addressed by the chosen memory
        // mapper. PRG-ROM bank 1 begins at 0x8000 and bank 2 begins at 0xC000.
        //
        // In the event that there are 2 PRG-ROM banks, make both banks
        // addressable at their respective locations. However if there's only
        // one bank, make PRG-ROM bank 1 addressable starting from both
        // addresses.
        //
        // NOTE: Should this be moved to mapper code?
        if header.prg_rom_size == 2 {
            log::log("init", "2 PRG-ROM banks detected", &runtime_options);
            let prg_rom_1_addr = cursor;
            let prg_rom_2_addr = cursor + PRG_ROM_SIZE;
            memory.memdump(PRG_ROM_1_START, &rom[prg_rom_1_addr..prg_rom_1_addr + PRG_ROM_SIZE]);
            memory.memdump(PRG_ROM_2_START, &rom[prg_rom_2_addr..prg_rom_2_addr + PRG_ROM_SIZE]);
        } else {
            log::log("init", "1 PRG-ROM bank detected", &runtime_options);
            let prg_rom_1_addr = cursor;
            memory.memdump(PRG_ROM_1_START, &rom[prg_rom_1_addr..prg_rom_1_addr + PRG_ROM_SIZE]);
            memory.memdump(PRG_ROM_2_START, &rom[prg_rom_1_addr..prg_rom_1_addr + PRG_ROM_SIZE]);
        }

        // Set the initial program counter to the address stored at 0xFFFC (this
        // allows ROMs to specify entry point). If a program counter was
        // specified on the command-line, use that one instead.
        let pc = match runtime_options.program_counter {
            Some(pc) => pc,
            None => {
                memory.read_u16(0xFFFC)
            },
        };

        // Create an SDL window that represents the display.
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("nes-rs", 256, 240)
            .position_centered()
            .build()
            .unwrap();

        // Create a canvas that is scaled up a bit.
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.clear();
        canvas.present();

        NES {
            header: header,
            cpu: CPU::new(runtime_options.clone(), pc),
            ppu: PPU::new(runtime_options.clone()),
            runtime_options: runtime_options,
            memory: memory,
            canvas: canvas,
            event_pump: sdl_context.event_pump().unwrap(),
        }
    }

    /// Starts the execution loop and starts executing PRG-ROM.
    pub fn run(&mut self) -> i32 {
        // Put the CPU into testing mode if a cpu log was passed in the runtime
        // options. This is done before execution so the log and the CPU state
        // are kept in sync.
        match self.runtime_options.cpu_log {
            Some(ref filename) => {
                match File::open(filename) {
                    Ok(f) => {
                        self.cpu.begin_testing(BufReader::new(f))
                    },
                    Err(e) => {
                        let mut stderr = io::stderr();
                        writeln!(stderr, "nes-rs: cannot open {}: {}", filename, e).unwrap();
                        return EXIT_CPU_LOG_NOT_FOUND;
                    },
                }
            },
            None => {},
        }

        // Start cycling the CPU and PPU and add a panic catcher so crash
        // information can be shown if the CPU panics.The PPU ticks three times
        // every CPU cycle, though there may need to be changes made for PAL
        // (currently assumes NTSC PPU clock speed).
        //
        // Depending on the runtime environment, execution can go one of two
        // ways. Either the virtual machine step function is called in an
        // infinite loop, or the debugger handles execution if the debug flag is
        // set.
        //
        // In debug mode, there is another step function that wraps the main
        // step function that lets the debugger control execution flow and
        // access virtual machine state. Another thread is also setup that waits
        // for input on stdin that sends input to the debugger for the debugger
        // subshell.
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            if self.runtime_options.debugging {
                let (tx, rx): (SyncSender<String>, Receiver<String>) = mpsc::sync_channel(1);
                let (mtx, mrx): (SyncSender<u8>, Receiver<u8>) = mpsc::sync_channel(1);

                // Input is read on another thread, so spin one up.
                self.setup_readline_thread(tx, mrx);

                // Execute until shutdown signal is received from debugger.
                let mut debugger = Debugger::new(mtx, rx);
                while !debugger.step(self) {
                    let quit = self.poll_sdl_events();
                    if quit {
                        break;
                    }
                }
            } else {
                loop {
                    let quit = self.poll_sdl_events();
                    if quit {
                        break;
                    }

                    self.step();
                }
            }
        }));

        // Unwinding point with shutdown code. In the event of a panic, we want
        // to display some diagnostic information to the user that can be sent
        // to the developer.
        match result {
            Ok(_) => {
                println!("Shutting down nes-rs, happy emulating!");
                return EXIT_SUCCESS; // Success exit code.
            },
            Err(_) => {
                thread::sleep(Duration::from_millis(16));
                println!("{}", self.cpu);
                return EXIT_RUNTIME_FAILURE; // Runtime failure exit code.
            }
        }
    }

    /// Executes a CPU instruction and steps the PPU 3 times per CPU cycle. This
    /// works since the PPU and CPU clocks are synchronized 1 to 3.
    pub fn step(&mut self) {
        let mut cycles = self.cpu.step(&mut self.memory);
        self.cpu.sleep(cycles);

        while cycles > 0 {
            for _ in 0..3 { // *Should* unroll.
                self.ppu.step(&mut self.memory);
            }
            cycles -= 1;
        }
    }

    /// Polls for SDL events, inparticular the quit one. A boolean is returned
    /// which if true will stop emulation.
    fn poll_sdl_events(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    return true;
                },
                _ => {}
            }
        }

        return false;
    }

    /// Creates a readline loop on another thread and sends commands to the
    /// debugger over a synchronous rust channel. Offers quality of life features
    /// such as history built into the library used.
    fn setup_readline_thread(&self, tx: SyncSender<String>, rx: Receiver<u8>) {
        thread::spawn(move || {
            let mut rl = Editor::<()>::new();
            if let Err(_) = rl.load_history(HISTORY_FILE) {
                // No history saved, do nothing.
            }

            loop {
                let readline = rl.readline("(nes-rs) ");
                match readline {
                    Ok(line) => {
                        rl.add_history_entry(&line);
                        tx.send(line).unwrap();

                        // Block until the command is done running or the main
                        // thread tells us to shutdown.
                        match rx.recv() {
                            Ok(code) => {
                                match code {
                                    0 => {}, // 0 means the command has run.
                                    1 => { break }, // 1 is an exit command.
                                    _ => {},
                                }
                            },
                            Err(_) => {
                                break;
                            },
                        }
                    },
                    Err(ReadlineError::Interrupted) => {
                        tx.send("exit".to_string()).unwrap();
                        break;
                    },
                    Err(ReadlineError::Eof) => {
                        tx.send("exit".to_string()).unwrap();
                        break;
                    },
                    Err(err) => {
                        println!("Error: {:?}", err);
                        tx.send("exit".to_string()).unwrap();
                        break;
                    },
                };
            }

            println!("Saving debugger history...");
            rl.save_history(HISTORY_FILE).unwrap();
        });
    }
}

/// Flags and other information set through command-line arguments.
#[derive(Clone, Debug)]
pub struct NESRuntimeOptions {
    pub program_counter: Option<u16>,
    pub cpu_log:         Option<String>,
    pub verbose:         bool,
    pub debugging:       bool,
}
