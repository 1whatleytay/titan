use crate::cpu::decoder::Decoder;
use crate::unit::register::RegisterName;
use num::FromPrimitive;

#[allow(dead_code)]
enum Instruction {
    Add { s: RegisterName, t: RegisterName, d: RegisterName },
    Addu { s: RegisterName, t: RegisterName, d: RegisterName },
    And { s: RegisterName, t: RegisterName, d: RegisterName },
    Div { s: RegisterName, t: RegisterName },
    Divu { s: RegisterName, t: RegisterName },
    Mult { s: RegisterName, t: RegisterName },
    Multu { s: RegisterName, t: RegisterName },
    Nor { s: RegisterName, t: RegisterName, d: RegisterName },
    Or { s: RegisterName, t: RegisterName, d: RegisterName },
    Sll { t: RegisterName, d: RegisterName, sham: u8 },
    Sllv { s: RegisterName, t: RegisterName, d: RegisterName },
    Sra { t: RegisterName, d: RegisterName, sham: u8 },
    Srav { s: RegisterName, t: RegisterName, d: RegisterName },
    Srl { t: RegisterName, d: RegisterName, sham: u8 },
    Srlv { s: RegisterName, t: RegisterName, d: RegisterName },
    Sub { s: RegisterName, t: RegisterName, d: RegisterName },
    Subu { s: RegisterName, t: RegisterName, d: RegisterName },
    Xor { s: RegisterName, t: RegisterName, d: RegisterName },
    Slt { s: RegisterName, t: RegisterName, d: RegisterName },
    Sltu { s: RegisterName, t: RegisterName, d: RegisterName },
    Jr { s: RegisterName },
    Jalr { s: RegisterName },
    Madd { s: RegisterName, t: RegisterName },
    Maddu { s: RegisterName, t: RegisterName },
    Mul { s: RegisterName, t: RegisterName, d: RegisterName },
    Msub { s: RegisterName, t: RegisterName },
    Msubu { s: RegisterName, t: RegisterName },
    Addi { s: RegisterName, t: RegisterName, imm: u16 },
    Addiu { s: RegisterName, t: RegisterName, imm: u16 },
    Andi { s: RegisterName, t: RegisterName, imm: u16 },
    Ori { s: RegisterName, t: RegisterName, imm: u16 },
    Xori { s: RegisterName, t: RegisterName, imm: u16 },
    Lui { s: RegisterName, imm: u16 },
    Lhi { t: RegisterName, imm: u16 },
    Llo { t: RegisterName, imm: u16 },
    Slti { s: RegisterName, t: RegisterName, imm: u16 },
    Sltiu { s: RegisterName, t: RegisterName, imm: u16 },
    Beq { s: RegisterName, t: RegisterName, imm: u16 },
    Bne { s: RegisterName, t: RegisterName, imm: u16 },
    Bgtz { s: RegisterName, imm: u16 },
    Blez { s: RegisterName, imm: u16 },
    Bltz { s: RegisterName, imm: u16 },
    Bgez { s: RegisterName, imm: u16 },
    Bltzal { s: RegisterName, imm: u16 },
    Bgezal { s: RegisterName, imm: u16 },
    J { imm: u32 },
    Jal { imm: u32 },
    Lb { s: RegisterName, t: RegisterName, imm: u16 },
    Lbu { s: RegisterName, t: RegisterName, imm: u16 },
    Lh { s: RegisterName, t: RegisterName, imm: u16 },
    Lhu { s: RegisterName, t: RegisterName, imm: u16 },
    Lw { s: RegisterName, t: RegisterName, imm: u16 },
    Sb { s: RegisterName, t: RegisterName, imm: u16 },
    Sh { s: RegisterName, t: RegisterName, imm: u16 },
    Sw { s: RegisterName, t: RegisterName, imm: u16 },
    Mfhi { d: RegisterName },
    Mflo { d: RegisterName },
    Mthi { s: RegisterName },
    Mtlo { s: RegisterName },
    Trap,
    Syscall,
}

impl From<u8> for RegisterName {
    fn from(value: u8) -> Self {
        FromPrimitive::from_u8(value).unwrap()
    }
}

struct InstructionDecoder {

}

impl Decoder<Instruction> for InstructionDecoder {
    fn add(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Add { s: s.into(), t: t.into(), d: d.into() }
    }

    fn addu(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Addu { s: s.into(), t: t.into(), d: d.into() }
    }

    fn and(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::And { s: s.into(), t: t.into(), d: d.into() }
    }

    fn div(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Div { s: s.into(), t: t.into() }
    }

    fn divu(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Divu { s: s.into(), t: t.into() }
    }

    fn mult(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Mult { s: s.into(), t: t.into() }
    }

    fn multu(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Multu { s: s.into(), t: t.into() }
    }

    fn nor(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Nor { s: s.into(), t: t.into(), d: d.into() }
    }

    fn or(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Or { s: s.into(), t: t.into(), d: d.into() }
    }

    fn sll(&mut self, t: u8, d: u8, sham: u8) -> Instruction {
        Instruction::Sll { t: t.into(), d: d.into(), sham }
    }

    fn sllv(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Sllv { s: s.into(), t: t.into(), d: d.into() }
    }

    fn sra(&mut self, t: u8, d: u8, sham: u8) -> Instruction {
        Instruction::Sra { t: t.into(), d: d.into(), sham }
    }

    fn srav(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Srav { s: s.into(), t: t.into(), d: d.into() }
    }

    fn srl(&mut self, t: u8, d: u8, sham: u8) -> Instruction {
        Instruction::Srl { t: t.into(), d: d.into(), sham }
    }

    fn srlv(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Srlv { s: s.into(), t: t.into(), d: d.into() }
    }

    fn sub(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Sub { s: s.into(), t: t.into(), d: d.into() }
    }

    fn subu(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Subu { s: s.into(), t: t.into(), d: d.into() }
    }

    fn xor(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Xor { s: s.into(), t: t.into(), d: d.into() }
    }

    fn slt(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Slt { s: s.into(), t: t.into(), d: d.into() }
    }

    fn sltu(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Sltu { s: s.into(), t: t.into(), d: d.into() }
    }

    fn jr(&mut self, s: u8) -> Instruction {
        Instruction::Jr { s: s.into() }
    }

    fn jalr(&mut self, s: u8) -> Instruction {
        Instruction::Jalr { s: s.into() }
    }

    fn madd(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Madd { s: s.into(), t: t.into() }
    }

    fn maddu(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Maddu { s: s.into(), t: t.into() }
    }

    fn mul(&mut self, s: u8, t: u8, d: u8) -> Instruction {
        Instruction::Mul { s: s.into(), t: t.into(), d: d.into() }
    }

    fn msub(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Msub { s: s.into(), t: t.into() }
    }

    fn msubu(&mut self, s: u8, t: u8) -> Instruction {
        Instruction::Msubu { s: s.into(), t: t.into() }
    }

    fn addi(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Addi { s: s.into(), t: t.into(), imm }
    }

    fn addiu(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Addiu { s: s.into(), t: t.into(), imm }
    }

    fn andi(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Andi { s: s.into(), t: t.into(), imm }
    }

    fn ori(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Ori { s: s.into(), t: t.into(), imm }
    }

    fn xori(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Xori { s: s.into(), t: t.into(), imm }
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
        Instruction::Slti { s: s.into(), t: t.into(), imm }
    }

    fn sltiu(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Sltiu { s: s.into(), t: t.into(), imm }
    }

    fn beq(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Beq { s: s.into(), t: t.into(), imm }
    }

    fn bne(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Bne { s: s.into(), t: t.into(), imm }
    }

    fn bgtz(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Bgtz { s: s.into(), imm }
    }

    fn blez(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Blez { s: s.into(), imm }
    }

    fn bltz(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Bltz { s: s.into(), imm }
    }

    fn bgez(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Bgez { s: s.into(), imm }
    }

    fn bltzal(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Bltzal { s: s.into(), imm }
    }

    fn bgezal(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Bgezal { s: s.into(), imm }
    }

    fn j(&mut self, imm: u32) -> Instruction {
        Instruction::J { imm }
    }

    fn jal(&mut self, imm: u32) -> Instruction {
        Instruction::Jal { imm }
    }

    fn lb(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Lb { s: s.into(), t: t.into(), imm }
    }

    fn lbu(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Lbu { s: s.into(), t: t.into(), imm }
    }

    fn lh(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Lh { s: s.into(), t: t.into(), imm }
    }

    fn lhu(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Lhu { s: s.into(), t: t.into(), imm }
    }

    fn lw(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Lw { s: s.into(), t: t.into(), imm }
    }

    fn sb(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Sb { s: s.into(), t: t.into(), imm }
    }

    fn sh(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Sh { s: s.into(), t: t.into(), imm }
    }

    fn sw(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Sw { s: s.into(), t: t.into(), imm }
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
