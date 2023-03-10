use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Mutex;
use crate::cpu::{Memory, State};
use crate::cpu::error::Error;
use crate::cpu::state::Registers;
use crate::debug::debugger::DebuggerMode::{Breakpoint, Invalid, Paused, Recovered, Running};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DebuggerMode {
    Running,
    Recovered, // Recovery State after Invalid(CpuSyscall)
    Invalid(Error),
    Paused,
    Breakpoint,
}

// Addresses
type Breakpoints = HashSet<u32>;

pub struct Debugger<Mem: Memory> {
    mode: DebuggerMode,

    state: State<Mem>,
    breakpoints: Breakpoints,
    batch: usize
}

#[derive(Debug)]
pub struct DebugFrame {
    pub mode: DebuggerMode,
    pub registers: Registers
}

impl<Mem: Memory> Debugger<Mem> {
    pub fn new(state: State<Mem>) -> Debugger<Mem> {
        Debugger {
            mode: Paused, state,
            breakpoints: HashSet::new(),
            batch: 140
        }
    }

    fn frame_with_pc(&self, pc: u32) -> DebugFrame {
        let mut registers = self.state.registers;
        registers.pc = pc;

        DebugFrame {
            mode: self.mode,
            registers
        }
    }

    pub fn invalid_handled(&mut self) {
        match self.mode {
            Invalid(_) => self.mode = Recovered,
            _ => { }
        }
    }

    pub fn set_breakpoints(&mut self, breakpoints: Breakpoints) {
        self.breakpoints = breakpoints
    }

    pub fn frame(&self) -> DebugFrame {
        self.frame_with_pc(self.state.registers.pc)
    }

    pub fn state(&mut self) -> &mut State<Mem> {
        &mut self.state
    }

    pub fn memory(&mut self) -> &mut Mem {
        &mut self.state.memory
    }

    pub fn cycle(&mut self, hit_breakpoint: bool) -> Option<DebugFrame> {
        if !hit_breakpoint && self.breakpoints.contains(&self.state.registers.pc) {
            self.mode = Breakpoint;

            return Some(self.frame())
        }

        let start_pc = self.state.registers.pc;

        if let Err(err) = self.state.step() {
            self.mode = Invalid(err);

            Some(self.frame_with_pc(start_pc))
        } else {
            None
        }
    }

    pub fn pause(&mut self) {
        self.mode = Paused
    }

    pub fn run(debugger: &Mutex<Debugger<Mem>>) -> DebugFrame {
        let mut hit_breakpoint = {
            let mut value = debugger.lock().unwrap();

            if value.mode == Running {
                return value.frame()
            }

            let result = value.mode;
            value.mode = Running;

            result == Breakpoint
        };

        loop {
            let mut value = debugger.lock().unwrap();

            for _ in 0 .. value.batch {
                if value.mode != Running {
                    return value.frame()
                }

                if let Some(frame) = value.cycle(hit_breakpoint) {
                    return frame
                }

                hit_breakpoint = false
            }
        }
    }
}
