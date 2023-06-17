use std::iter::repeat_with;
use smallvec::SmallVec;
use crate::cpu::{Memory, State};
use crate::cpu::memory::watched::{WatchedMemory, WatchEntry};
use crate::cpu::state::Registers;
use crate::debug::trackers::Tracker;

pub struct HistoryEntry {
    pub registers: Registers,
    pub edits: SmallVec<[WatchEntry; 4]>
}

pub struct HistoryTracker {
    buffer: Vec<Option<HistoryEntry>>,
    next: usize,
    count: usize,
    registers: Option<Registers>
}

impl HistoryEntry {
    pub fn apply<Mem: Memory>(self, state: &mut State<Mem>) {
        state.registers = self.registers;

        for entry in self.edits {
            entry.apply(&mut state.memory).ok(); // ignore error
        }
    }
}

impl HistoryTracker {
    pub fn new(capacity: usize) -> HistoryTracker {
        HistoryTracker {
            buffer: repeat_with(|| None).take(capacity).collect(),
            next: 0,
            count: 0,
            registers: None
        }
    }

    fn push(&mut self, entry: HistoryEntry) {
        self.buffer[self.next] = Some(entry);

        self.next += 1;
        self.count += 1;

        if self.next >= self.buffer.len() {
            self.next = 0;
        }
    }

    pub fn pop(&mut self) -> Option<HistoryEntry> {
        self.next = self.next.checked_sub(1).unwrap_or(self.buffer.len() - 1);
        self.count = self.count.checked_sub(1).unwrap_or(0);

        self.buffer[self.next].take()
    }

    pub fn len(&self) -> usize {
        self.count
    }
}

impl<Mem: Memory> Tracker<WatchedMemory<Mem>> for HistoryTracker {
    fn pre_track(&mut self, state: &mut State<WatchedMemory<Mem>>) {
        self.registers = Some(state.registers.clone())
    }

    fn post_track(&mut self, state: &mut State<WatchedMemory<Mem>>) {
        let Some(registers) = self.registers else { return };
        let entry = HistoryEntry { registers, edits: state.memory.take() };

        self.push(entry);
    }
}