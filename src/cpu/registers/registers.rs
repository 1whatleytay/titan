use crate::unit::register::RegisterName;
use num::ToPrimitive;

#[derive(Copy, Clone, Debug)]
pub enum WhichRegister {
    Pc,
    Line(u8),
    Lo,
    Hi,
    Fp(u8),
    Cf,
}

pub trait Registers {
    fn get(&self, name: WhichRegister) -> u32;
    fn set(&mut self, name: WhichRegister, value: u32);
    fn raw(&self) -> RawRegisters;

    fn get_l(&self, name: RegisterName) -> u32 {
        let index = name.to_u8().unwrap();

        self.get(WhichRegister::Line(index))
    }

    fn set_l(&mut self, name: RegisterName, value: u32) {
        let index = name.to_u8().unwrap();

        self.set(WhichRegister::Line(index), value)
    }
}

#[derive(Clone, Debug)]
pub struct RawRegisters {
    pub pc: u32,
    pub line: [u32; 32],
    pub lo: u32,
    pub hi: u32,
    // Coprocessor 1: FPU
    pub fp: [u32; 32],
    pub cf: u32,
}

impl RawRegisters {
    pub fn new(entry: u32) -> RawRegisters {
        RawRegisters {
            pc: entry,
            line: [0; 32],
            lo: 0,
            hi: 0,
            fp: [0; 32],
            cf: 0,
        }
    }
}

impl Registers for RawRegisters {
    fn get(&self, name: WhichRegister) -> u32 {
        match name {
            WhichRegister::Pc => self.pc,
            WhichRegister::Line(index) => self.line[index as usize],
            WhichRegister::Lo => self.lo,
            WhichRegister::Hi => self.hi,
            WhichRegister::Fp(index) => self.fp[index as usize],
            WhichRegister::Cf => self.cf,
        }
    }

    fn set(&mut self, name: WhichRegister, value: u32) {
        match name {
            WhichRegister::Pc => self.pc = value,
            WhichRegister::Line(index) => self.line[index as usize] = value,
            WhichRegister::Lo => self.lo = value,
            WhichRegister::Hi => self.hi = value,
            WhichRegister::Fp(index) => self.fp[index as usize] = value,
            WhichRegister::Cf => self.cf = value,
        }
    }

    fn raw(&self) -> RawRegisters {
        self.clone()
    }
}
