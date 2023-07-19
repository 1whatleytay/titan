use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::{fs, thread};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use crate::assembler::binary::Binary;
use crate::assembler::string::{assemble_from_path, SourceError};
use crate::cpu::memory::{Mountable, Region};
use crate::cpu::memory::section::{DefaultResponder, SectionMemory};
use crate::cpu::memory::watched::WatchedMemory;
use crate::cpu::State;
use crate::cpu::state::Registers;
use crate::execution::executor::{DebugFrame, Executor};
use crate::execution::trackers::history::HistoryTracker;
use crate::unit::device::MakeUnitDeviceError::{CompileFailed, FileMissing};
use crate::unit::device::UnitDeviceError::{ExecutionTimedOut, InvalidInstruction, MissingLabel, ProgramCompleted};
use num::{ToPrimitive, FromPrimitive};
use num_derive::{ToPrimitive, FromPrimitive};
use crate::execution::executor::ExecutorMode::Invalid;
use crate::unit::device::StopCondition::{Address, Steps, Timeout};
use crate::cpu::error::Error as CpuError;
use crate::unit::device::RegisterName::{A0, RA, V0};

pub type MemoryType = WatchedMemory<SectionMemory<DefaultResponder>>;
pub type TrackerType = HistoryTracker;

#[derive(Debug)]
pub enum MakeUnitDeviceError {
    CompileFailed(SourceError),
    FileMissing(std::io::Error)
}

impl Display for MakeUnitDeviceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileFailed(e) => Display::fmt(e, f),
            FileMissing(e) => Display::fmt(e, f)
        }
    }
}

impl Error for MakeUnitDeviceError { }

pub struct UnitDevice {
    pub executor: Arc<Executor<MemoryType, TrackerType>>,
    pub binary: Binary,
    pub finished_pcs: Vec<u32>,
    handlers: HashMap<u32, Box<dyn Fn ()>>,
}

#[derive(Clone, Debug)]
pub struct LabelIdentifier {
    pub name: String,
    pub offset: i64
}

impl From<&str> for LabelIdentifier {
    fn from(value: &str) -> Self {
        LabelIdentifier { name: value.to_string(), offset: 0 }
    }
}

#[derive(Clone, Debug)]
pub enum StopCondition {
    Address(u32), // PC Address
    MaybeLabel(LabelIdentifier), // Label (if it exists)
    Label(LabelIdentifier), // Label (fail if it doesn't exist)
    Steps(usize), // Number of Instructions to Execute
    Timeout(Duration), // Timeout
    Complete,
    // InstructionMatched
    // Jr Return/Jal
}

struct StopConditionParameters {
    timeout: Option<Duration>,
    steps: Option<usize>,
    breakpoints: Vec<u32>,
    complete_error: bool
}

impl StopConditionParameters {
    pub fn from<F: FnMut(&str) -> Option<u32>>(
        conditions: &[StopCondition], mut get_label: F
    ) -> Result<StopConditionParameters, UnitDeviceError> {
        let timeout = conditions.iter()
            .filter_map(|c| {
                if let Timeout(duration) = c {
                    Some(*duration)
                } else {
                    None
                }
            })
            .min();

        let steps = conditions.iter()
            .filter_map(|c| {
                if let Steps(count) = c {
                    Some(*count)
                } else {
                    None
                }
            })
            .min();

        if let Some(failed) = conditions.iter()
            .filter_map(|c| {
                if let StopCondition::Label(identifier) = c {
                    if get_label(&identifier.name).is_none() {
                        return Some(identifier.name.clone())
                    }
                }

                None
            }).next() {
            return Err(MissingLabel(failed))
        }

        let breakpoints = conditions.iter()
            .filter_map(|c| {
                match c {
                    StopCondition::Address(pc) => Some(*pc),
                    StopCondition::MaybeLabel(identifier)
                        | StopCondition::Label(identifier) => {
                        get_label(&identifier.name)
                            .map(|x| (x as i64 + identifier.offset) as u32)
                    }
                    _ => None
                }
            })
            .collect();

        let complete_error = !conditions.iter()
            .any(|c| matches!(c, StopCondition::Complete));

        Ok(StopConditionParameters {
            timeout,
            steps,
            breakpoints,
            complete_error
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum UnitDeviceError {
    MissingLabel(String),
    ExecutionTimedOut,
    InvalidInstruction(CpuError),
    ProgramCompleted
}

impl Display for UnitDeviceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MissingLabel(label) => write!(f, "Could not find label {} in program", label),
            ExecutionTimedOut => write!(f, "Execution timed out (by stop condition)"),
            InvalidInstruction(error) => write!(f, "Cpu execution failed with error {}", error),
            ProgramCompleted => write!(f, "Program completed and this was not caught")
        }
    }
}

fn make_timeout<F: FnOnce () + Send + 'static>(f: F, duration: Duration) -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    let result = stop.clone();

    let start = Instant::now();

    thread::spawn(move || {
        while start.elapsed() < duration {
            if stop.load(Ordering::Relaxed) {
                return
            }

            thread::sleep(Duration::from_millis(100));
        }

        f()
    });

    result
}

impl Error for UnitDeviceError { }

#[derive(ToPrimitive, FromPrimitive)]
pub enum RegisterName {
    Zero = 0, AT = 1,
    V0 = 2, V1 = 3, A0 = 4, A1 = 5, A2 = 6, A3 = 7,
    T0 = 8, T1 = 9, T2 = 10, T3 = 11, T4 = 12, T5 = 13, T6 = 14, T7 = 15,
    S0 = 16, S1 = 17, S2 = 18, S3 = 19, S4 = 20, S5 = 21, S6 = 22, S7 = 23,
    T8 = 24, T9 = 25, K0 = 26, K1 = 27,
    GP = 28, SP = 29, FP = 30, RA = 31,
}

impl Registers {
    pub fn get(&self, name: RegisterName) -> u32 {
        let index = ToPrimitive::to_usize(&name).unwrap();

        self.line[index]
    }

    pub fn set(&mut self, name: RegisterName, value: u32) {
        let index = ToPrimitive::to_usize(&name).unwrap();

        self.line[index] = value
    }
}

impl UnitDevice {
    pub fn make(path: PathBuf) -> Result<UnitDevice, MakeUnitDeviceError> {
        let source = fs::read_to_string(&path).map_err(FileMissing)?;
        let binary = assemble_from_path(source, path).map_err(CompileFailed)?;

        let mut memory = WatchedMemory::new(SectionMemory::new());

        let heap_size = 0x100000;

        for header in &binary.regions {
            let region = Region {
                start: header.address,
                data: header.data.clone(),
            };

            memory.mount(region)
        }

        let heap_end = 0x7FFFFFFCu32;

        let heap = Region {
            start: heap_end - heap_size,
            data: vec![0; heap_size as usize],
        };

        memory.mount(heap);

        let mut state = State::new(binary.entry, memory);
        state.registers.line[29] = heap_end;

        let tracker = HistoryTracker::new(1000);

        let executor = Arc::new(Executor::new(state, tracker));


        let finished_pcs = binary
            .regions
            .iter()
            .map(|region| region.address + region.data.len() as u32)
            .collect();

        Ok(UnitDevice { executor, binary, handlers: HashMap::new(), finished_pcs })
    }

    pub fn registers(&self) -> Registers {
        self.executor.with_state(|s| s.registers)
    }

    pub fn get(&self, name: RegisterName) -> u32 {
        self.executor.with_state(|s| s.registers.get(name))
    }

    pub fn set(&self, name: RegisterName, value: u32) {
        self.executor.with_state(|s| s.registers.set(name, value))
    }

    pub fn has_label(&self, name: &str) -> bool {
        self.binary.labels.contains_key(name)
    }

    pub fn arrived_at_label(&self, name: &str) -> bool {
        self.binary.labels.get(name).map(
            |v| self.executor.with_state(|s| s.registers.pc == *v)
        ).unwrap_or(false)
    }

    pub fn jump_to(&self, pc: u32) {
        self.executor.with_state(|s| s.registers.pc = pc)
    }

    pub fn jump_to_label(&self, name: &str) -> Result<(), UnitDeviceError> {
        let Some(value) = self.binary.labels.get(name) else {
            return Err(MissingLabel(name.to_string()))
        };

        self.jump_to(*value);

        Ok(())
    }

    pub fn snapshot(&self) -> State<MemoryType> {
        self.executor.with_state(|s| s.clone())
    }

    pub fn restore(&self, state: State<MemoryType>) {
        self.executor.with_state(|s| *s = state)
    }

    pub fn handle_syscall<F: Fn() + 'static>(&mut self, v0: u32, f: F) {
        self.handlers.insert(v0, Box::new(f));
    }

    pub fn handle_frame(&self, frame: &DebugFrame, complete_error: bool) -> Result<bool, UnitDeviceError> {
        match frame.mode {
            Invalid(error) => match error {
                CpuError::CpuSyscall => {
                    let v0 = self.executor.with_state(|s| s.registers.get(V0));

                    if let Some(handler) = self.handlers.get(&v0) {
                        handler();

                        self.executor.invalid_handled();

                        Ok(false)
                    } else {
                        Err(InvalidInstruction(error))
                    }
                }

                _ => {
                    if self.finished_pcs.contains(&frame.registers.pc) {
                        if complete_error {
                            Err(ProgramCompleted)
                        } else {
                            Ok(true)
                        }
                    } else {
                        Err(InvalidInstruction(error))
                    }
                }
            },

            _ => Ok(true)
        }
    }

    pub fn step(&self) -> Result<(), UnitDeviceError> {
        self.execute_until([Steps(1)])
    }

    pub fn backstep(&self) -> bool {
        let Some(entry) = self.executor.with_tracker(|tracker| tracker.pop()) else {
            return false
        };

        self.executor.with_state(|state| {
            entry.apply(&mut state.registers, &mut state.memory.backing);
        });

        true
    }

    pub fn load_params(&self, params: &[u32]) {
        for (index, value) in params.iter().enumerate() {
            let index = index + A0.to_usize().unwrap();

            if index >= 32 {
                return
            }

            let index = FromPrimitive::from_usize(index).unwrap();

            self.set(index, *value)
        }
    }

    pub fn call_slice(&self, label: &str, params: &[u32], timeout: Option<Duration>) -> Result<(), UnitDeviceError> {
        self.jump_to_label(label)?;

        let last_ra = self.registers().get(RA);
        let return_address = 0xEABADDEA;

        self.executor.with_state(|s| s.registers.set(RA, return_address));

        self.load_params(params);

        if let Some(duration) = timeout {
            self.execute_until([
                Address(return_address),
                Timeout(duration)
            ])?;
        } else {
            self.execute_until([
                Address(return_address)
            ])?;
        }

        self.executor.with_state(|s| s.registers.set(RA, last_ra));

        Ok(())
    }

    pub fn call<const N: usize>(&self, label: &str, params: [u32; N], timeout: Option<Duration>) -> Result<(), UnitDeviceError> {
        self.call_slice(label, &params, timeout)
    }

    pub fn execute_until_slice(&self, conditions: &[StopCondition]) -> Result<(), UnitDeviceError> {
        let parameters = StopConditionParameters::from(
            conditions, |s| self.binary.labels.get(s).copied()
        )?;

        self.executor.set_breakpoints(parameters.breakpoints.into_iter().collect());

        let did_timeout = Arc::new(AtomicBool::new(false));
        let did_timeout_clone = did_timeout.clone();

        let cancel = parameters.timeout.map(move |duration| {
            let executor = self.executor.clone();

            make_timeout(move || {
                did_timeout_clone.store(true, Ordering::Relaxed);

                executor.pause();
            }, duration)
        });

        loop {
            let frame = if let Some(count) = parameters.steps {
                self.executor.run_limited::<true>(count)
            } else {
                self.executor.run()
            };

            if self.handle_frame(&frame, parameters.complete_error)? {
                break
            }
        }

        if let Some(cancel) = cancel {
            cancel.store(true, Ordering::Relaxed)
        }

        if did_timeout.load(Ordering::Relaxed) {
            return Err(ExecutionTimedOut)
        }

        Ok(())
    }

    pub fn execute_until<const N: usize>(&self, conditions: [StopCondition; N]) -> Result<(), UnitDeviceError> {
        self.execute_until_slice(&conditions)
    }
}
