use crate::cpu::error::Error;
// use crate::cpu::registers::registers::RawRegisters;
// use crate::cpu::registers::WhichRegister::Pc;
// use crate::cpu::state::Registers;
// use crate::cpu::{Memory, State};
use crate::execution::executor::ExecutorMode::{Breakpoint, Invalid, Paused, Running};
use crate::execution::trackers::empty::EmptyTracker;
use std::collections::HashSet;
use std::fmt::Debug;
use crate::cpu::Memory;
use crate::cpu::error::Result;
use crate::execution::trackers::tracker::Tracker;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExecutorMode {
    Running,
    Invalid(Error),
    Paused,
    Breakpoint,
}

// Addresses
type Breakpoints = HashSet<u32>;

pub trait ExecutableState<Reg, Mem: Memory> {
    fn pc(&self) -> u32;
    fn set_pc(&mut self, value: u32);

    fn registers(&self) -> Reg;

    fn memory_mut(&mut self) -> &mut Mem;

    fn step(&mut self) -> Result<()>;
}

pub struct ExecutorState<State, Track: Tracker<State>> {
    mode: ExecutorMode,

    state: State,
    breakpoints: Breakpoints,
    batch: usize,

    tracker: Track,
}

pub struct Executor<State, Track: Tracker<State>> {
    mutex: parking_lot::Mutex<ExecutorState<State, Track>>,
}

#[derive(Debug)]
pub struct DebugFrame<Reg> {
    pub mode: ExecutorMode,
    pub registers: Reg,
}

impl<State, Track: Tracker<State>> ExecutorState<State, Track> {
    fn new(state: State, tracker: Track) -> Self {
        ExecutorState {
            mode: Paused,
            state,
            breakpoints: HashSet::new(),
            batch: 140,
            tracker,
        }
    }

    pub fn frame<Reg, Mem: Memory>(&self) -> DebugFrame<Reg> where State: ExecutableState<Reg, Mem> {
        DebugFrame {
            mode: self.mode,
            registers: self.state.registers(),
        }
    }

    // Returns true if the CPU was interrupted.
    // If true, see self.frame() for details (ex. the mode)
    pub fn cycle<Reg, Mem: Memory>(&mut self, no_breakpoints: bool) -> bool where State: ExecutableState<Reg, Mem> {
        if !no_breakpoints && self.breakpoints.contains(&self.state.pc()) {
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

impl<State, Track: Tracker<State>> Executor<State, Track> {
    pub fn new(state: State, tracker: Track) -> Self {
        Executor {
            mutex: parking_lot::Mutex::new(ExecutorState::new(state, tracker)),
        }
    }

    pub fn from_state(state: State) -> Executor<State, EmptyTracker> {
        Executor {
            mutex: parking_lot::Mutex::new(ExecutorState::new(state, EmptyTracker)),
        }
    }

    pub fn frame<Reg, Mem: Memory>(&self) -> DebugFrame<Reg> where State: ExecutableState<Reg, Mem> {
        self.mutex.lock().frame()
    }

    pub fn pause(&self) {
        self.mutex.lock().mode = Paused
    }

    pub fn override_mode(&self, mode: ExecutorMode) {
        self.mutex.lock().mode = mode
    }

    pub fn with_state<T, F: FnOnce(&mut State) -> T>(&self, f: F) -> T {
        let mut lock = self.mutex.lock();

        f(&mut lock.state)
    }

    pub fn with_memory<T, Reg, Mem: Memory, F: FnOnce(&mut Mem) -> T>(&self, f: F) -> T where State: ExecutableState<Reg, Mem> {
        let mut lock = self.mutex.lock();

        f(lock.state.memory_mut())
    }

    pub fn with_tracker<T, F: FnOnce(&mut Track) -> T>(&self, f: F) -> T {
        let mut lock = self.mutex.lock();

        f(&mut lock.tracker)
    }

    // Instruction Size - What to add to PC to get the next instruction.
    pub fn syscall_handled<Reg, Mem: Memory>(&self, instruction_size: u32) where State: ExecutableState<Reg, Mem> {
        let mut lock = self.mutex.lock();

        if let Invalid(_) = lock.mode {
            lock.mode = Running
        }

        let new_pc = lock.state.pc() + instruction_size; // !
        lock.state.set_pc(new_pc);
    }

    pub fn set_breakpoints(&self, breakpoints: Breakpoints) {
        let mut lock = self.mutex.lock();

        lock.breakpoints = breakpoints
    }

    // Returns true if CPU was interrupted.
    pub fn cycle<Reg, Mem: Memory>(&self, no_breakpoints: bool) -> bool where State: ExecutableState<Reg, Mem> {
        self.mutex.lock().cycle(no_breakpoints)
    }

    pub fn is_breakpoint(&self) -> bool {
        self.mutex.lock().mode == Breakpoint
    }

    // Returns true if the CPU was interrupted.
    pub fn run_batched<Reg, Mem: Memory>(
        &self,
        batch: usize,
        mut skip_first_breakpoint: bool,
        allow_interrupt: bool,
    ) -> BatchResult where State: ExecutableState<Reg, Mem> {
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

    pub fn run<Reg, Mem: Memory>(&self, mut skip_first_breakpoint: bool) -> DebugFrame<Reg> where State: ExecutableState<Reg, Mem> {
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
