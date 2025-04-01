use super::{registers::RawRegisters, Registers, WhichRegister};
use crate::cpu::memory::watched::LOG_SIZE;
use smallvec::SmallVec;

#[derive(Clone, Copy, Debug)]
pub struct RegisterEntry(pub WhichRegister, pub u32);

#[derive(Clone)]
pub struct WatchedRegisters {
    pub backing: RawRegisters,
    pub log: SmallVec<[RegisterEntry; LOG_SIZE]>,
}

impl WatchedRegisters {
    pub fn new(entry: u32) -> Self {
        WatchedRegisters {
            backing: RawRegisters::new(entry),
            log: SmallVec::new(),
        }
    }
    pub fn take(&mut self) -> SmallVec<[RegisterEntry; LOG_SIZE]> {
        std::mem::take(&mut self.log)
    }
}

impl Registers for WatchedRegisters {
    fn get(&self, name: WhichRegister) -> u32 {
        self.backing.get(name)
    }

    fn set(&mut self, name: WhichRegister, value: u32) {
        self.log.push(RegisterEntry(name, value));
        self.backing.set(name, value);
    }
    fn raw(&self) -> RawRegisters {
        self.backing.clone()
    }
}
