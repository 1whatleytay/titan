use std::collections::HashSet;
use std::sync::Mutex;
use crate::cpu::{Memory, State};
use crate::debug::debugger::DebuggerMode::{Breakpoint, Invalid, Paused, Running};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DebuggerMode {
    Running,
    Invalid,
    Paused,
    Breakpoint,
}

pub struct Debugger {
    mode: DebuggerMode,

    state: State
}

// Addresses
type Breakpoints = HashSet<u32>;

#[derive(Debug)]
pub struct DebugFrame {
    pub mode: DebuggerMode,

    pub pc: u32,
    pub registers: [u32; 32],
    pub lo: u32,
    pub hi: u32
}

impl DebugFrame {
    fn default(mode: DebuggerMode) -> DebugFrame {
        DebugFrame {
            mode,

            pc: 0,
            registers: [0; 32],
            lo: 0,
            hi: 0
        }
    }
}

impl Debugger {
    pub fn new(state: State) -> Debugger {
        Debugger { mode: Paused, state }
    }

    fn frame_with_pc(&self, pc: u32) -> DebugFrame {
        DebugFrame {
            mode: self.mode,
            pc,
            registers: self.state.registers,
            lo: self.state.lo,
            hi: self.state.hi,
        }
    }

    pub fn frame(&self) -> DebugFrame {
        self.frame_with_pc(self.state.pc)
    }

    pub fn state(&mut self) -> &mut State {
        &mut self.state
    }

    pub fn memory(&mut self) -> &mut Memory {
        &mut self.state.memory
    }

    pub fn cycle(&mut self, breakpoints: &Breakpoints, hit_breakpoint: bool) -> Option<DebugFrame> {
        if !hit_breakpoint && breakpoints.contains(&self.state.pc) {
            self.mode = Breakpoint;

            return Some(self.frame())
        }

        let start_pc = self.state.pc;

        if let Err(err) = self.state.step() {
            println!("Invalid Instruction: {}", err);

            self.mode = Invalid;

            Some(self.frame_with_pc(start_pc))
        } else {
            None
        }
    }

    pub fn pause(&mut self) {
        self.mode = Paused
    }

    pub fn run(debugger: &Mutex<Debugger>, breakpoints: &Breakpoints) -> DebugFrame {
        let mut hit_breakpoint = {
            let mut value = debugger.lock().unwrap();

            if value.mode == Running {
                return DebugFrame::default(Running)
            }

            let result = value.mode;
            value.mode = Running;

            result == Breakpoint
        };

        loop {
            let mut value = debugger.lock().unwrap();

            if value.mode != Running {
                return value.frame()
            }

            if let Some(frame) = value.cycle(breakpoints, hit_breakpoint) {
                return frame
            }

            hit_breakpoint = false
        }
    }
}

impl Drop for Debugger {
    fn drop(&mut self) {
        self.pause()
    }
}
