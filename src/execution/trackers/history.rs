use crate::cpu::memory::watched::{WatchEntry, WatchedMemory, LOG_SIZE};
use crate::cpu::state::Registers;
use crate::cpu::{Memory, State};
use crate::execution::trackers::Tracker;
use smallvec::SmallVec;
use std::collections::VecDeque;

pub struct HistoryEntry {
    pub registers: Registers,
    pub edits: SmallVec<[WatchEntry; LOG_SIZE]>,
}

impl HistoryEntry {
    pub fn apply<Mem: Memory>(self, registers: &mut Registers, memory: &mut Mem) {
        *registers = self.registers;

        for entry in self.edits {
            entry.apply(memory).ok(); // ignore error
        }
    }
}

pub struct HistoryTracker {
    buffer: VecDeque<HistoryEntry>,
    registers: Option<Registers>,
}

impl HistoryTracker {
    pub fn new(capacity: usize) -> HistoryTracker {
        HistoryTracker {
            buffer: VecDeque::with_capacity(capacity),
            registers: None,
        }
    }

    fn push(&mut self, entry: HistoryEntry) {
        if self.buffer.capacity() == self.buffer.len() {
            self.buffer.pop_front();
        }
        self.buffer.push_back(entry);
    }

    pub fn pop(&mut self) -> Option<HistoryEntry> {
        self.buffer.pop_back()
    }

    pub fn last(&mut self) -> Option<&HistoryEntry> {
        self.buffer.back()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl<Mem: Memory> Tracker<WatchedMemory<Mem>> for HistoryTracker {
    fn pre_track(&mut self, state: &mut State<WatchedMemory<Mem>>) {
        self.registers = Some(state.registers.clone())
    }

    fn post_track(&mut self, state: &mut State<WatchedMemory<Mem>>) {
        let Some(registers) = self.registers else {
            return;
        };
        let entry = HistoryEntry {
            registers,
            edits: state.memory.take(),
        };

        self.push(entry);
    }
}
