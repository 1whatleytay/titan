pub use crate::assembler::registers::RegisterSlot as RegisterName;
use crate::cpu::state::Registers;
use num::ToPrimitive;

impl Registers {
    pub fn get(&self, name: RegisterName) -> u32 {
        let index = name.to_usize().unwrap();

        self.line[index]
    }

    pub fn set(&mut self, name: RegisterName, value: u32) {
        let index = name.to_usize().unwrap();

        self.line[index] = value
    }
}
