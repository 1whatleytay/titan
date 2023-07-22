use crate::cpu::state::Registers;
use num_derive::{ToPrimitive, FromPrimitive};
use num_traits::ToPrimitive;

#[derive(Copy, Clone, Debug, ToPrimitive, FromPrimitive)]
pub enum RegisterName {
    Zero = 0, AT = 1,
    V0 = 2, V1 = 3, A0 = 4, A1 = 5, A2 = 6, A3 = 7,
    T0 = 8, T1 = 9, T2 = 10, T3 = 11, T4 = 12, T5 = 13, T6 = 14, T7 = 15,
    S0 = 16, S1 = 17, S2 = 18, S3 = 19, S4 = 20, S5 = 21, S6 = 22, S7 = 23,
    T8 = 24, T9 = 25, K0 = 26, K1 = 27,
    GP = 28, SP = 29, FP = 30, RA = 31,
}

impl Registers {
    pub fn get(&self, name: RegisterName) -> u32 {
        let index = ToPrimitive::to_usize(&name).unwrap();

        self.line[index]
    }

    pub fn set(&mut self, name: RegisterName, value: u32) {
        let index = ToPrimitive::to_usize(&name).unwrap();

        self.line[index] = value
    }
}
