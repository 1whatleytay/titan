use num_derive::{FromPrimitive, ToPrimitive};
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, Eq, ToPrimitive, FromPrimitive)]
pub enum RegisterSlot {
    Zero = 0,
    AssemblerTemporary = 1,
    Value0 = 2,
    Value1 = 3,
    Parameter0 = 4,
    Parameter1 = 5,
    Parameter2 = 6,
    Parameter3 = 7,
    Temporary0 = 8,
    Temporary1 = 9,
    Temporary2 = 10,
    Temporary3 = 11,
    Temporary4 = 12,
    Temporary5 = 13,
    Temporary6 = 14,
    Temporary7 = 15,
    Saved0 = 16,
    Saved1 = 17,
    Saved2 = 18,
    Saved3 = 19,
    Saved4 = 20,
    Saved5 = 21,
    Saved6 = 22,
    Saved7 = 23,
    Temporary8 = 24,
    Temporary9 = 25,
    Kernel0 = 26,
    Kernel1 = 27,
    GeneralPointer = 28,
    StackPointer = 29,
    FramePointer = 30,
    ReturnAddress = 31,
}

impl RegisterSlot {
    pub fn from_string(input: &str) -> Option<RegisterSlot> {
        Some(match input {
            "zero" => RegisterSlot::Zero,
            "at" => RegisterSlot::AssemblerTemporary,
            "v0" => RegisterSlot::Value0,
            "v1" => RegisterSlot::Value1,
            "a0" => RegisterSlot::Parameter0,
            "a1" => RegisterSlot::Parameter1,
            "a2" => RegisterSlot::Parameter2,
            "a3" => RegisterSlot::Parameter3,
            "t0" => RegisterSlot::Temporary0,
            "t1" => RegisterSlot::Temporary1,
            "t2" => RegisterSlot::Temporary2,
            "t3" => RegisterSlot::Temporary3,
            "t4" => RegisterSlot::Temporary4,
            "t5" => RegisterSlot::Temporary5,
            "t6" => RegisterSlot::Temporary6,
            "t7" => RegisterSlot::Temporary7,
            "s0" => RegisterSlot::Saved0,
            "s1" => RegisterSlot::Saved1,
            "s2" => RegisterSlot::Saved2,
            "s3" => RegisterSlot::Saved3,
            "s4" => RegisterSlot::Saved4,
            "s5" => RegisterSlot::Saved5,
            "s6" => RegisterSlot::Saved6,
            "s7" => RegisterSlot::Saved7,
            "t8" => RegisterSlot::Temporary8,
            "t9" => RegisterSlot::Temporary9,
            "k0" => RegisterSlot::Kernel0,
            "k1" => RegisterSlot::Kernel1,
            "gp" => RegisterSlot::GeneralPointer,
            "sp" => RegisterSlot::StackPointer,
            "fp" => RegisterSlot::FramePointer,
            "ra" => RegisterSlot::ReturnAddress,

            _ => return None,
        })
    }

    pub fn as_string(&self) -> &str {
        match self {
            RegisterSlot::Zero => "zero",
            RegisterSlot::AssemblerTemporary => "at",
            RegisterSlot::Value0 => "v0",
            RegisterSlot::Value1 => "v1",
            RegisterSlot::Parameter0 => "a0",
            RegisterSlot::Parameter1 => "a1",
            RegisterSlot::Parameter2 => "a2",
            RegisterSlot::Parameter3 => "a3",
            RegisterSlot::Temporary0 => "t0",
            RegisterSlot::Temporary1 => "t1",
            RegisterSlot::Temporary2 => "t2",
            RegisterSlot::Temporary3 => "t3",
            RegisterSlot::Temporary4 => "t4",
            RegisterSlot::Temporary5 => "t5",
            RegisterSlot::Temporary6 => "t6",
            RegisterSlot::Temporary7 => "t7",
            RegisterSlot::Saved0 => "s0",
            RegisterSlot::Saved1 => "s1",
            RegisterSlot::Saved2 => "s2",
            RegisterSlot::Saved3 => "s3",
            RegisterSlot::Saved4 => "s4",
            RegisterSlot::Saved5 => "s5",
            RegisterSlot::Saved6 => "s6",
            RegisterSlot::Saved7 => "s7",
            RegisterSlot::Temporary8 => "t8",
            RegisterSlot::Temporary9 => "t9",
            RegisterSlot::Kernel0 => "k0",
            RegisterSlot::Kernel1 => "k1",
            RegisterSlot::GeneralPointer => "gp",
            RegisterSlot::StackPointer => "sp",
            RegisterSlot::FramePointer => "fp",
            RegisterSlot::ReturnAddress => "ra",
        }
    }
}

impl Display for RegisterSlot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.as_string())
    }
}
