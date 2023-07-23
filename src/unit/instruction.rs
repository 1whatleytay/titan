use crate::cpu::decoder::Decoder;
use crate::unit::register::RegisterName;
use num::FromPrimitive;
use num_traits::Signed;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Instruction {
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
    Beq { s: RegisterName, t: RegisterName, address: u32 },
    Bne { s: RegisterName, t: RegisterName, address: u32 },
    Bgtz { s: RegisterName, address: u32 },
    Blez { s: RegisterName, address: u32 },
    Bltz { s: RegisterName, address: u32 },
    Bgez { s: RegisterName, address: u32 },
    Bltzal { s: RegisterName, address: u32 },
    Bgezal { s: RegisterName, address: u32 },
    J { address: u32 },
    Jal { address: u32 },
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

fn sig(imm: u16) -> String {
    let value = imm as i16 as i64;

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
    address: u32
}

impl InstructionDecoder {
    pub fn decode(address: u32, instruction: u32) -> Option<Instruction> {
        InstructionDecoder { address }.dispatch(instruction)
    }
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
        Instruction::Beq { s: s.into(), t: t.into(), address: rel_dest(self.address, imm) }
    }

    fn bne(&mut self, s: u8, t: u8, imm: u16) -> Instruction {
        Instruction::Bne { s: s.into(), t: t.into(), address: rel_dest(self.address, imm) }
    }

    fn bgtz(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Bgtz { s: s.into(), address: rel_dest(self.address, imm) }
    }

    fn blez(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Blez { s: s.into(), address: rel_dest(self.address, imm) }
    }

    fn bltz(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Bltz { s: s.into(), address: rel_dest(self.address, imm) }
    }

    fn bgez(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Bgez { s: s.into(), address: rel_dest(self.address, imm) }
    }

    fn bltzal(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Bltzal { s: s.into(), address: rel_dest(self.address, imm) }
    }

    fn bgezal(&mut self, s: u8, imm: u16) -> Instruction {
        Instruction::Bgezal { s: s.into(), address: rel_dest(self.address, imm) }
    }

    fn j(&mut self, imm: u32) -> Instruction {
        Instruction::J { address: jump_dest(self.address, imm) }
    }

    fn jal(&mut self, imm: u32) -> Instruction {
        Instruction::Jal { address: jump_dest(self.address, imm) }
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

impl ToString for Instruction {
    fn to_string(&self) -> String {
        match self {
            Instruction::Add { s, t, d } => format!("add {}, {}, {}", d, s, t),
            Instruction::Addu { s, t, d } => format!("addu {}, {}, {}", d, s, t),
            Instruction::And { s, t, d } => format!("and {}, {}, {}", d, s, t),
            Instruction::Div { s, t } => format!("div {}, {}", s, t),
            Instruction::Divu { s, t } => format!("divu {}, {}", s, t),
            Instruction::Mult { s, t } => format!("mult {}, {}", s, t),
            Instruction::Multu { s, t } => format!("multu {}, {}", s, t),
            Instruction::Nor { s, t, d } => format!("nor {}, {}, {}", d, s, t),
            Instruction::Or { s, t, d } => format!("or {}, {}, {}", d, s, t),
            Instruction::Sll { t, d, sham } => format!("sll {}, {}, {}", d, t, sham),
            Instruction::Sllv { s, t, d } => format!("sllv {}, {}, {}", d, t, s),
            Instruction::Sra { t, d, sham } => format!("sra {}, {}, {}", d, t, sham),
            Instruction::Srav { s, t, d } => format!("srav {}, {}, {}", d, t, s),
            Instruction::Srl { t, d, sham } => format!("srl {}, {}, {}", d, t, sham),
            Instruction::Srlv { s, t, d } => format!("srlv {}, {}, {}", d, t, s),
            Instruction::Sub { s, t, d } => format!("sub {}, {}, {}", s, t, d),
            Instruction::Subu { s, t, d } => format!("subu {}, {}, {}", s, t, d),
            Instruction::Xor { s, t, d } => format!("xor {}, {}, {}", s, t, d),
            Instruction::Slt { s, t, d } => format!("slt {}, {}, {}", s, t, d),
            Instruction::Sltu { s, t, d } => format!("sltu {}, {}, {}", s, t, d),
            Instruction::Jr { s } => format!("jr {}", s),
            Instruction::Jalr { s } => format!("jalr {}", s),
            Instruction::Madd { s, t } => format!("madd {}, {}", s, t),
            Instruction::Maddu { s, t } => format!("maddu {}, {}", s, t),
            Instruction::Mul { s, t, d } => format!("mul {}, {}, {}", d, s, t),
            Instruction::Msub { s, t } => format!("msub {}, {}", s, t),
            Instruction::Msubu { s, t } => format!("msubu {}, {}", s, t),
            Instruction::Addi { s, t, imm } => format!("addi {}, {}, {}", t, s, sig(*imm)),
            Instruction::Addiu { s, t, imm } => format!("addiu {}, {}, {}", t, s, sig(*imm)),
            Instruction::Andi { s, t, imm } => format!("andi {}, {}, {}", t, s, sig(*imm)),
            Instruction::Ori { s, t, imm } => format!("ori {}, {}, {}", t, s, sig(*imm)),
            Instruction::Xori { s, t, imm } => format!("xori {}, {}, {}", t, s, sig(*imm)),
            Instruction::Lui { s, imm } => format!("lui {}, {}", s, sig(*imm)),
            Instruction::Lhi { t, imm } => format!("lhi {}, {}", t, sig(*imm)),
            Instruction::Llo { t, imm } => format!("llo {}, {}", t, sig(*imm)),
            Instruction::Slti { s, t, imm } => format!("slti {}, {}, {}", t, s, sig(*imm)),
            Instruction::Sltiu { s, t, imm } => format!("sltiu {}, {}, {}", t, s, sig(*imm)),
            Instruction::Beq { s, t, address } => format!("beq {}, {}, 0x{:x}", s, t, address),
            Instruction::Bne { s, t, address } => format!("bne {}, {}, 0x{:x}", s, t, address),
            Instruction::Bgtz { s, address } => format!("bgtz {}, 0x{:x}", s, address),
            Instruction::Blez { s, address } => format!("blez {}, 0x{:x}", s, address),
            Instruction::Bltz { s, address } => format!("bltz {}, 0x{:x}", s, address),
            Instruction::Bgez { s, address } => format!("bgez {}, 0x{:x}", s, address),
            Instruction::Bltzal { s, address } => format!("bltzal {}, 0x{:x}", s, address),
            Instruction::Bgezal { s, address } => format!("bgezal {}, 0x{:x}", s, address),
            Instruction::J { address } => format!("j 0x{:x}", address),
            Instruction::Jal { address } => format!("jal 0x{:x}", address),
            Instruction::Lb { s, t, imm } => format!("lb {}, {}, {}", s, t, sig(*imm)),
            Instruction::Lbu { s, t, imm } => format!("lbu {}, {}, {}", s, t, sig(*imm)),
            Instruction::Lh { s, t, imm } => format!("lh {}, {}, {}", s, t, sig(*imm)),
            Instruction::Lhu { s, t, imm } => format!("lhu {}, {}, {}", s, t, sig(*imm)),
            Instruction::Lw { s, t, imm } => format!("lw {}, {}, {}", s, t, sig(*imm)),
            Instruction::Sb { s, t, imm } => format!("sb {}, {}, {}", s, t, sig(*imm)),
            Instruction::Sh { s, t, imm } => format!("sh {}, {}, {}", s, t, sig(*imm)),
            Instruction::Sw { s, t, imm } => format!("sw {}, {}, {}", s, t, sig(*imm)),
            Instruction::Mfhi { d } => format!("mfhi {}", d),
            Instruction::Mflo { d } => format!("mflo {}", d),
            Instruction::Mthi { s } => format!("mthi {}", s),
            Instruction::Mtlo { s } => format!("mtlo {}", s),
            Instruction::Trap => "trap".to_string(),
            Instruction::Syscall => "syscall".to_string(),
        }
    }
}
