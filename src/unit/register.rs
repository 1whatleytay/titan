pub use crate::assembler::registers::RegisterSlot as RegisterName;
use crate::cpu::registers::registers::RawRegisters;
use num::ToPrimitive;

impl RawRegisters {
    pub fn get_l(&self, name: RegisterName) -> u32 {
        let index = name.to_usize().unwrap();

        self.line[index]
    }

    pub fn set_l(&mut self, name: RegisterName, value: u32) {
        let index = name.to_usize().unwrap();

        self.line[index] = value
    }
}
