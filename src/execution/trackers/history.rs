use crate::cpu::memory::watched::{WatchEntry, WatchedMemory, LOG_SIZE};
use crate::cpu::registers::watched::REGISTER_LOG_SIZE;
use crate::cpu::registers::{RegisterEntry, Registers, WatchedRegisters, WhichRegister};
use crate::cpu::{Memory, State};
use crate::execution::trackers::Tracker;
use smallvec::SmallVec;
use std::collections::VecDeque;
use WhichRegister::Pc;

impl RegisterEntry {
    pub fn apply<Reg: Registers>(self, registers: &mut Reg) {
        let RegisterEntry(name, value) = self;
        registers.set(name, value);
    }
}

pub struct HistoryEntry {
    pub registers: SmallVec<[RegisterEntry; REGISTER_LOG_SIZE]>,
    pub edits: SmallVec<[WatchEntry; LOG_SIZE]>,
}

impl HistoryEntry {
    pub fn apply<Mem: Memory, Reg: Registers>(self, registers: &mut Reg, memory: &mut Mem) {
        for entry in self.registers.iter().rev() {
            entry.apply(registers);
        }
        registers.set(Pc, registers.get(Pc).wrapping_sub(4));

        for entry in self.edits {
            entry.apply(memory).ok(); // ignore error
        }
    }
}

pub struct HistoryTracker {
    buffer: VecDeque<HistoryEntry>,
}

impl HistoryTracker {
    pub fn new(capacity: usize) -> HistoryTracker {
        HistoryTracker {
            buffer: VecDeque::with_capacity(capacity),
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

impl<Mem: Memory> Tracker<WatchedMemory<Mem>, WatchedRegisters> for HistoryTracker {
    fn pre_track(&mut self, state: &mut State<WatchedMemory<Mem>, WatchedRegisters>) {}

    fn post_track(&mut self, state: &mut State<WatchedMemory<Mem>, WatchedRegisters>) {
        let entry = HistoryEntry {
            registers: state.registers.take(),
            edits: state.memory.take(),
        };

        self.push(entry);
    }
}
