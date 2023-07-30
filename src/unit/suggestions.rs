use std::fmt::{Display, Formatter};
use crate::cpu::state::Registers;
use crate::unit::instruction::{Instruction, sig, sig_u32};
use crate::unit::instruction::Instruction::{Add, Addi, Div, Divu, Lb, Lbu, Lh, Lhu, Lw, Sb, Sh, Sub, Sw};
use crate::unit::register::RegisterName;
use crate::unit::suggestions::TrapErrorReason::{DivByZero, OverflowAdd, OverflowOther, OverflowSub};

pub enum MemoryErrorReason {
    Unmapped,
    Alignment
}

pub struct MemoryErrorDescription {
    pub instruction: Instruction,
    pub reason: MemoryErrorReason,
    pub alignment: u32,
    pub source: RegisterValue,
    pub immediate: u16
}

pub struct RegisterValue {
    pub name: RegisterName,
    pub value: u32
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
    pub instruction: Instruction,
    pub reason: TrapErrorReason,
    pub source: RegisterValue,
    pub temp: RegisterImmediate,
}

impl MemoryErrorDescription {
    fn new(
        instruction: Instruction, reason: MemoryErrorReason, alignment: u32, source: RegisterName, immediate: u16, registers: &Registers
    ) -> MemoryErrorDescription {
        MemoryErrorDescription {
            instruction,
            reason,
            alignment,
            source: registers.value(source),
            immediate
        }
    }
}

impl TrapErrorDescription {
    fn from_temp(
        instruction: Instruction, reason: TrapErrorReason, source: RegisterName, temp: RegisterName, registers: &Registers
    ) -> TrapErrorDescription {
        TrapErrorDescription {
            instruction,
            reason,
            source: registers.value(source),
            temp: RegisterImmediate::Value(registers.value(temp))
        }
    }

    fn from_imm(
        instruction: Instruction, reason: TrapErrorReason, source: RegisterName, imm: u16, registers: &Registers
    ) -> TrapErrorDescription {
        TrapErrorDescription {
            instruction,
            reason,
            source: registers.value(source),
            temp: RegisterImmediate::Immediate(imm)
        }
    }
}

// Keeping error suggestions separate from interpreting to avoid potential performance impacts.
impl Instruction {
    pub fn describe_memory_error(&self, reason: MemoryErrorReason, registers: &Registers) -> Option<MemoryErrorDescription> {
        Some(match self {
            Lb { s, imm, .. }
                | Lbu { s, imm, .. }
                | Sb { s, imm, .. } =>
                MemoryErrorDescription::new(self.clone(), reason, 1, *s, *imm, registers),
            Lh { s, imm, .. }
                | Lhu { s, imm, .. }
                | Sh { s, imm, .. } =>
                MemoryErrorDescription::new(self.clone(), reason, 2, *s, *imm, registers),
            Lw { s, imm, .. }
                | Sw { s, imm, .. } =>
                MemoryErrorDescription::new(self.clone(), reason, 4, *s, *imm, registers),
            _ => return None
        })
    }

    pub fn describe_trap_error(&self, registers: &Registers) -> Option<TrapErrorDescription> {
        Some(match self {
            Add { s, t, .. } =>
                TrapErrorDescription::from_temp(self.clone(), OverflowAdd, *s, *t, registers),
            Addi { s, imm, .. } =>
                TrapErrorDescription::from_imm(self.clone(), OverflowAdd, *s, *imm, registers),
            Sub { s, t, .. } =>
                TrapErrorDescription::from_temp(self.clone(), OverflowSub, *s, *t, registers),
            Div { s, t }
                | Divu { s, t } =>
                TrapErrorDescription::from_temp(self.clone(), DivByZero, *s, *t, registers),
            Instruction::Madd { s, t, .. }
                | Instruction::Msub { s, t, .. } =>
                TrapErrorDescription::from_temp(self.clone(), OverflowOther, *s, *t, registers),
            _ => return None
        })
    }
}

impl MemoryErrorDescription {
    fn address(&self) -> u32 {
        (self.source.value as i32)
            .wrapping_add(self.immediate as i16 as i32) as u32
    }
}

impl RegisterValue {
    fn hex_string(&self) -> String {
        format!("{} = 0x{:08x}", self.name, self.value)
    }

    fn signed_string(&self) -> String {
        format!("{} = {}", self.name, sig_u32(self.value))
    }
}

impl RegisterImmediate {
    fn signed_string(&self) -> String {
        match self {
            RegisterImmediate::Value(value) => value.signed_string(),
            RegisterImmediate::Immediate(imm) => sig(*imm)
        }
    }
}

impl Display for MemoryErrorDescription {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.reason {
            MemoryErrorReason::Unmapped => {
                writeln!(f, "Memory access to 0x{:08x} is prohibited,", self.address())?;
                writeln!(f, " > {} ({} + {} = 0x{:08x} is unmapped)", self.instruction, self.source.hex_string(), sig(self.immediate), self.address())?;
                writeln!(f, "Double check to make sure you meant to access this location.")
            },
            MemoryErrorReason::Alignment => {
                writeln!(f, "Memory access to 0x{:08x} must be a multiple of {} for this instruction.", self.address(), self.alignment)?;
                writeln!(f, " > {} ({} + {} = 0x{:08x} is not a multiple of {})", self.instruction, self.source.hex_string(), sig(self.immediate), self.address(), self.alignment)?;
                writeln!(f, "Ensure that the data you are accessing is aligned by {}, or use lb/sb to load/store unaligned bytes.", self.alignment)
            }
        }
    }
}

impl Display for TrapErrorDescription {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.reason {
            OverflowAdd => {
                writeln!(f, "Encountered integer overflow during addition, which is prohibited.")?;
                writeln!(f, " > {} ({} + {} overflows)", self.instruction, self.source.signed_string(), self.temp.signed_string())?;
            }
            OverflowSub => {
                writeln!(f, "Encountered integer overflow during subtraction, which is prohibited.")?;
                writeln!(f, " > {} ({} - {} overflows)", self.instruction, self.source.signed_string(), self.temp.signed_string())?;
            }
            OverflowOther => {
                writeln!(f, "Encountered integer overflow occurred, which is prohibited.")?;
                writeln!(f, " > {} ({} with {} overflows)", self.instruction, self.source.signed_string(), self.temp.signed_string())?;
            }
            DivByZero => {
                writeln!(f, "Encountered division by zero, which is prohibited.")?;
                writeln!(f, " > {} ({} / {} overflows)", self.instruction, self.source.signed_string(), self.temp.signed_string())?;
            }
        }

        writeln!(f, "If you expected overflow behaviour, use unsigned instructions (addu, subu, multu, etc.)")
    }
}
