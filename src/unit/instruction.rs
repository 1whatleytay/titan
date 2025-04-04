use crate::cpu::decoder::Decoder;
use crate::unit::instruction::InstructionParameter::{Address, Immediate, Offset, Register};
use crate::unit::register::RegisterName;
use num::FromPrimitive;
use std::fmt::{Display, Formatter};

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Add {
        s: RegisterName,
        t: RegisterName,
        d: RegisterName,
    },
    Addu {
        s: RegisterName,
        t: RegisterName,
        d: RegisterName,
    },
    And {
        s: RegisterName,
        t: RegisterName,
        d: RegisterName,
    },
    Div {
        s: RegisterName,
        t: RegisterName,
    },
    Divu {
        s: RegisterName,
        t: RegisterName,
    },
    Mult {
        s: RegisterName,
        t: RegisterName,
    },
    Multu {
        s: RegisterName,
        t: RegisterName,
    },
    Nor {
        s: RegisterName,
        t: RegisterName,
        d: RegisterName,
    },
    Or {
        s: RegisterName,
        t: RegisterName,
        d: RegisterName,
    },
    Sll {
        t: RegisterName,
        d: RegisterName,
        sham: u8,
    },
    Sllv {
        s: RegisterName,
        t: RegisterName,
        d: RegisterName,
    },
    Sra {
        t: RegisterName,
        d: RegisterName,
        sham: u8,
    },
    Srav {
        s: RegisterName,
        t: RegisterName,
        d: RegisterName,
    },
    Srl {
        t: RegisterName,
        d: RegisterName,
        sham: u8,
    },
    Srlv {
        s: RegisterName,
        t: RegisterName,
        d: RegisterName,
    },
    Sub {
        s: RegisterName,
        t: RegisterName,
        d: RegisterName,
    },
    Subu {
        s: RegisterName,
        t: RegisterName,
        d: RegisterName,
    },
    Xor {
        s: RegisterName,
        t: RegisterName,
        d: RegisterName,
    },
    Slt {
        s: RegisterName,
        t: RegisterName,
        d: RegisterName,
    },
    Sltu {
        s: RegisterName,
        t: RegisterName,
        d: RegisterName,
    },
    Jr {
        s: RegisterName,
    },
    Jalr {
        s: RegisterName,
    },
    Madd {
        s: RegisterName,
        t: RegisterName,
    },
    Maddu {
        s: RegisterName,
        t: RegisterName,
    },
    Mul {
        s: RegisterName,
        t: RegisterName,
        d: RegisterName,
    },
    Msub {
        s: RegisterName,
        t: RegisterName,
    },
    Msubu {
        s: RegisterName,
        t: RegisterName,
    },
    Addi {
        s: RegisterName,
        t: RegisterName,
        imm: u16,
    },
    Addiu {
        s: RegisterName,
        t: RegisterName,
        imm: u16,
    },
    Andi {
        s: RegisterName,
        t: RegisterName,
        imm: u16,
    },
    Ori {
        s: RegisterName,
        t: RegisterName,
        imm: u16,
    },
    Xori {
        s: RegisterName,
        t: RegisterName,
        imm: u16,
    },
    Lui {
        s: RegisterName,
        imm: u16,
    },
    Lhi {
        t: RegisterName,
        imm: u16,
    },
    Llo {
        t: RegisterName,
        imm: u16,
    },
    Slti {
        s: RegisterName,
        t: RegisterName,
        imm: u16,
    },
    Sltiu {
        s: RegisterName,
        t: RegisterName,
        imm: u16,
    },
    Beq {
        s: RegisterName,
        t: RegisterName,
        address: u32,
    },
    Bne {
        s: RegisterName,
        t: RegisterName,
        address: u32,
    },
    Bgtz {
        s: RegisterName,
        address: u32,
    },
    Blez {
        s: RegisterName,
        address: u32,
    },
    Bltz {
        s: RegisterName,
        address: u32,
    },
    Bgez {
        s: RegisterName,
        address: u32,
    },
    Bltzal {
        s: RegisterName,
        address: u32,
    },
    Bgezal {
        s: RegisterName,
        address: u32,
    },
    J {
        address: u32,
    },
    Jal {
        address: u32,
    },
    Lb {
        s: RegisterName,
        t: RegisterName,
        imm: u16,
    },
    Lbu {
        s: RegisterName,
        t: RegisterName,
        imm: u16,
    },
    Lh {
        s: RegisterName,
        t: RegisterName,
        imm: u16,
    },
    Lhu {
        s: RegisterName,
        t: RegisterName,
        imm: u16,
    },
    Lw {
        s: RegisterName,
        t: RegisterName,
        imm: u16,
    },
    Sb {
        s: RegisterName,
        t: RegisterName,
        imm: u16,
    },
    Sh {
        s: RegisterName,
        t: RegisterName,
        imm: u16,
    },
    Sw {
        s: RegisterName,
        t: RegisterName,
        imm: u16,
    },
    Mfhi {
        d: RegisterName,
    },
    Mflo {
        d: RegisterName,
    },
    Mthi {
        s: RegisterName,
    },
    Mtlo {
        s: RegisterName,
    },
    Trap,
    Syscall,
}

pub fn sig(imm: u16) -> String {
    let value = imm as i16 as i64;

    if value.abs() < 10 {
        format!("{value}")
    } else {
        let sign = if value < 0 { "-" } else { "" };

        format!("{}0x{:x}", sign, value.abs())
    }
}

pub fn sig_u32(imm: u32) -> String {
    let value = imm as i32 as i64;

    if value.abs() < 10 {
        format!("{value}")
    } else {
        let sign = if value < 0 { "-" } else { "" };

        format!("{}0x{:x}", sign, value.abs())
    }
}

fn jump_dest(pc: u32, imm: u32) -> u32 {
    ((pc + 4) & 0xFC000000) | (imm << 2)
}

fn rel_dest(pc: u32, imm: u16) -> u32 {
    ((pc + 4) as i32 + ((imm as i16 as i32) << 2)) as u32
}

impl From<u8> for RegisterName {
    fn from(value: u8) -> Self {
        FromPrimitive::from_u8(value).unwrap()
    }
}

pub struct InstructionDecoder {
    address: u32,
}

impl InstructionDecoder {
    pub fn decode(address: u32, instruction: u32) -> Option<Instruction> {
        InstructionDecoder { address }.dispatch(instruction)
    }
}

impl Decoder<Instruction> for InstructionDecoder {
    fn add(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Add {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }

    fn addu(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Addu {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }

    fn and(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::And {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }

    fn div(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Div {
            s: s.into(),
            t: t.into(),
        }
    }

    fn divu(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Divu {
            s: s.into(),
            t: t.into(),
        }
    }

    fn mult(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Mult {
            s: s.into(),
            t: t.into(),
        }
    }

    fn multu(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Multu {
            s: s.into(),
            t: t.into(),
        }
    }

    fn nor(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Nor {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }

    fn or(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Or {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }

    fn sll(&mut self, t: u8, d: u8, sham: u8) -> Instruction {
        Instruction::Sll {
            t: t.into(),
            d: d.into(),
            sham,
        }
    }

    fn sllv(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Sllv {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }

    fn sra(&mut self, t: u8, d: u8, sham: u8) -> Instruction {
        Instruction::Sra {
            t: t.into(),
            d: d.into(),
            sham,
        }
    }

    fn srav(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Srav {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }

    fn srl(&mut self, t: u8, d: u8, sham: u8) -> Instruction {
        Instruction::Srl {
            t: t.into(),
            d: d.into(),
            sham,
        }
    }

    fn srlv(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Srlv {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }

    fn sub(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Sub {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }

    fn subu(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Subu {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }

    fn xor(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Xor {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }

    fn slt(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Slt {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }

    fn sltu(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Sltu {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }

    fn jr(&mut self, s: u8) -> Instruction {
        Instruction::Jr { s: s.into() }
    }

    fn jalr(&mut self, s: u8) -> Instruction {
        Instruction::Jalr { s: s.into() }
    }

    fn madd(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Madd {
            s: s.into(),
            t: t.into(),
        }
    }

    fn maddu(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Maddu {
            s: s.into(),
            t: t.into(),
        }
    }

    fn mul(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Mul {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }

    fn msub(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Msub {
            s: s.into(),
            t: t.into(),
        }
    }

    fn msubu(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Msubu {
            s: s.into(),
            t: t.into(),
        }
    }

    fn addi(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Addi {
            s: s.into(),
            t: t.into(),
            imm,
        }
    }

    fn addiu(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Addiu {
            s: s.into(),
            t: t.into(),
            imm,
        }
    }

    fn andi(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Andi {
            s: s.into(),
            t: t.into(),
            imm,
        }
    }

    fn ori(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Ori {
            s: s.into(),
            t: t.into(),
            imm,
        }
    }

    fn xori(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Xori {
            s: s.into(),
            t: t.into(),
            imm,
        }
    }

    fn lui(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Lui { s: s.into(), imm }
    }

    fn lhi(&mut self, t: u8, imm: u16) -> Instruction {
        Instruction::Lhi { t: t.into(), imm }
    }

    fn llo(&mut self, t: u8, imm: u16) -> Instruction {
        Instruction::Llo { t: t.into(), imm }
    }

    fn slti(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Slti {
            s: s.into(),
            t: t.into(),
            imm,
        }
    }

    fn sltiu(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Sltiu {
            s: s.into(),
            t: t.into(),
            imm,
        }
    }

    fn beq(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Beq {
            s: s.into(),
            t: t.into(),
            address: rel_dest(self.address, imm),
        }
    }

    fn bne(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Bne {
            s: s.into(),
            t: t.into(),
            address: rel_dest(self.address, imm),
        }
    }

    fn bgtz(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Bgtz {
            s: s.into(),
            address: rel_dest(self.address, imm),
        }
    }

    fn blez(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Blez {
            s: s.into(),
            address: rel_dest(self.address, imm),
        }
    }

    fn bltz(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Bltz {
            s: s.into(),
            address: rel_dest(self.address, imm),
        }
    }

    fn bgez(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Bgez {
            s: s.into(),
            address: rel_dest(self.address, imm),
        }
    }

    fn bltzal(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Bltzal {
            s: s.into(),
            address: rel_dest(self.address, imm),
        }
    }

    fn bgezal(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Bgezal {
            s: s.into(),
            address: rel_dest(self.address, imm),
        }
    }

    fn j(&mut self, imm: u32) -> Instruction {
        Instruction::J {
            address: jump_dest(self.address, imm),
        }
    }

    fn jal(&mut self, imm: u32) -> Instruction {
        Instruction::Jal {
            address: jump_dest(self.address, imm),
        }
    }

    fn lb(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Lb {
            s: s.into(),
            t: t.into(),
            imm,
        }
    }

    fn lbu(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Lbu {
            s: s.into(),
            t: t.into(),
            imm,
        }
    }

    fn lh(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Lh {
            s: s.into(),
            t: t.into(),
            imm,
        }
    }

    fn lhu(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Lhu {
            s: s.into(),
            t: t.into(),
            imm,
        }
    }

    fn lw(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Lw {
            s: s.into(),
            t: t.into(),
            imm,
        }
    }

    fn sb(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Sb {
            s: s.into(),
            t: t.into(),
            imm,
        }
    }

    fn sh(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Sh {
            s: s.into(),
            t: t.into(),
            imm,
        }
    }

    fn sw(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Sw {
            s: s.into(),
            t: t.into(),
            imm,
        }
    }

    fn mfhi(&mut self, d: u8) -> Instruction {
        Instruction::Mfhi { d: d.into() }
    }

    fn mflo(&mut self, d: u8) -> Instruction {
        Instruction::Mflo { d: d.into() }
    }

    fn mthi(&mut self, s: u8) -> Instruction {
        Instruction::Mthi { s: s.into() }
    }

    fn mtlo(&mut self, s: u8) -> Instruction {
        Instruction::Mtlo { s: s.into() }
    }

    fn trap(&mut self) -> Instruction {
        Instruction::Trap
    }

    fn syscall(&mut self) -> Instruction {
        Instruction::Syscall
    }
}

pub enum InstructionParameter {
    Register(RegisterName),
    Immediate(u16),
    Address(u32),
    Offset(u16, RegisterName),
}

impl From<RegisterName> for InstructionParameter {
    fn from(value: RegisterName) -> Self {
        Register(value)
    }
}

impl Instruction {
    pub fn name(&self) -> &'static str {
        match self {
            Instruction::Add { .. } => "add",
            Instruction::Addu { .. } => "addu",
            Instruction::And { .. } => "and",
            Instruction::Div { .. } => "div",
            Instruction::Divu { .. } => "divu",
            Instruction::Mult { .. } => "mult",
            Instruction::Multu { .. } => "multu",
            Instruction::Nor { .. } => "nor",
            Instruction::Or { .. } => "or",
            Instruction::Sll { .. } => "sll",
            Instruction::Sllv { .. } => "sllv",
            Instruction::Sra { .. } => "sra",
            Instruction::Srav { .. } => "srav",
            Instruction::Srl { .. } => "srl",
            Instruction::Srlv { .. } => "srlv",
            Instruction::Sub { .. } => "sub",
            Instruction::Subu { .. } => "subu",
            Instruction::Xor { .. } => "xor",
            Instruction::Slt { .. } => "slt",
            Instruction::Sltu { .. } => "sltu",
            Instruction::Jr { .. } => "jr",
            Instruction::Jalr { .. } => "jalr",
            Instruction::Madd { .. } => "madd",
            Instruction::Maddu { .. } => "maddu",
            Instruction::Mul { .. } => "mul",
            Instruction::Msub { .. } => "msub",
            Instruction::Msubu { .. } => "msubu",
            Instruction::Addi { .. } => "addi",
            Instruction::Addiu { .. } => "addiu",
            Instruction::Andi { .. } => "andi",
            Instruction::Ori { .. } => "ori",
            Instruction::Xori { .. } => "xori",
            Instruction::Lui { .. } => "lui",
            Instruction::Lhi { .. } => "lhi",
            Instruction::Llo { .. } => "llo",
            Instruction::Slti { .. } => "slti",
            Instruction::Sltiu { .. } => "sltiu",
            Instruction::Beq { .. } => "beq",
            Instruction::Bne { .. } => "bne",
            Instruction::Bgtz { .. } => "bgtz",
            Instruction::Blez { .. } => "blez",
            Instruction::Bltz { .. } => "bltz",
            Instruction::Bgez { .. } => "bgez",
            Instruction::Bltzal { .. } => "bltzal",
            Instruction::Bgezal { .. } => "bgezal",
            Instruction::J { .. } => "j",
            Instruction::Jal { .. } => "jal",
            Instruction::Lb { .. } => "lb",
            Instruction::Lbu { .. } => "lbu",
            Instruction::Lh { .. } => "lh",
            Instruction::Lhu { .. } => "lhu",
            Instruction::Lw { .. } => "lw",
            Instruction::Sb { .. } => "sb",
            Instruction::Sh { .. } => "sh",
            Instruction::Sw { .. } => "sw",
            Instruction::Mfhi { .. } => "mfhi",
            Instruction::Mflo { .. } => "mflo",
            Instruction::Mthi { .. } => "mthi",
            Instruction::Mtlo { .. } => "mtlo",
            Instruction::Trap { .. } => "trap",
            Instruction::Syscall { .. } => "syscall",
        }
    }

    pub fn parameters(self) -> Vec<InstructionParameter> {
        match self {
            Instruction::Add { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::Addu { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::And { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::Div { s, t } => vec![s.into(), t.into()],
            Instruction::Divu { s, t } => vec![s.into(), t.into()],
            Instruction::Mult { s, t } => vec![s.into(), t.into()],
            Instruction::Multu { s, t } => vec![s.into(), t.into()],
            Instruction::Nor { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::Or { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::Sll { t, d, sham } => vec![d.into(), t.into(), Immediate(sham as u16)],
            Instruction::Sllv { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::Sra { t, d, sham } => vec![d.into(), t.into(), Immediate(sham as u16)],
            Instruction::Srav { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::Srl { t, d, sham } => vec![d.into(), t.into(), Immediate(sham as u16)],
            Instruction::Srlv { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::Sub { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::Subu { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::Xor { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::Slt { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::Sltu { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::Jr { s } => vec![s.into()],
            Instruction::Jalr { s } => vec![s.into()],
            Instruction::Madd { s, t } => vec![s.into(), t.into()],
            Instruction::Maddu { s, t } => vec![s.into(), t.into()],
            Instruction::Mul { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::Msub { s, t } => vec![s.into(), t.into()],
            Instruction::Msubu { s, t } => vec![s.into(), t.into()],
            Instruction::Addi { s, t, imm } => vec![t.into(), s.into(), Immediate(imm)],
            Instruction::Addiu { s, t, imm } => vec![t.into(), s.into(), Immediate(imm)],
            Instruction::Andi { s, t, imm } => vec![t.into(), s.into(), Immediate(imm)],
            Instruction::Ori { s, t, imm } => vec![t.into(), s.into(), Immediate(imm)],
            Instruction::Xori { s, t, imm } => vec![t.into(), s.into(), Immediate(imm)],
            Instruction::Lui { s, imm } => vec![s.into(), Immediate(imm)],
            Instruction::Lhi { t, imm } => vec![t.into(), Immediate(imm)],
            Instruction::Llo { t, imm } => vec![t.into(), Immediate(imm)],
            Instruction::Slti { s, t, imm } => vec![t.into(), s.into(), Immediate(imm)],
            Instruction::Sltiu { s, t, imm } => vec![t.into(), s.into(), Immediate(imm)],
            Instruction::Beq { s, t, address } => vec![s.into(), t.into(), Address(address)],
            Instruction::Bne { s, t, address } => vec![s.into(), t.into(), Address(address)],
            Instruction::Bgtz { s, address } => vec![s.into(), Address(address)],
            Instruction::Blez { s, address } => vec![s.into(), Address(address)],
            Instruction::Bltz { s, address } => vec![s.into(), Address(address)],
            Instruction::Bgez { s, address } => vec![s.into(), Address(address)],
            Instruction::Bltzal { s, address } => vec![s.into(), Address(address)],
            Instruction::Bgezal { s, address } => vec![s.into(), Address(address)],
            Instruction::J { address } => vec![Address(address)],
            Instruction::Jal { address } => vec![Address(address)],
            Instruction::Lb { s, t, imm } => vec![t.into(), Offset(imm, s)],
            Instruction::Lbu { s, t, imm } => vec![t.into(), Offset(imm, s)],
            Instruction::Lh { s, t, imm } => vec![t.into(), Offset(imm, s)],
            Instruction::Lhu { s, t, imm } => vec![t.into(), Offset(imm, s)],
            Instruction::Lw { s, t, imm } => vec![t.into(), Offset(imm, s)],
            Instruction::Sb { s, t, imm } => vec![t.into(), Offset(imm, s)],
            Instruction::Sh { s, t, imm } => vec![t.into(), Offset(imm, s)],
            Instruction::Sw { s, t, imm } => vec![t.into(), Offset(imm, s)],
            Instruction::Mfhi { d } => vec![d.into()],
            Instruction::Mflo { d } => vec![d.into()],
            Instruction::Mthi { s } => vec![s.into()],
            Instruction::Mtlo { s } => vec![s.into()],
            Instruction::Trap => vec![],
            Instruction::Syscall => vec![],
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Add { s, t, d } => write!(f, "add {}, {}, {}", d, s, t),
            Instruction::Addu { s, t, d } => write!(f, "addu {}, {}, {}", d, s, t),
            Instruction::And { s, t, d } => write!(f, "and {}, {}, {}", d, s, t),
            Instruction::Div { s, t } => write!(f, "div {}, {}", s, t),
            Instruction::Divu { s, t } => write!(f, "divu {}, {}", s, t),
            Instruction::Mult { s, t } => write!(f, "mult {}, {}", s, t),
            Instruction::Multu { s, t } => write!(f, "multu {}, {}", s, t),
            Instruction::Nor { s, t, d } => write!(f, "nor {}, {}, {}", d, s, t),
            Instruction::Or { s, t, d } => write!(f, "or {}, {}, {}", d, s, t),
            Instruction::Sll { t, d, sham } => write!(f, "sll {}, {}, {}", d, t, sham),
            Instruction::Sllv { s, t, d } => write!(f, "sllv {}, {}, {}", d, t, s),
            Instruction::Sra { t, d, sham } => write!(f, "sra {}, {}, {}", d, t, sham),
            Instruction::Srav { s, t, d } => write!(f, "srav {}, {}, {}", d, t, s),
            Instruction::Srl { t, d, sham } => write!(f, "srl {}, {}, {}", d, t, sham),
            Instruction::Srlv { s, t, d } => write!(f, "srlv {}, {}, {}", d, t, s),
            Instruction::Sub { s, t, d } => write!(f, "sub {}, {}, {}", s, t, d),
            Instruction::Subu { s, t, d } => write!(f, "subu {}, {}, {}", s, t, d),
            Instruction::Xor { s, t, d } => write!(f, "xor {}, {}, {}", s, t, d),
            Instruction::Slt { s, t, d } => write!(f, "slt {}, {}, {}", s, t, d),
            Instruction::Sltu { s, t, d } => write!(f, "sltu {}, {}, {}", s, t, d),
            Instruction::Jr { s } => write!(f, "jr {}", s),
            Instruction::Jalr { s } => write!(f, "jalr {}", s),
            Instruction::Madd { s, t } => write!(f, "madd {}, {}", s, t),
            Instruction::Maddu { s, t } => write!(f, "maddu {}, {}", s, t),
            Instruction::Mul { s, t, d } => write!(f, "mul {}, {}, {}", d, s, t),
            Instruction::Msub { s, t } => write!(f, "msub {}, {}", s, t),
            Instruction::Msubu { s, t } => write!(f, "msubu {}, {}", s, t),
            Instruction::Addi { s, t, imm } => write!(f, "addi {}, {}, {}", t, s, sig(*imm)),
            Instruction::Addiu { s, t, imm } => write!(f, "addiu {}, {}, {}", t, s, sig(*imm)),
            Instruction::Andi { s, t, imm } => write!(f, "andi {}, {}, {}", t, s, sig(*imm)),
            Instruction::Ori { s, t, imm } => write!(f, "ori {}, {}, {}", t, s, sig(*imm)),
            Instruction::Xori { s, t, imm } => write!(f, "xori {}, {}, {}", t, s, sig(*imm)),
            Instruction::Lui { s, imm } => write!(f, "lui {}, {}", s, sig(*imm)),
            Instruction::Lhi { t, imm } => write!(f, "lhi {}, {}", t, sig(*imm)),
            Instruction::Llo { t, imm } => write!(f, "llo {}, {}", t, sig(*imm)),
            Instruction::Slti { s, t, imm } => write!(f, "slti {}, {}, {}", t, s, sig(*imm)),
            Instruction::Sltiu { s, t, imm } => write!(f, "sltiu {}, {}, {}", t, s, sig(*imm)),
            Instruction::Beq { s, t, address } => write!(f, "beq {}, {}, 0x{:x}", s, t, address),
            Instruction::Bne { s, t, address } => write!(f, "bne {}, {}, 0x{:x}", s, t, address),
            Instruction::Bgtz { s, address } => write!(f, "bgtz {}, 0x{:x}", s, address),
            Instruction::Blez { s, address } => write!(f, "blez {}, 0x{:x}", s, address),
            Instruction::Bltz { s, address } => write!(f, "bltz {}, 0x{:x}", s, address),
            Instruction::Bgez { s, address } => write!(f, "bgez {}, 0x{:x}", s, address),
            Instruction::Bltzal { s, address } => write!(f, "bltzal {}, 0x{:x}", s, address),
            Instruction::Bgezal { s, address } => write!(f, "bgezal {}, 0x{:x}", s, address),
            Instruction::J { address } => write!(f, "j 0x{:x}", address),
            Instruction::Jal { address } => write!(f, "jal 0x{:x}", address),
            Instruction::Lb { s, t, imm } => write!(f, "lb {}, {}({})", t, sig(*imm), s),
            Instruction::Lbu { s, t, imm } => write!(f, "lbu {}, {}({})", t, sig(*imm), s),
            Instruction::Lh { s, t, imm } => write!(f, "lh {}, {}({})", t, sig(*imm), s),
            Instruction::Lhu { s, t, imm } => write!(f, "lhu {}, {}({})", t, sig(*imm), s),
            Instruction::Lw { s, t, imm } => write!(f, "lw {}, {}({})", t, sig(*imm), s),
            Instruction::Sb { s, t, imm } => write!(f, "sb {}, {}({})", t, sig(*imm), s),
            Instruction::Sh { s, t, imm } => write!(f, "sh {}, {}({})", t, sig(*imm), s),
            Instruction::Sw { s, t, imm } => write!(f, "sw {}, {}({})", t, sig(*imm), s),
            Instruction::Mfhi { d } => write!(f, "mfhi {}", d),
            Instruction::Mflo { d } => write!(f, "mflo {}", d),
            Instruction::Mthi { s } => write!(f, "mthi {}", s),
            Instruction::Mtlo { s } => write!(f, "mtlo {}", s),
            Instruction::Trap => write!(f, "trap"),
            Instruction::Syscall => write!(f, "syscall"),
        }
    }
}
