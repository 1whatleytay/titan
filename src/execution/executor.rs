use crate::cpu::error::Error;
use crate::cpu::state::Registers;
use crate::cpu::{Memory, State};
use crate::execution::executor::ExecutorMode::{Breakpoint, Invalid, Paused, Recovered, Running};
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Mutex;
use crate::execution::trackers::empty::EmptyTracker;
use crate::execution::trackers::Tracker;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExecutorMode {
    Running,
    Recovered, // Recovery State after Invalid(CpuSyscall)
    Invalid(Error),
    Paused,
    Breakpoint,
}

// Addresses
type Breakpoints = HashSet<u32>;

pub struct ExecutorState<Mem: Memory, Track: Tracker<Mem>> {
    mode: ExecutorMode,

    state: State<Mem>,
    breakpoints: Breakpoints,
    batch: usize,

    tracker: Track
}

pub struct Executor<Mem: Memory, Track: Tracker<Mem>> {
    mutex: Mutex<ExecutorState<Mem, Track>>,
}

#[derive(Debug)]
pub struct DebugFrame {
    pub mode: ExecutorMode,
    pub registers: Registers,
}

impl<Mem: Memory, Track: Tracker<Mem>> ExecutorState<Mem, Track> {
    fn new(state: State<Mem>, tracker: Track) -> ExecutorState<Mem, Track> {
        ExecutorState {
            mode: Paused,
            state,
            breakpoints: HashSet::new(),
            batch: 140,
            tracker
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

        self.tracker.pre_track(&mut self.state);
        let result = self.state.step();

        if let Err(err) = result {
            self.mode = Invalid(err);

            Some(self.frame_with_pc(start_pc))
        } else {
            // Only track the instruction if it did not fail.
            // This means back-stepping will not go back to your instruction.
            self.tracker.post_track(&mut self.state);

            None
        }
    }
}

impl<Mem: Memory, Track: Tracker<Mem>> Executor<Mem, Track> {
    pub fn new(state: State<Mem>, tracker: Track) -> Executor<Mem, Track> {
        Executor {
            mutex: Mutex::new(ExecutorState::new(state, tracker)),
        }
    }

    pub fn from_state(state: State<Mem>) -> Executor<Mem, EmptyTracker> {
        Executor {
            mutex: Mutex::new(ExecutorState::new(state, EmptyTracker { }))
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

    pub fn with_tracker<T, F: FnOnce (&mut Track) -> T>(&self, f: F) -> T {
        let mut lock = self.mutex.lock().unwrap();

        f(&mut lock.tracker)
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

    pub fn run_limited<const BREAK_BATCH: bool>(&self, batch: usize) -> DebugFrame {
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

            for _ in 0..batch {
                if value.mode != Running {
                    return value.frame();
                }

                if let Some(frame) = value.cycle(hit_breakpoint) {
                    return frame;
                }

                hit_breakpoint = false
            }

            if BREAK_BATCH {
                value.mode = Breakpoint;

                return value.frame();
            }
        }
    }

    pub fn run(&self) -> DebugFrame {
        let batch = self.mutex.lock().unwrap().batch;

        self.run_limited::<false>(batch)
    }
}
