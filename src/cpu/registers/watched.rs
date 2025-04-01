use super::{registers::RawRegisters, Registers, WhichRegister};
use smallvec::SmallVec;

pub const REGISTER_LOG_SIZE: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct RegisterEntry(pub WhichRegister, pub u32);

#[derive(Clone, Default)]
pub struct WatchedRegisters {
    pub backing: RawRegisters,
    pub log: SmallVec<[RegisterEntry; REGISTER_LOG_SIZE]>,
}

impl WatchedRegisters {
    pub fn take(&mut self) -> SmallVec<[RegisterEntry; REGISTER_LOG_SIZE]> {
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
