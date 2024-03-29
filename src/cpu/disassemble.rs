use crate::cpu::decoder::Decoder;
use num_traits::abs;

pub trait LabelProvider {
    fn label_for(&mut self, address: u32) -> String;
}

#[derive(Default)]
pub struct HexLabelProvider {}

impl LabelProvider for HexLabelProvider {
    fn label_for(&mut self, address: u32) -> String {
        format!("0x{address:08x}")
    }
}

pub struct Disassembler<Provider: LabelProvider> {
    pub pc: u32,
    pub labels: Provider,
}

fn jump_dest(pc: u32, imm: u32) -> u32 {
    ((pc + 4) & 0xFC000000) | (imm << 2)
}

fn rel_dest(pc: u32, imm: u16) -> u32 {
    ((pc + 4) as i32 + ((imm as i16 as i32) << 2)) as u32
}

fn reg(value: u8) -> &'static str {
    match value {
        0 => "$zero",
        1 => "$at",
        2 => "$v0",
        3 => "$v1",
        4 => "$a0",
        5 => "$a1",
        6 => "$a2",
        7 => "$a3",
        8 => "$t0",
        9 => "$t1",
        10 => "$t2",
        11 => "$t3",
        12 => "$t4",
        13 => "$t5",
        14 => "$t6",
        15 => "$t7",
        16 => "$s0",
        17 => "$s1",
        18 => "$s2",
        19 => "$s3",
        20 => "$s4",
        21 => "$s5",
        22 => "$s6",
        23 => "$s7",
        24 => "$t8",
        25 => "$t9",
        26 => "$k0",
        27 => "$k1",
        28 => "$gp",
        29 => "$sp",
        30 => "$fp",
        31 => "$ra",

        _ => "$unk",
    }
}

fn uns(imm: u16) -> String {
    if imm < 10 {
        format!("{imm}")
    } else {
        format!("0x{imm:x}")
    }
}

fn sig(imm: u16) -> String {
    let value = imm as i16 as i64;

    if abs(value) < 10 {
        format!("{value}")
    } else {
        let sign = if value < 0 { "-" } else { "" };

        format!("{}0x{:x}", sign, abs(value))
    }
}

fn hex(imm: u16) -> String {
    format!("0x{imm:x}")
}

impl<Provider: LabelProvider> Decoder<String> for Disassembler<Provider> {
    fn add(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("add {}, {}, {}", reg(d), reg(s), reg(t))
    }

    fn addu(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("addu {}, {}, {}", reg(d), reg(s), reg(t))
    }

    fn and(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("and {}, {}, {}", reg(d), reg(s), reg(t))
    }

    fn div(&mut self, s: u8, t: u8) -> String {
        format!("div {}, {}", reg(s), reg(t))
    }

    fn divu(&mut self, s: u8, t: u8) -> String {
        format!("divu {}, {}", reg(s), reg(t))
    }

    fn mult(&mut self, s: u8, t: u8) -> String {
        format!("mult {}, {}", reg(s), reg(t))
    }

    fn multu(&mut self, s: u8, t: u8) -> String {
        format!("multu {}, {}", reg(s), reg(t))
    }

    fn nor(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("nor {}, {}, {}", reg(d), reg(s), reg(t))
    }

    fn or(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("or {}, {}, {}", reg(d), reg(s), reg(t))
    }

    fn sll(&mut self, t: u8, d: u8, sham: u8) -> String {
        format!("sll {}, {}, {}", reg(d), reg(t), uns(sham as u16))
    }

    fn sllv(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("sllv {}, {}, {}", reg(d), reg(t), reg(s))
    }

    fn sra(&mut self, t: u8, d: u8, sham: u8) -> String {
        format!("sra {}, {}, {}", reg(d), reg(t), uns(sham as u16))
    }

    fn srav(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("srav {}, {}, {}", reg(d), reg(t), reg(s))
    }

    fn srl(&mut self, t: u8, d: u8, sham: u8) -> String {
        format!("srl {}, {}, {}", reg(d), reg(t), uns(sham as u16))
    }

    fn srlv(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("srlv {}, {}, {}", reg(d), reg(t), reg(s))
    }

    fn sub(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("sub {}, {}, {}", reg(d), reg(s), reg(t))
    }

    fn subu(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("subu {}, {}, {}", reg(d), reg(s), reg(t))
    }

    fn xor(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("xor {}, {}, {}", reg(d), reg(s), reg(t))
    }

    fn slt(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("slt {}, {}, {}", reg(d), reg(s), reg(t))
    }

    fn sltu(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("sltu {}, {}, {}", reg(d), reg(s), reg(t))
    }

    fn jr(&mut self, s: u8) -> String {
        format!("jr {}", reg(s))
    }

    fn jalr(&mut self, s: u8) -> String {
        format!("jalr {}", reg(s))
    }

    fn madd(&mut self, s: u8, t: u8) -> String {
        format!("madd {}, {}", reg(s), reg(t))
    }

    fn maddu(&mut self, s: u8, t: u8) -> String {
        format!("maddu {}, {}", reg(s), reg(t))
    }

    fn mul(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("mul {}, {}, {}", reg(d), reg(s), reg(t))
    }

    fn msub(&mut self, s: u8, t: u8) -> String {
        format!("msub {}, {}", reg(s), reg(t))
    }

    fn msubu(&mut self, s: u8, t: u8) -> String {
        format!("msubu {}, {}", reg(s), reg(t))
    }

    fn addi(&mut self, s: u8, t: u8, imm: u16) -> String {
        format!("addi {}, {}, {}", reg(t), reg(s), sig(imm))
    }

    fn addiu(&mut self, s: u8, t: u8, imm: u16) -> String {
        format!("addiu {}, {}, {}", reg(t), reg(s), sig(imm))
    }

    fn andi(&mut self, s: u8, t: u8, imm: u16) -> String {
        format!("andi {}, {}, {}", reg(t), reg(s), hex(imm))
    }

    fn ori(&mut self, s: u8, t: u8, imm: u16) -> String {
        format!("ori {}, {}, {}", reg(t), reg(s), hex(imm))
    }

    fn xori(&mut self, s: u8, t: u8, imm: u16) -> String {
        format!("xori {}, {}, {}", reg(t), reg(s), hex(imm))
    }

    fn lui(&mut self, t: u8, imm: u16) -> String {
        format!("lui {}, {}", reg(t), hex(imm))
    }

    fn lhi(&mut self, t: u8, imm: u16) -> String {
        format!("lhi {}, {}", reg(t), hex(imm))
    }

    fn llo(&mut self, t: u8, imm: u16) -> String {
        format!("llo {}, {}", reg(t), hex(imm))
    }

    fn slti(&mut self, s: u8, t: u8, imm: u16) -> String {
        format!("slti {}, {}, {}", reg(t), reg(s), sig(imm))
    }

    fn sltiu(&mut self, s: u8, t: u8, imm: u16) -> String {
        format!("sltiu {}, {}, {}", reg(t), reg(s), uns(imm))
    }

    fn beq(&mut self, s: u8, t: u8, imm: u16) -> String {
        let label = self.labels.label_for(rel_dest(self.pc, imm));

        format!("beq {}, {}, {}", reg(s), reg(t), label)
    }

    fn bne(&mut self, s: u8, t: u8, imm: u16) -> String {
        let label = self.labels.label_for(rel_dest(self.pc, imm));

        format!("bne {}, {}, {}", reg(s), reg(t), label)
    }

    fn bgtz(&mut self, s: u8, imm: u16) -> String {
        let label = self.labels.label_for(rel_dest(self.pc, imm));

        format!("bgtz {}, {}", reg(s), label)
    }

    fn blez(&mut self, s: u8, imm: u16) -> String {
        let label = self.labels.label_for(rel_dest(self.pc, imm));

        format!("blez {}, {}", reg(s), label)
    }

    fn bltz(&mut self, s: u8, imm: u16) -> String {
        let label = self.labels.label_for(rel_dest(self.pc, imm));

        format!("bltz {}, {}", reg(s), label)
    }

    fn bgez(&mut self, s: u8, imm: u16) -> String {
        let label = self.labels.label_for(rel_dest(self.pc, imm));

        format!("bgez {}, {}", reg(s), label)
    }

    fn bltzal(&mut self, s: u8, imm: u16) -> String {
        let label = self.labels.label_for(rel_dest(self.pc, imm));

        format!("bltzal {}, {}", reg(s), label)
    }

    fn bgezal(&mut self, s: u8, imm: u16) -> String {
        let label = self.labels.label_for(rel_dest(self.pc, imm));

        format!("bgezal {}, {}", reg(s), label)
    }

    fn j(&mut self, imm: u32) -> String {
        format!("j {}", self.labels.label_for(jump_dest(self.pc, imm)))
    }

    fn jal(&mut self, imm: u32) -> String {
        format!("jal {}", self.labels.label_for(jump_dest(self.pc, imm)))
    }

    fn lb(&mut self, s: u8, t: u8, imm: u16) -> String {
        format!("lb {}, {}({})", reg(t), sig(imm), reg(s))
    }

    fn lbu(&mut self, s: u8, t: u8, imm: u16) -> String {
        format!("lbu {}, {}({})", reg(t), sig(imm), reg(s))
    }

    fn lh(&mut self, s: u8, t: u8, imm: u16) -> String {
        format!("lh {}, {}({})", reg(t), sig(imm), reg(s))
    }

    fn lhu(&mut self, s: u8, t: u8, imm: u16) -> String {
        format!("lhu {}, {}({})", reg(t), sig(imm), reg(s))
    }

    fn lw(&mut self, s: u8, t: u8, imm: u16) -> String {
        format!("lw {}, {}({})", reg(t), sig(imm), reg(s))
    }

    fn sb(&mut self, s: u8, t: u8, imm: u16) -> String {
        format!("sb {}, {}({})", reg(t), sig(imm), reg(s))
    }

    fn sh(&mut self, s: u8, t: u8, imm: u16) -> String {
        format!("sh {}, {}({})", reg(t), sig(imm), reg(s))
    }

    fn sw(&mut self, s: u8, t: u8, imm: u16) -> String {
        format!("sw {}, {}({})", reg(t), sig(imm), reg(s))
    }

    fn mfhi(&mut self, d: u8) -> String {
        format!("mfhi {}", reg(d))
    }

    fn mflo(&mut self, d: u8) -> String {
        format!("mflo {}", reg(d))
    }

    fn mthi(&mut self, s: u8) -> String {
        format!("mthi {}", reg(s))
    }

    fn mtlo(&mut self, s: u8) -> String {
        format!("mtlo {}", reg(s))
    }

    fn trap(&mut self) -> String {
        "trap".to_string()
    }

    fn syscall(&mut self) -> String {
        "syscall".to_string()
    }
}
