use crate::cpu::error::Error;
use crate::cpu::registers::registers::RawRegisters;
use crate::cpu::registers::WhichRegister::Pc;
use crate::cpu::state::Registers;
use crate::cpu::{Memory, State};
use crate::execution::executor::ExecutorMode::{Breakpoint, Invalid, Paused, Running};
use crate::execution::trackers::empty::EmptyTracker;
use crate::execution::trackers::Tracker;
use std::collections::HashSet;
use std::fmt::Debug;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExecutorMode {
    Running,
    Invalid(Error),
    Paused,
    Breakpoint,
}

// Addresses
type Breakpoints = HashSet<u32>;

pub struct ExecutorState<Mem: Memory, Reg: Registers, Track: Tracker<Mem, Reg>> {
    mode: ExecutorMode,

    state: State<Mem, Reg>,
    breakpoints: Breakpoints,
    batch: usize,

    tracker: Track,
}

pub struct Executor<Mem: Memory, Reg: Registers, Track: Tracker<Mem, Reg>> {
    mutex: parking_lot::Mutex<ExecutorState<Mem, Reg, Track>>,
}

#[derive(Debug)]
pub struct DebugFrame {
    pub mode: ExecutorMode,
    pub registers: RawRegisters,
}

impl<Mem: Memory, Reg: Registers, Track: Tracker<Mem, Reg>> ExecutorState<Mem, Reg, Track> {
    fn new(state: State<Mem, Reg>, tracker: Track) -> ExecutorState<Mem, Reg, Track> {
        ExecutorState {
            mode: Paused,
            state,
            breakpoints: HashSet::new(),
            batch: 140,
            tracker,
        }
    }

    pub fn frame(&self) -> DebugFrame {
        DebugFrame {
            mode: self.mode,
            registers: self.state.registers.raw(),
        }
    }

    // Returns true if the CPU was interrupted.
    // If true, see self.frame() for details (ex. the mode)
    pub fn cycle(&mut self, no_breakpoints: bool) -> bool {
        if !no_breakpoints && self.breakpoints.contains(&self.state.registers.get(Pc)) {
            self.mode = Breakpoint;

            return true;
        }

        self.tracker.pre_track(&mut self.state);
        let result = self.state.step();

        if let Err(err) = result {
            self.mode = Invalid(err);

            true
        } else {
            // Only track the instruction if it did not fail.
            // This means back-stepping will not go back to your instruction.
            self.tracker.post_track(&mut self.state);

            false
        }
    }
}

pub struct BatchResult {
    pub instructions_executed: u64,
    pub interrupted: bool,
}

impl<Mem: Memory, Reg: Registers, Track: Tracker<Mem, Reg>> Executor<Mem, Reg, Track> {
    pub fn new(state: State<Mem, Reg>, tracker: Track) -> Executor<Mem, Reg, Track> {
        Executor {
            mutex: parking_lot::Mutex::new(ExecutorState::new(state, tracker)),
        }
    }

    pub fn from_state(state: State<Mem, Reg>) -> Executor<Mem, Reg, EmptyTracker> {
        Executor {
            mutex: parking_lot::Mutex::new(ExecutorState::new(state, EmptyTracker {})),
        }
    }

    pub fn frame(&self) -> DebugFrame {
        self.mutex.lock().frame()
    }

    pub fn pause(&self) {
        self.mutex.lock().mode = Paused
    }

    pub fn override_mode(&self, mode: ExecutorMode) {
        self.mutex.lock().mode = mode
    }

    pub fn with_state<T, F: FnOnce(&mut State<Mem, Reg>) -> T>(&self, f: F) -> T {
        let mut lock = self.mutex.lock();

        f(&mut lock.state)
    }

    pub fn with_memory<T, F: FnOnce(&mut Mem) -> T>(&self, f: F) -> T {
        let mut lock = self.mutex.lock();

        f(&mut lock.state.memory)
    }

    pub fn with_tracker<T, F: FnOnce(&mut Track) -> T>(&self, f: F) -> T {
        let mut lock = self.mutex.lock();

        f(&mut lock.tracker)
    }

    pub fn syscall_handled(&self) {
        let mut lock = self.mutex.lock();

        if let Invalid(_) = lock.mode {
            lock.mode = Running
        }

        let new_pc = lock.state.registers.get(Pc) + 4;
        lock.state.registers.set(Pc, new_pc);
    }

    pub fn set_breakpoints(&self, breakpoints: Breakpoints) {
        let mut lock = self.mutex.lock();

        lock.breakpoints = breakpoints
    }

    // Returns true if CPU was interrupted.
    pub fn cycle(&self, no_breakpoints: bool) -> bool {
        self.mutex.lock().cycle(no_breakpoints)
    }

    pub fn is_breakpoint(&self) -> bool {
        self.mutex.lock().mode == Breakpoint
    }

    // Returns true if the CPU was interrupted.
    pub fn run_batched(
        &self,
        batch: usize,
        mut skip_first_breakpoint: bool,
        allow_interrupt: bool,
    ) -> BatchResult {
        let mut value = self.mutex.lock();

        let mut instructions_executed = 0;

        for _ in 0..batch {
            if allow_interrupt && value.mode != Running {
                return BatchResult {
                    instructions_executed,
                    interrupted: true,
                };
            }

            if value.cycle(skip_first_breakpoint) {
                return BatchResult {
                    instructions_executed,
                    interrupted: true,
                };
            }

            instructions_executed += 1;

            skip_first_breakpoint = false
        }

        BatchResult {
            instructions_executed,
            interrupted: false,
        }
    }

    pub fn run(&self, mut skip_first_breakpoint: bool) -> DebugFrame {
        let batch = self.mutex.lock().batch;

        while !self
            .run_batched(batch, skip_first_breakpoint, true)
            .interrupted
        {
            skip_first_breakpoint = false
        }

        self.frame()
    }
}
