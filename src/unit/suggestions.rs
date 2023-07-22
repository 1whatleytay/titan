use crate::cpu::state::Registers;
use crate::unit::instruction::Instruction;
use crate::unit::instruction::Instruction::{Add, Addi, Div, Divu, Lb, Lbu, Lh, Lhu, Lw, Sb, Sh, Sub, Sw};
use crate::unit::register::RegisterName;
use crate::unit::suggestions::TrapErrorReason::{DivByZero, OverflowAdd, OverflowOther, OverflowSub};

pub struct RegisterValue {
    pub name: RegisterName,
    pub value: u32
}

pub struct MemoryErrorDescription {
    pub source: RegisterValue,
    pub immediate: u16
}

impl Registers {
    fn value(&self, name: RegisterName) -> RegisterValue {
        RegisterValue {
            name,
            value: self.get(name)
        }
    }
}

pub enum TrapErrorReason {
    OverflowAdd,
    OverflowSub,
    OverflowOther,
    DivByZero,
}

pub enum RegisterImmediate {
    Value(RegisterValue),
    Immediate(u16)
}

pub struct TrapErrorDescription {
    pub reason: TrapErrorReason,
    pub source: RegisterValue,
    pub temp: RegisterImmediate,
}

impl MemoryErrorDescription {
    fn new(source: RegisterName, immediate: u16, registers: &Registers) -> MemoryErrorDescription {
        MemoryErrorDescription {
            source: registers.value(source),
            immediate
        }
    }
}

impl TrapErrorDescription {
    fn from_temp(reason: TrapErrorReason, source: RegisterName, temp: RegisterName, registers: &Registers) -> TrapErrorDescription {
        TrapErrorDescription {
            reason,
            source: registers.value(source),
            temp: RegisterImmediate::Value(registers.value(temp))
        }
    }

    fn from_imm(reason: TrapErrorReason, source: RegisterName, imm: u16, registers: &Registers) -> TrapErrorDescription {
        TrapErrorDescription {
            reason,
            source: registers.value(source),
            temp: RegisterImmediate::Immediate(imm)
        }
    }
}

// Keeping error suggestions separate from interpreting to avoid potential performance impacts.
impl Instruction {
    pub fn describe_memory_error(&self, registers: &Registers) -> Option<MemoryErrorDescription> {
        Some(match self {
            Lb { s, imm, .. }
                | Lbu { s, imm, .. }
                | Lh { s, imm, .. }
                | Lhu { s, imm, .. }
                | Lw { s, imm, .. }
                | Sb { s, imm, ..}
                | Sh { s, imm, ..}
                | Sw { s, imm, ..} =>
                MemoryErrorDescription::new(*s, *imm, registers),
            _ => return None
        })
    }

    pub fn describe_trap_error(&self, registers: &Registers) -> Option<TrapErrorDescription> {
        Some(match self {
            Add { s, t, .. } =>
                TrapErrorDescription::from_temp(OverflowAdd, *s, *t, registers),
            Addi { s, imm, .. } =>
                TrapErrorDescription::from_imm(OverflowAdd, *s, *imm, registers),
            Sub { s, t, .. } =>
                TrapErrorDescription::from_temp(OverflowSub, *s, *t, registers),
            Div { s, t }
                | Divu { s, t } =>
                TrapErrorDescription::from_temp(DivByZero, *s, *t, registers),
            Instruction::Madd { s, t, .. }
                | Instruction::Msub { s, t, .. } =>
                TrapErrorDescription::from_temp(OverflowOther, *s, *t, registers),
            _ => return None
        })
    }
}