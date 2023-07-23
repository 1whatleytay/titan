use std::fmt::{Display, Formatter};
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

impl ToString for RegisterName {
    fn to_string(&self) -> String {
        match self {
            RegisterName::Zero => "zero".to_string(),
            RegisterName::AT => "at".to_string(),
            RegisterName::V0 => "v0".to_string(),
            RegisterName::V1 => "v1".to_string(),
            RegisterName::A0 => "a0".to_string(),
            RegisterName::A1 => "a1".to_string(),
            RegisterName::A2 => "a2".to_string(),
            RegisterName::A3 => "a3".to_string(),
            RegisterName::T0 => "t0".to_string(),
            RegisterName::T1 => "t1".to_string(),
            RegisterName::T2 => "t2".to_string(),
            RegisterName::T3 => "t3".to_string(),
            RegisterName::T4 => "t4".to_string(),
            RegisterName::T5 => "t5".to_string(),
            RegisterName::T6 => "t6".to_string(),
            RegisterName::T7 => "t7".to_string(),
            RegisterName::S0 => "s0".to_string(),
            RegisterName::S1 => "s1".to_string(),
            RegisterName::S2 => "s2".to_string(),
            RegisterName::S3 => "s3".to_string(),
            RegisterName::S4 => "s4".to_string(),
            RegisterName::S5 => "s5".to_string(),
            RegisterName::S6 => "s6".to_string(),
            RegisterName::S7 => "s7".to_string(),
            RegisterName::T8 => "t8".to_string(),
            RegisterName::T9 => "t9".to_string(),
            RegisterName::K0 => "k0".to_string(),
            RegisterName::K1 => "k1".to_string(),
            RegisterName::GP => "gp".to_string(),
            RegisterName::SP => "sp".to_string(),
            RegisterName::FP => "fp".to_string(),
            RegisterName::RA => "ra".to_string(),
        }
    }
}

impl Display for RegisterName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.to_string())
    }
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
