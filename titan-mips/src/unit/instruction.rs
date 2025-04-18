use crate::assembler::registers::{FPRegisterSlot, RegisterSlot};
use crate::cpu::decoder::Decoder;
use crate::unit::instruction::InstructionParameter::{
    Address, FPRegister, Immediate, Offset, Register,
};
use num::FromPrimitive;
use std::fmt::{Display, Formatter};

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Add {
        s: RegisterSlot,
        t: RegisterSlot,
        d: RegisterSlot,
    },
    Addu {
        s: RegisterSlot,
        t: RegisterSlot,
        d: RegisterSlot,
    },
    And {
        s: RegisterSlot,
        t: RegisterSlot,
        d: RegisterSlot,
    },
    Div {
        s: RegisterSlot,
        t: RegisterSlot,
    },
    Divu {
        s: RegisterSlot,
        t: RegisterSlot,
    },
    Mult {
        s: RegisterSlot,
        t: RegisterSlot,
    },
    Multu {
        s: RegisterSlot,
        t: RegisterSlot,
    },
    Nor {
        s: RegisterSlot,
        t: RegisterSlot,
        d: RegisterSlot,
    },
    Or {
        s: RegisterSlot,
        t: RegisterSlot,
        d: RegisterSlot,
    },
    Sll {
        t: RegisterSlot,
        d: RegisterSlot,
        sham: u8,
    },
    Sllv {
        s: RegisterSlot,
        t: RegisterSlot,
        d: RegisterSlot,
    },
    Sra {
        t: RegisterSlot,
        d: RegisterSlot,
        sham: u8,
    },
    Srav {
        s: RegisterSlot,
        t: RegisterSlot,
        d: RegisterSlot,
    },
    Srl {
        t: RegisterSlot,
        d: RegisterSlot,
        sham: u8,
    },
    Srlv {
        s: RegisterSlot,
        t: RegisterSlot,
        d: RegisterSlot,
    },
    Sub {
        s: RegisterSlot,
        t: RegisterSlot,
        d: RegisterSlot,
    },
    Subu {
        s: RegisterSlot,
        t: RegisterSlot,
        d: RegisterSlot,
    },
    Xor {
        s: RegisterSlot,
        t: RegisterSlot,
        d: RegisterSlot,
    },
    Slt {
        s: RegisterSlot,
        t: RegisterSlot,
        d: RegisterSlot,
    },
    Sltu {
        s: RegisterSlot,
        t: RegisterSlot,
        d: RegisterSlot,
    },
    Jr {
        s: RegisterSlot,
    },
    Jalr {
        s: RegisterSlot,
    },
    Madd {
        s: RegisterSlot,
        t: RegisterSlot,
    },
    Maddu {
        s: RegisterSlot,
        t: RegisterSlot,
    },
    Mul {
        s: RegisterSlot,
        t: RegisterSlot,
        d: RegisterSlot,
    },
    Msub {
        s: RegisterSlot,
        t: RegisterSlot,
    },
    Msubu {
        s: RegisterSlot,
        t: RegisterSlot,
    },
    Addi {
        s: RegisterSlot,
        t: RegisterSlot,
        imm: u16,
    },
    Addiu {
        s: RegisterSlot,
        t: RegisterSlot,
        imm: u16,
    },
    Andi {
        s: RegisterSlot,
        t: RegisterSlot,
        imm: u16,
    },
    Ori {
        s: RegisterSlot,
        t: RegisterSlot,
        imm: u16,
    },
    Xori {
        s: RegisterSlot,
        t: RegisterSlot,
        imm: u16,
    },
    Lui {
        s: RegisterSlot,
        imm: u16,
    },
    Lhi {
        t: RegisterSlot,
        imm: u16,
    },
    Llo {
        t: RegisterSlot,
        imm: u16,
    },
    Slti {
        s: RegisterSlot,
        t: RegisterSlot,
        imm: u16,
    },
    Sltiu {
        s: RegisterSlot,
        t: RegisterSlot,
        imm: u16,
    },
    Beq {
        s: RegisterSlot,
        t: RegisterSlot,
        address: u32,
    },
    Bne {
        s: RegisterSlot,
        t: RegisterSlot,
        address: u32,
    },
    Bgtz {
        s: RegisterSlot,
        address: u32,
    },
    Blez {
        s: RegisterSlot,
        address: u32,
    },
    Bltz {
        s: RegisterSlot,
        address: u32,
    },
    Bgez {
        s: RegisterSlot,
        address: u32,
    },
    Bltzal {
        s: RegisterSlot,
        address: u32,
    },
    Bgezal {
        s: RegisterSlot,
        address: u32,
    },
    J {
        address: u32,
    },
    Jal {
        address: u32,
    },
    Lb {
        s: RegisterSlot,
        t: RegisterSlot,
        imm: u16,
    },
    Lbu {
        s: RegisterSlot,
        t: RegisterSlot,
        imm: u16,
    },
    Lh {
        s: RegisterSlot,
        t: RegisterSlot,
        imm: u16,
    },
    Lhu {
        s: RegisterSlot,
        t: RegisterSlot,
        imm: u16,
    },
    Lw {
        s: RegisterSlot,
        t: RegisterSlot,
        imm: u16,
    },
    Sb {
        s: RegisterSlot,
        t: RegisterSlot,
        imm: u16,
    },
    Sh {
        s: RegisterSlot,
        t: RegisterSlot,
        imm: u16,
    },
    Sw {
        s: RegisterSlot,
        t: RegisterSlot,
        imm: u16,
    },
    Mfhi {
        d: RegisterSlot,
    },
    Mflo {
        d: RegisterSlot,
    },
    Mthi {
        s: RegisterSlot,
    },
    Mtlo {
        s: RegisterSlot,
    },
    Trap,
    Syscall,
    AddS {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    SubS {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    MulS {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    DivS {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    SqrtS {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    AbsS {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    NegS {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    FloorWS {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    CeilWS {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    RoundWS {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    TruncWS {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    AddD {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    SubD {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    MulD {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    DivD {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    SqrtD {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    AbsD {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    NegD {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    FloorWD {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    CeilWD {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    RoundWD {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    TruncWD {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    CEqS {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        cc: u8,
    },
    CLeS {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        cc: u8,
    },
    CLtS {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        cc: u8,
    },
    CEqD {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        cc: u8,
    },
    CLeD {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        cc: u8,
    },
    CLtD {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        cc: u8,
    },
    BC1T {
        cc: u8,
        offset: u16,
    },
    BC1F {
        cc: u8,
        offset: u16,
    },
    MovS {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    MovFS {
        cc: u8,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    MovTS {
        cc: u8,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    MovNS {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    MovZS {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    MovD {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    MovFD {
        cc: u8,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    MovTD {
        cc: u8,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    MovND {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    MovZD {
        t: FPRegisterSlot,
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    MovF {
        s: FPRegisterSlot,
        cc: u8,
        d: FPRegisterSlot,
    },
    MovT {
        s: FPRegisterSlot,
        cc: u8,
        d: FPRegisterSlot,
    },
    MovN {
        s: FPRegisterSlot,
        t: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    MovZ {
        s: FPRegisterSlot,
        t: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    CvtSW {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    CvtWS {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    CvtDS {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    CvtSD {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    CvtDW {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    CvtWD {
        s: FPRegisterSlot,
        d: FPRegisterSlot,
    },
    Mtc1 {
        t: FPRegisterSlot,
        s: RegisterSlot,
    },
    Mfc1 {
        t: RegisterSlot,
        s: FPRegisterSlot,
    },
    Lwc1 {
        base: RegisterSlot,
        t: FPRegisterSlot,
        offset: u16,
    },
    Swc1 {
        base: RegisterSlot,
        t: FPRegisterSlot,
        offset: u16,
    },
    Ldc1 {
        base: RegisterSlot,
        t: FPRegisterSlot,
        offset: u16,
    },
    Sdc1 {
        base: RegisterSlot,
        t: FPRegisterSlot,
        offset: u16,
    },
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

impl From<u8> for RegisterSlot {
    fn from(value: u8) -> Self {
        FromPrimitive::from_u8(value).unwrap()
    }
}
impl From<u8> for FPRegisterSlot {
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
    fn add_s(&mut self, t: u8, s: u8, d: u8) -> Instruction {
        Instruction::AddS {
            t: t.into(),
            s: s.into(),
            d: d.into(),
        }
    }
    fn sub_s(&mut self, t: u8, s: u8, d: u8) -> Instruction {
        Instruction::SubS {
            t: t.into(),
            s: s.into(),
            d: d.into(),
        }
    }
    fn mul_s(&mut self, t: u8, s: u8, d: u8) -> Instruction {
        Instruction::MulS {
            t: t.into(),
            s: s.into(),
            d: d.into(),
        }
    }
    fn div_s(&mut self, t: u8, s: u8, d: u8) -> Instruction {
        Instruction::DivS {
            t: t.into(),
            s: s.into(),
            d: d.into(),
        }
    }
    fn sqrt_s(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::SqrtS {
            s: s.into(),
            d: d.into(),
        }
    }
    fn abs_s(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::AbsS {
            s: s.into(),
            d: d.into(),
        }
    }
    fn neg_s(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::NegS {
            s: s.into(),
            d: d.into(),
        }
    }
    fn floor_w_s(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::FloorWS {
            s: s.into(),
            d: d.into(),
        }
    }
    fn ceil_w_s(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::CeilWS {
            s: s.into(),
            d: d.into(),
        }
    }
    fn round_w_s(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::RoundWS {
            s: s.into(),
            d: d.into(),
        }
    }
    fn trunc_w_s(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::TruncWS {
            s: s.into(),
            d: d.into(),
        }
    }
    fn add_d(&mut self, t: u8, s: u8, d: u8) -> Instruction {
        Instruction::AddD {
            t: t.into(),
            s: s.into(),
            d: d.into(),
        }
    }
    fn sub_d(&mut self, t: u8, s: u8, d: u8) -> Instruction {
        Instruction::SubD {
            t: t.into(),
            s: s.into(),
            d: d.into(),
        }
    }
    fn mul_d(&mut self, t: u8, s: u8, d: u8) -> Instruction {
        Instruction::MulD {
            t: t.into(),
            s: s.into(),
            d: d.into(),
        }
    }
    fn div_d(&mut self, t: u8, s: u8, d: u8) -> Instruction {
        Instruction::DivD {
            t: t.into(),
            s: s.into(),
            d: d.into(),
        }
    }
    fn sqrt_d(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::SqrtD {
            s: s.into(),
            d: d.into(),
        }
    }
    fn abs_d(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::AbsD {
            s: s.into(),
            d: d.into(),
        }
    }
    fn neg_d(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::NegD {
            s: s.into(),
            d: d.into(),
        }
    }
    fn floor_w_d(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::FloorWD {
            s: s.into(),
            d: d.into(),
        }
    }
    fn ceil_w_d(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::CeilWD {
            s: s.into(),
            d: d.into(),
        }
    }
    fn round_w_d(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::RoundWD {
            s: s.into(),
            d: d.into(),
        }
    }
    fn trunc_w_d(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::TruncWD {
            s: s.into(),
            d: d.into(),
        }
    }
    fn c_eq_s(&mut self, t: u8, s: u8, cc: u8) -> Instruction {
        Instruction::CEqS {
            t: t.into(),
            s: s.into(),
            cc,
        }
    }
    fn c_le_s(&mut self, t: u8, s: u8, cc: u8) -> Instruction {
        Instruction::CLeS {
            t: t.into(),
            s: s.into(),
            cc,
        }
    }
    fn c_lt_s(&mut self, t: u8, s: u8, cc: u8) -> Instruction {
        Instruction::CLtS {
            t: t.into(),
            s: s.into(),
            cc,
        }
    }
    fn c_eq_d(&mut self, t: u8, s: u8, cc: u8) -> Instruction {
        Instruction::CEqD {
            t: t.into(),
            s: s.into(),
            cc,
        }
    }
    fn c_le_d(&mut self, t: u8, s: u8, cc: u8) -> Instruction {
        Instruction::CLeD {
            t: t.into(),
            s: s.into(),
            cc,
        }
    }
    fn c_lt_d(&mut self, t: u8, s: u8, cc: u8) -> Instruction {
        Instruction::CLtD {
            t: t.into(),
            s: s.into(),
            cc,
        }
    }
    fn bc1t(&mut self, cc: u8, address: u16) -> Instruction {
        Instruction::BC1T {
            cc,
            offset: address,
        }
    }
    fn bc1f(&mut self, cc: u8, address: u16) -> Instruction {
        Instruction::BC1F {
            cc,
            offset: address,
        }
    }
    fn mov_s(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::MovS {
            s: s.into(),
            d: d.into(),
        }
    }
    fn movf_s(&mut self, cc: u8, s: u8, d: u8) -> Instruction {
        Instruction::MovFS {
            cc,
            s: s.into(),
            d: d.into(),
        }
    }
    fn movt_s(&mut self, cc: u8, s: u8, d: u8) -> Instruction {
        Instruction::MovTS {
            cc,
            s: s.into(),
            d: d.into(),
        }
    }
    fn movn_s(&mut self, t: u8, s: u8, d: u8) -> Instruction {
        Instruction::MovNS {
            t: t.into(),
            s: s.into(),
            d: d.into(),
        }
    }
    fn movz_s(&mut self, t: u8, s: u8, d: u8) -> Instruction {
        Instruction::MovZS {
            t: t.into(),
            s: s.into(),
            d: d.into(),
        }
    }
    fn mov_d(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::MovD {
            s: s.into(),
            d: d.into(),
        }
    }
    fn movf_d(&mut self, cc: u8, s: u8, d: u8) -> Instruction {
        Instruction::MovFD {
            cc,
            s: s.into(),
            d: d.into(),
        }
    }
    fn movt_d(&mut self, cc: u8, s: u8, d: u8) -> Instruction {
        Instruction::MovTD {
            cc,
            s: s.into(),
            d: d.into(),
        }
    }
    fn movn_d(&mut self, t: u8, s: u8, d: u8) -> Instruction {
        Instruction::MovND {
            t: t.into(),
            s: s.into(),
            d: d.into(),
        }
    }
    fn movz_d(&mut self, t: u8, s: u8, d: u8) -> Instruction {
        Instruction::MovZD {
            t: t.into(),
            s: s.into(),
            d: d.into(),
        }
    }
    fn movf(&mut self, s: u8, cc: u8, d: u8) -> Instruction {
        Instruction::MovF {
            s: s.into(),
            cc,
            d: d.into(),
        }
    }
    fn movt(&mut self, s: u8, cc: u8, d: u8) -> Instruction {
        Instruction::MovT {
            s: s.into(),
            cc,
            d: d.into(),
        }
    }
    fn movn(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::MovN {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }
    fn movz(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::MovZ {
            s: s.into(),
            t: t.into(),
            d: d.into(),
        }
    }
    fn cvt_s_w(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::CvtSW {
            s: s.into(),
            d: d.into(),
        }
    }
    fn cvt_w_s(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::CvtWS {
            s: s.into(),
            d: d.into(),
        }
    }
    fn cvt_s_d(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::CvtSD {
            s: s.into(),
            d: d.into(),
        }
    }
    fn cvt_d_s(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::CvtDS {
            s: s.into(),
            d: d.into(),
        }
    }
    fn cvt_d_w(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::CvtDW {
            s: s.into(),
            d: d.into(),
        }
    }
    fn cvt_w_d(&mut self, s: u8, d: u8) -> Instruction {
        Instruction::CvtWD {
            s: s.into(),
            d: d.into(),
        }
    }
    fn mtc1(&mut self, t: u8, s: u8) -> Instruction {
        Instruction::Mtc1 {
            t: t.into(),
            s: s.into(),
        }
    }
    fn mfc1(&mut self, t: u8, s: u8) -> Instruction {
        Instruction::Mfc1 {
            t: t.into(),
            s: s.into(),
        }
    }
    fn ldc1(&mut self, base: u8, t: u8, offset: u16) -> Instruction {
        Instruction::Ldc1 {
            base: base.into(),
            t: t.into(),
            offset,
        }
    }
    fn sdc1(&mut self, base: u8, t: u8, offset: u16) -> Instruction {
        Instruction::Sdc1 {
            base: base.into(),
            t: t.into(),
            offset,
        }
    }

    fn lwc1(&mut self, base: u8, t: u8, offset: u16) -> Instruction {
        Instruction::Lwc1 {
            base: base.into(),
            t: t.into(),
            offset,
        }
    }

    fn swc1(&mut self, base: u8, t: u8, offset: u16) -> Instruction {
        Instruction::Swc1 {
            base: base.into(),
            t: t.into(),
            offset,
        }
    }
}

pub enum InstructionParameter {
    Register(RegisterSlot),
    FPRegister(FPRegisterSlot),
    Immediate(u16),
    Address(u32),
    Offset(u16, RegisterSlot),
}

impl From<RegisterSlot> for InstructionParameter {
    fn from(value: RegisterSlot) -> Self {
        Register(value)
    }
}
impl From<FPRegisterSlot> for InstructionParameter {
    fn from(value: FPRegisterSlot) -> Self {
        FPRegister(value)
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
            Instruction::AddS { .. } => "add.s",
            Instruction::SubS { .. } => "sub.s",
            Instruction::MulS { .. } => "mul.s",
            Instruction::DivS { .. } => "div.s",
            Instruction::SqrtS { .. } => "sqrt.s",
            Instruction::AbsS { .. } => "abs.s",
            Instruction::NegS { .. } => "neg.s",
            Instruction::FloorWS { .. } => "floor.w.s",
            Instruction::CeilWS { .. } => "ceil.w.s",
            Instruction::RoundWS { .. } => "round.w.s",
            Instruction::TruncWS { .. } => "trunc.w.s",
            Instruction::AddD { .. } => "add.d",
            Instruction::SubD { .. } => "sub.d",
            Instruction::MulD { .. } => "mul.d",
            Instruction::DivD { .. } => "div.d",
            Instruction::SqrtD { .. } => "sqrt.d",
            Instruction::AbsD { .. } => "abs.d",
            Instruction::NegD { .. } => "neg.d",
            Instruction::FloorWD { .. } => "floor.w.d",
            Instruction::CeilWD { .. } => "ceil.w.d",
            Instruction::RoundWD { .. } => "round.w.d",
            Instruction::TruncWD { .. } => "trunc.w.d",
            Instruction::CEqS { .. } => "c.eq.s",
            Instruction::CLeS { .. } => "c.le.s",
            Instruction::CLtS { .. } => "c.lt.s",
            Instruction::CEqD { .. } => "c.eq.d",
            Instruction::CLeD { .. } => "c.le.d",
            Instruction::CLtD { .. } => "c.lt.d",
            Instruction::BC1T { .. } => "bc1t",
            Instruction::BC1F { .. } => "bc1f",
            Instruction::MovS { .. } => "mov.s",
            Instruction::MovFS { .. } => "movf.s",
            Instruction::MovTS { .. } => "movt.s",
            Instruction::MovNS { .. } => "movn.s",
            Instruction::MovZS { .. } => "movz.s",
            Instruction::MovD { .. } => "mov.d",
            Instruction::MovFD { .. } => "movf.d",
            Instruction::MovTD { .. } => "movt.d",
            Instruction::MovND { .. } => "movn.d",
            Instruction::MovZD { .. } => "movz.d",
            Instruction::MovF { .. } => "movf",
            Instruction::MovT { .. } => "movt",
            Instruction::MovN { .. } => "movn",
            Instruction::MovZ { .. } => "movz",
            Instruction::CvtSW { .. } => "cvt.s.w",
            Instruction::CvtWS { .. } => "cvt.w.s",
            Instruction::CvtDS { .. } => "cvt.d.s",
            Instruction::CvtSD { .. } => "cvt.s.d",
            Instruction::CvtDW { .. } => "cvt.d.w",
            Instruction::CvtWD { .. } => "cvt.w.d",
            Instruction::Mtc1 { .. } => "mtc1",
            Instruction::Mfc1 { .. } => "mfc1",
            Instruction::Lwc1 { .. } => "lwc1",
            Instruction::Swc1 { .. } => "swc1",
            Instruction::Ldc1 { .. } => "ldc1",
            Instruction::Sdc1 { .. } => "sdc1",
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
            Instruction::AddS { t, s, d } => vec![d.into(), s.into(), t.into()],
            Instruction::SubS { t, s, d } => vec![d.into(), s.into(), t.into()],
            Instruction::MulS { t, s, d } => vec![d.into(), s.into(), t.into()],
            Instruction::DivS { t, s, d } => vec![d.into(), s.into(), t.into()],
            Instruction::SqrtS { s, d } => vec![d.into(), s.into()],
            Instruction::AbsS { s, d } => vec![d.into(), s.into()],
            Instruction::NegS { s, d } => vec![d.into(), s.into()],
            Instruction::FloorWS { s, d } => vec![d.into(), s.into()],
            Instruction::CeilWS { s, d } => vec![d.into(), s.into()],
            Instruction::RoundWS { s, d } => vec![d.into(), s.into()],
            Instruction::TruncWS { s, d } => vec![d.into(), s.into()],
            Instruction::AddD { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::SubD { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::MulD { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::DivD { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::SqrtD { s, d } => vec![d.into(), s.into()],
            Instruction::AbsD { s, d } => vec![d.into(), s.into()],
            Instruction::NegD { s, d } => vec![d.into(), s.into()],
            Instruction::FloorWD { s, d } => vec![d.into(), s.into()],
            Instruction::CeilWD { s, d } => vec![d.into(), s.into()],
            Instruction::RoundWD { s, d } => vec![d.into(), s.into()],
            Instruction::TruncWD { s, d } => vec![d.into(), s.into()],
            Instruction::CEqS { t, s, cc } => vec![Immediate(cc.into()), s.into(), t.into()],
            Instruction::CLeS { t, s, cc } => vec![Immediate(cc.into()), s.into(), t.into()],
            Instruction::CLtS { t, s, cc } => vec![Immediate(cc.into()), s.into(), t.into()],
            Instruction::CEqD { t, s, cc } => vec![Immediate(cc.into()), s.into(), t.into()],
            Instruction::CLeD { t, s, cc } => vec![Immediate(cc.into()), s.into(), t.into()],
            Instruction::CLtD { t, s, cc } => vec![Immediate(cc.into()), s.into(), t.into()],
            Instruction::BC1T { cc, offset } => vec![Immediate(cc.into()), Address(offset.into())],
            Instruction::BC1F { cc, offset } => vec![Immediate(cc.into()), Address(offset.into())],
            Instruction::MovS { s, d } => vec![d.into(), s.into()],
            Instruction::MovFS { cc, s, d } => vec![d.into(), s.into(), Immediate(cc.into())],
            Instruction::MovTS { cc, s, d } => vec![d.into(), s.into(), Immediate(cc.into())],
            Instruction::MovNS { t, s, d } => vec![d.into(), s.into(), t.into()],
            Instruction::MovZS { t, s, d } => vec![d.into(), s.into(), t.into()],
            Instruction::MovD { s, d } => vec![d.into(), s.into()],
            Instruction::MovFD { cc, s, d } => vec![d.into(), s.into(), Immediate(cc.into())],
            Instruction::MovTD { cc, s, d } => vec![d.into(), s.into(), Immediate(cc.into())],
            Instruction::MovND { t, s, d } => vec![d.into(), s.into(), t.into()],
            Instruction::MovZD { t, s, d } => vec![d.into(), s.into(), t.into()],
            Instruction::MovF { s, cc, d } => vec![d.into(), s.into(), Immediate(cc.into())],
            Instruction::MovT { s, cc, d } => vec![d.into(), s.into(), Immediate(cc.into())],
            Instruction::MovN { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::MovZ { s, t, d } => vec![d.into(), s.into(), t.into()],
            Instruction::CvtSW { s, d } => vec![d.into(), s.into()],
            Instruction::CvtWS { s, d } => vec![d.into(), s.into()],
            Instruction::CvtDS { s, d } => vec![d.into(), s.into()],
            Instruction::CvtSD { s, d } => vec![d.into(), s.into()],
            Instruction::CvtDW { s, d } => vec![d.into(), s.into()],
            Instruction::CvtWD { s, d } => vec![d.into(), s.into()],
            Instruction::Mtc1 { t, s } => vec![t.into(), s.into()],
            Instruction::Mfc1 { t, s } => vec![t.into(), s.into()],
            Instruction::Lwc1 { base, t, offset } => vec![t.into(), Offset(offset, base)],
            Instruction::Swc1 { base, t, offset } => vec![t.into(), Offset(offset, base)],
            Instruction::Ldc1 { base, t, offset } => vec![t.into(), Offset(offset, base)],
            Instruction::Sdc1 { base, t, offset } => vec![t.into(), Offset(offset, base)],
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
            Instruction::AddS { t, s, d } => write!(f, "add.s {}, {}, {}", d, s, t),
            Instruction::SubS { t, s, d } => write!(f, "sub.s {}, {}, {}", d, s, t),
            Instruction::MulS { t, s, d } => write!(f, "mul.s {}, {}, {}", d, s, t),
            Instruction::DivS { t, s, d } => write!(f, "div.s {}, {}, {}", d, s, t),
            Instruction::SqrtS { s, d } => write!(f, "sqrt.s {}, {}", d, s),
            Instruction::AbsS { s, d } => write!(f, "abs.s {}, {}", d, s),
            Instruction::NegS { s, d } => write!(f, "neg.s {}, {}", d, s),
            Instruction::FloorWS { s, d } => write!(f, "floor.w.s {}, {}", d, s),
            Instruction::CeilWS { s, d } => write!(f, "ceil.w.s {}, {}", d, s),
            Instruction::RoundWS { s, d } => write!(f, "round.w.s {}, {}", d, s),
            Instruction::TruncWS { s, d } => write!(f, "trunc.w.s {}, {}", d, s),
            Instruction::AddD { t, s, d } => write!(f, "add.d {}, {}, {}", d, s, t),
            Instruction::SubD { t, s, d } => write!(f, "sub.d {}, {}, {}", d, s, t),
            Instruction::MulD { t, s, d } => write!(f, "mul.d {}, {}, {}", d, s, t),
            Instruction::DivD { t, s, d } => write!(f, "div.d {}, {}, {}", d, s, t),
            Instruction::SqrtD { s, d } => write!(f, "sqrt.d {}, {}", d, s),
            Instruction::AbsD { s, d } => write!(f, "abs.d {}, {}", d, s),
            Instruction::NegD { s, d } => write!(f, "neg.d {}, {}", d, s),
            Instruction::FloorWD { s, d } => write!(f, "floor.w.d {}, {}", d, s),
            Instruction::CeilWD { s, d } => write!(f, "ceil.w.d {}, {}", d, s),
            Instruction::RoundWD { s, d } => write!(f, "round.w.d {}, {}", d, s),
            Instruction::TruncWD { s, d } => write!(f, "trunc.w.d {}, {}", d, s),
            Instruction::CEqS { t, s, cc } => write!(f, "c.eq.s {}, {}, {}", *cc, s, t),
            Instruction::CLeS { t, s, cc } => write!(f, "c.le.s {}, {}, {}", *cc, s, t),
            Instruction::CLtS { t, s, cc } => write!(f, "c.lt.s {}, {}, {}", *cc, s, t),
            Instruction::CEqD { t, s, cc } => write!(f, "c.eq.d {}, {}, {}", *cc, s, t),
            Instruction::CLeD { t, s, cc } => write!(f, "c.le.d {}, {}, {}", *cc, s, t),
            Instruction::CLtD { t, s, cc } => write!(f, "c.lt.d {}, {}, {}", *cc, s, t),
            Instruction::BC1T { cc, offset } => write!(f, "bc1t {}, 0x{:x}", *cc, offset),
            Instruction::BC1F { cc, offset } => write!(f, "bc1f {}, 0x{:x}", *cc, offset),
            Instruction::MovS { s, d } => write!(f, "mov.s {}, {}", d, s),
            Instruction::MovFS { cc, s, d } => write!(f, "movf.s {}, {}, {}", d, s, *cc),
            Instruction::MovTS { cc, s, d } => write!(f, "movt.s {}, {}, {}", d, s, *cc),
            Instruction::MovNS { t, s, d } => write!(f, "movn.s {}, {}, {}", d, s, t),
            Instruction::MovZS { t, s, d } => write!(f, "movz.s {}, {}, {}", d, s, t),
            Instruction::MovD { s, d } => write!(f, "mov.d {}, {}", d, s),
            Instruction::MovFD { cc, s, d } => write!(f, "movf.d {}, {}, {}", d, s, *cc),
            Instruction::MovTD { cc, s, d } => write!(f, "movt.d {}, {}, {}", d, s, *cc),
            Instruction::MovND { t, s, d } => write!(f, "movn.d {}, {}, {}", d, s, t),
            Instruction::MovZD { t, s, d } => write!(f, "movz.d {}, {}, {}", d, s, t),
            Instruction::MovF { s, cc, d } => write!(f, "movf {}, {}, {}", d, s, *cc),
            Instruction::MovT { s, cc, d } => write!(f, "movt {}, {}, {}", d, s, *cc),
            Instruction::MovN { s, t, d } => write!(f, "movn {}, {}, {}", d, s, t),
            Instruction::MovZ { s, t, d } => write!(f, "movz {}, {}, {}", d, s, t),
            Instruction::CvtSW { s, d } => write!(f, "cvt.s.w {}, {}", d, s),
            Instruction::CvtWS { s, d } => write!(f, "cvt.w.s {}, {}", d, s),
            Instruction::CvtDS { s, d } => write!(f, "cvt.d.s {}, {}", d, s),
            Instruction::CvtSD { s, d } => write!(f, "cvt.s.d {}, {}", d, s),
            Instruction::CvtDW { s, d } => write!(f, "cvt.d.w {}, {}", d, s),
            Instruction::CvtWD { s, d } => write!(f, "cvt.w.d {}, {}", d, s),
            Instruction::Mtc1 { t, s } => write!(f, "mtc1 {}, {}", t, s),
            Instruction::Mfc1 { t, s } => write!(f, "mfc1 {}, {}", t, s),
            Instruction::Lwc1 { base, t, offset } => {
                write!(f, "lwc1 {}, {}({})", t, sig(*offset), base)
            }
            Instruction::Swc1 { base, t, offset } => {
                write!(f, "swc1 {}, {}({})", t, sig(*offset), base)
            }
            Instruction::Ldc1 { base, t, offset } => {
                write!(f, "ldc1 {}, {}({})", t, sig(*offset), base)
            }
            Instruction::Sdc1 { base, t, offset } => {
                write!(f, "sdc1 {}, {}({})", t, sig(*offset), base)
            }
        }
    }
}
