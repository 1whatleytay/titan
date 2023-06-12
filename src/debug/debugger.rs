use crate::cpu::error::Error;
use crate::cpu::state::Registers;
use crate::cpu::{Memory, State};
use crate::debug::debugger::DebuggerMode::{Breakpoint, Invalid, Paused, Recovered, Running};
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Mutex;

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

pub struct DebuggerState<Mem: Memory> {
    mode: DebuggerMode,

    state: State<Mem>,
    breakpoints: Breakpoints,
    batch: usize,
}

pub struct Debugger<Mem: Memory> {
    mutex: Mutex<DebuggerState<Mem>>
}

#[derive(Debug)]
pub struct DebugFrame {
    pub mode: DebuggerMode,
    pub registers: Registers,
}

impl<Mem: Memory> DebuggerState<Mem> {
    fn new(state: State<Mem>) -> DebuggerState<Mem> {
        DebuggerState {
            mode: Paused,
            state,
            breakpoints: HashSet::new(),
            batch: 140,
        }
    }

    fn frame_with_pc(&self, pc: u32) -> DebugFrame {
        let mut registers = self.state.registers;
        registers.pc = pc;

        DebugFrame {
            mode: self.mode,
            registers,
        }
    }

    pub fn frame(&self) -> DebugFrame {
        self.frame_with_pc(self.state.registers.pc)
    }

    pub fn cycle(&mut self, no_breakpoints: bool) -> Option<DebugFrame> {
        if !no_breakpoints && self.breakpoints.contains(&self.state.registers.pc) {
            self.mode = Breakpoint;

            return Some(self.frame());
        }

        let start_pc = self.state.registers.pc;

        if let Err(err) = self.state.step() {
            self.mode = Invalid(err);

            Some(self.frame_with_pc(start_pc))
        } else {
            None
        }
    }
}

impl<Mem: Memory> Debugger<Mem> {
    pub fn new(state: State<Mem>) -> Debugger<Mem> {
        Debugger {
            mutex: Mutex::new(DebuggerState::new(state))
        }
    }

    pub fn frame(&self) -> DebugFrame {
        self.mutex.lock().unwrap().frame()
    }

    pub fn pause(&self) {
        self.mutex.lock().unwrap().mode = Paused
    }

    pub fn with_state<T, F: FnOnce (&mut State<Mem>) -> T>(&self, f: F) -> T {
        let mut lock = self.mutex.lock().unwrap();

        f(&mut lock.state)
    }

    pub fn with_memory<T, F: FnOnce (&mut Mem) -> T>(&self, f: F) -> T {
        let mut lock = self.mutex.lock().unwrap();

        f(&mut lock.state.memory)
    }

    pub fn invalid_handled(&self) {
        let mut lock = self.mutex.lock().unwrap();

        if let Invalid(_) = lock.mode {
            lock.mode = Recovered
        }
    }

    pub fn set_breakpoints(&self, breakpoints: Breakpoints) {
        let mut lock = self.mutex.lock().unwrap();

        lock.breakpoints = breakpoints
    }

    pub fn cycle(&self, no_breakpoints: bool) -> Option<DebugFrame> {
        self.mutex.lock().unwrap().cycle(no_breakpoints)
    }

    pub fn run(&self) -> DebugFrame {
        let mut hit_breakpoint = {
            let mut value = self.mutex.lock().unwrap();

            if value.mode == Running {
                return value.frame();
            }

            let result = value.mode;
            value.mode = Running;

            result == Breakpoint
        };

        loop {
            let mut value = self.mutex.lock().unwrap();

            for _ in 0..value.batch {
                if value.mode != Running {
                    return value.frame();
                }

                if let Some(frame) = value.cycle(hit_breakpoint) {
                    return frame;
                }

                hit_breakpoint = false
            }
        }
    }
}
