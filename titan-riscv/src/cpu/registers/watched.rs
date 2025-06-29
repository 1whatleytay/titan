use crate::cpu::decoder::InstructionSize;
use crate::cpu::registers::registers::{RawRegisters, Registers, WhichRegister};
use smallvec::SmallVec;

pub const REGISTER_LOG_SIZE: usize = 1;

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
    #[inline]
    fn get(&self, name: WhichRegister) -> u32 {
        self.backing.get(name)
    }

    #[inline]
    fn set(&mut self, name: WhichRegister, value: u32) {
        let old_value = self.backing.get(name);
        self.log.push(RegisterEntry(name, old_value));
        self.backing.set(name, value);
    }

    #[inline]
    fn step_pc(&mut self, size: InstructionSize) {
        // If we aren't just using a typical regular jump, our back step will be different.
        if !matches!(size, InstructionSize::Regular) {
            self.log
                .push(RegisterEntry(WhichRegister::Pc, self.backing.pc))
        }

        self.backing.step_pc(size);
    }

    fn clear(&mut self) {
        self.log.clear();
    }

    fn raw(&self) -> RawRegisters {
        self.backing.clone()
    }
}
