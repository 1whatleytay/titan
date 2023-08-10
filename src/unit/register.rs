use std::fmt::{Display, Formatter};
use crate::cpu::state::Registers;
use num_derive::{ToPrimitive, FromPrimitive};
use num_traits::ToPrimitive;

#[derive(Copy, Clone, Debug, PartialEq, Eq, ToPrimitive, FromPrimitive)]
pub enum RegisterName {
    Zero = 0, AT = 1,
    V0 = 2, V1 = 3, A0 = 4, A1 = 5, A2 = 6, A3 = 7,
    T0 = 8, T1 = 9, T2 = 10, T3 = 11, T4 = 12, T5 = 13, T6 = 14, T7 = 15,
    S0 = 16, S1 = 17, S2 = 18, S3 = 19, S4 = 20, S5 = 21, S6 = 22, S7 = 23,
    T8 = 24, T9 = 25, K0 = 26, K1 = 27,
    GP = 28, SP = 29, FP = 30, RA = 31,
}

impl RegisterName {
    fn to_str(&self) -> &str {
        match self {
            RegisterName::Zero => "zero",
            RegisterName::AT => "at",
            RegisterName::V0 => "v0",
            RegisterName::V1 => "v1",
            RegisterName::A0 => "a0",
            RegisterName::A1 => "a1",
            RegisterName::A2 => "a2",
            RegisterName::A3 => "a3",
            RegisterName::T0 => "t0",
            RegisterName::T1 => "t1",
            RegisterName::T2 => "t2",
            RegisterName::T3 => "t3",
            RegisterName::T4 => "t4",
            RegisterName::T5 => "t5",
            RegisterName::T6 => "t6",
            RegisterName::T7 => "t7",
            RegisterName::S0 => "s0",
            RegisterName::S1 => "s1",
            RegisterName::S2 => "s2",
            RegisterName::S3 => "s3",
            RegisterName::S4 => "s4",
            RegisterName::S5 => "s5",
            RegisterName::S6 => "s6",
            RegisterName::S7 => "s7",
            RegisterName::T8 => "t8",
            RegisterName::T9 => "t9",
            RegisterName::K0 => "k0",
            RegisterName::K1 => "k1",
            RegisterName::GP => "gp",
            RegisterName::SP => "sp",
            RegisterName::FP => "fp",
            RegisterName::RA => "ra",
        }
    }
}

impl Display for RegisterName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.to_str())
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
