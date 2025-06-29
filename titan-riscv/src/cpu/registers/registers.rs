use crate::assembler::registers::RegisterSlot;
use crate::cpu::decoder::InstructionSize;
use num::ToPrimitive;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WhichRegister {
    Pc,
    Line(u8),
}

pub trait Registers {
    fn get(&self, name: WhichRegister) -> u32;
    fn set(&mut self, name: WhichRegister, value: u32);

    fn step_pc(&mut self, size: InstructionSize);

    fn clear(&mut self);

    fn raw(&self) -> RawRegisters;

    #[inline]
    fn get_l(&self, name: RegisterSlot) -> u32 {
        let index = name.to_u8().unwrap();

        self.get(WhichRegister::Line(index))
    }

    #[inline]
    fn set_l(&mut self, name: RegisterSlot, value: u32) {
        let index = name.to_u8().unwrap();

        self.set(WhichRegister::Line(index), value)
    }
}

#[derive(Clone, Debug, Default)]
pub struct RawRegisters {
    pub pc: u32,
    pub line: [u32; 32],
}

impl Registers for RawRegisters {
    #[inline]
    fn get(&self, name: WhichRegister) -> u32 {
        match name {
            WhichRegister::Pc => self.pc,
            WhichRegister::Line(index) => self.line[index as usize],
        }
    }

    #[inline]
    fn set(&mut self, name: WhichRegister, value: u32) {
        match name {
            WhichRegister::Pc => self.pc = value,
            WhichRegister::Line(index) => self.line[index as usize] = value,
        }
    }

    #[inline]
    fn step_pc(&mut self, size: InstructionSize) {
        self.pc = self.pc.wrapping_add(size.byte_size());
    }

    fn raw(&self) -> RawRegisters {
        self.clone()
    }

    fn clear(&mut self) {}
}
