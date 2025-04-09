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

fn freg(value: u8) -> &'static str {
    match value {
        0 => "$f0",
        1 => "$f1",
        2 => "$f2",
        3 => "$f3",
        4 => "$f4",
        5 => "$f5",
        6 => "$f6",
        7 => "$f7",
        8 => "$f8",
        9 => "$f9",
        10 => "$f10",
        11 => "$f11",
        12 => "$f12",
        13 => "$f13",
        14 => "$f14",
        15 => "$f15",
        16 => "$f16",
        17 => "$f17",
        18 => "$f18",
        19 => "$f19",
        20 => "$f20",
        21 => "$f21",
        22 => "$f22",
        23 => "$f23",
        24 => "$f24",
        25 => "$f25",
        26 => "$f26",
        27 => "$f27",
        28 => "$f28",
        29 => "$f29",
        30 => "$f30",
        31 => "$f31",
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

    fn add_s(&mut self, t: u8, s: u8, d: u8) -> String {
        format!("add.s {}, {}, {}", freg(d), freg(s), freg(t))
    }
    fn sub_s(&mut self, t: u8, s: u8, d: u8) -> String {
        format!("sub.s {}, {}, {}", freg(d), freg(s), freg(t))
    }
    fn mul_s(&mut self, t: u8, s: u8, d: u8) -> String {
        format!("mul.s {}, {}, {}", freg(d), freg(s), freg(t))
    }
    fn div_s(&mut self, t: u8, s: u8, d: u8) -> String {
        format!("div.s {}, {}, {}", freg(d), freg(s), freg(t))
    }
    fn sqrt_s(&mut self, s: u8, d: u8) -> String {
        format!("sqrt.s {}, {}", freg(d), freg(s))
    }
    fn abs_s(&mut self, s: u8, d: u8) -> String {
        format!("abs.s {}, {}", freg(d), freg(s))
    }
    fn neg_s(&mut self, s: u8, d: u8) -> String {
        format!("neg.s {}, {}", freg(d), freg(s))
    }
    fn floor_w_s(&mut self, s: u8, d: u8) -> String {
        format!("floor.w.s {}, {}", freg(d), freg(s))
    }
    fn ceil_w_s(&mut self, s: u8, d: u8) -> String {
        format!("ceil.w.s {}, {}", freg(d), freg(s))
    }
    fn round_w_s(&mut self, s: u8, d: u8) -> String {
        format!("round.w.s {}, {}", freg(d), freg(s))
    }
    fn trunc_w_s(&mut self, s: u8, d: u8) -> String {
        format!("trunc.w.s {}, {}", freg(d), freg(s))
    }
    fn add_d(&mut self, t: u8, s: u8, d: u8) -> String {
        format!("add.d {}, {}, {}", freg(d), freg(s), freg(t))
    }
    fn sub_d(&mut self, t: u8, s: u8, d: u8) -> String {
        format!("sub.d {}, {}, {}", freg(d), freg(s), freg(t))
    }
    fn mul_d(&mut self, t: u8, s: u8, d: u8) -> String {
        format!("mul.d {}, {}, {}", freg(d), freg(s), freg(t))
    }
    fn div_d(&mut self, t: u8, s: u8, d: u8) -> String {
        format!("div.d {}, {}, {}", freg(d), freg(s), freg(t))
    }
    fn sqrt_d(&mut self, s: u8, d: u8) -> String {
        format!("sqrt.d {}, {}", freg(d), freg(s))
    }
    fn abs_d(&mut self, s: u8, d: u8) -> String {
        format!("abs.d {}, {}", freg(d), freg(s))
    }
    fn neg_d(&mut self, s: u8, d: u8) -> String {
        format!("neg.d {}, {}", freg(d), freg(s))
    }
    fn floor_w_d(&mut self, s: u8, d: u8) -> String {
        format!("floor.w.d {}, {}", freg(d), freg(s))
    }
    fn ceil_w_d(&mut self, s: u8, d: u8) -> String {
        format!("ceil.w.d {}, {}", freg(d), freg(s))
    }
    fn round_w_d(&mut self, s: u8, d: u8) -> String {
        format!("round.w.d {}, {}", freg(d), freg(s))
    }
    fn trunc_w_d(&mut self, s: u8, d: u8) -> String {
        format!("trunc.w.d {}, {}", freg(d), freg(s))
    }
    fn c_eq_s(&mut self, t: u8, s: u8, cc: u8) -> String {
        format!("c.eq.s {}, {}, {}", cc, freg(s), freg(t))
    }
    fn c_le_s(&mut self, t: u8, s: u8, cc: u8) -> String {
        format!("c.le.s {}, {}, {}", cc, freg(s), freg(t))
    }
    fn c_lt_s(&mut self, t: u8, s: u8, cc: u8) -> String {
        format!("c.lt.s {}, {}, {}", cc, freg(s), freg(t))
    }
    fn c_eq_d(&mut self, t: u8, s: u8, cc: u8) -> String {
        format!("c.eq.d {}, {}, {}", cc, freg(s), freg(t))
    }
    fn c_le_d(&mut self, t: u8, s: u8, cc: u8) -> String {
        format!("c.le.d {}, {}, {}", cc, freg(s), freg(t))
    }
    fn c_lt_d(&mut self, t: u8, s: u8, cc: u8) -> String {
        format!("c.lt.d {}, {}, {}", cc, freg(s), freg(t))
    }
    fn bc1t(&mut self, imm: u8, addr: u16) -> String {
        let label = self.labels.label_for(rel_dest(self.pc, addr));

        format!("bc1t {}, {}", imm, label)
    }
    fn bc1f(&mut self, imm: u8, addr: u16) -> String {
        let label = self.labels.label_for(rel_dest(self.pc, addr));

        format!("bc1f {}, {}", imm, label)
    }
    fn mov_s(&mut self, s: u8, d: u8) -> String {
        format!("mov.s {}, {}", freg(d), freg(s))
    }
    fn movf_s(&mut self, cc: u8, s: u8, d: u8) -> String {
        format!("movf.s {}, {}, {}", freg(d), freg(s), cc)
    }
    fn movt_s(&mut self, cc: u8, s: u8, d: u8) -> String {
        format!("movt.s {}, {}, {}", freg(d), freg(s), cc)
    }
    fn movn_s(&mut self, t: u8, s: u8, d: u8) -> String {
        format!("movn.s {}, {}, {}", freg(d), freg(s), freg(t))
    }
    fn movz_s(&mut self, t: u8, s: u8, d: u8) -> String {
        format!("movz.s {}, {}, {}", freg(d), freg(s), freg(t))
    }
    fn mov_d(&mut self, s: u8, d: u8) -> String {
        format!("mov.d {}, {}", freg(d), freg(s))
    }
    fn movf_d(&mut self, cc: u8, s: u8, d: u8) -> String {
        format!("movf.d {}, {}, {}", freg(d), freg(s), cc)
    }
    fn movt_d(&mut self, cc: u8, s: u8, d: u8) -> String {
        format!("movt.d {}, {}, {}", freg(d), freg(s), cc)
    }
    fn movn_d(&mut self, t: u8, s: u8, d: u8) -> String {
        format!("movn.d {}, {}, {}", freg(d), freg(s), freg(t))
    }
    fn movz_d(&mut self, t: u8, s: u8, d: u8) -> String {
        format!("movz.d {}, {}, {}", freg(d), freg(s), freg(t))
    }
    fn movf(&mut self, s: u8, cc: u8, d: u8) -> String {
        format!("movf {}, {}, {}", freg(d), freg(s), cc)
    }
    fn movt(&mut self, s: u8, cc: u8, d: u8) -> String {
        format!("movt {}, {}, {}", freg(d), freg(s), cc)
    }
    fn movn(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("movn {}, {}, {}", freg(d), freg(s), freg(t))
    }
    fn movz(&mut self, s: u8, t: u8, d: u8) -> String {
        format!("movz {}, {}, {}", freg(d), freg(s), freg(t))
    }
    fn cvt_s_w(&mut self, s: u8, d: u8) -> String {
        format!("cvt.s.w {}, {}", freg(d), freg(s))
    }
    fn cvt_w_s(&mut self, s: u8, d: u8) -> String {
        format!("cvt.w.s {}, {}", freg(d), freg(s))
    }
    fn cvt_s_d(&mut self, s: u8, d: u8) -> String {
        format!("cvt.s.d {}, {}", freg(d), freg(s))
    }
    fn cvt_d_s(&mut self, s: u8, d: u8) -> String {
        format!("cvt.d.s {}, {}", freg(d), freg(s))
    }
    fn cvt_w_d(&mut self, s: u8, d: u8) -> String {
        format!("cvt.w.d {}, {}", freg(d), freg(s))
    }
    fn cvt_d_w(&mut self, s: u8, d: u8) -> String {
        format!("cvt.d.w {}, {}", freg(d), freg(s))
    }
    fn mtc1(&mut self, t: u8, s: u8) -> String {
        format!("mtc1 {}, {}", freg(t), reg(s))
    }
    fn mfc1(&mut self, t: u8, s: u8) -> String {
        format!("mfc1 {}, {}", reg(t), freg(s))
    }
    fn ldc1(&mut self, base: u8, t: u8, offset: u16) -> String {
        format!("ldc1 {}, {}({})", freg(t), sig(offset), reg(base))
    }
    fn sdc1(&mut self, base: u8, t: u8, offset: u16) -> String {
        format!("sdc1 {}, {}({})", freg(t), sig(offset), reg(base))
    }
    fn lwc1(&mut self, base: u8, t: u8, offset: u16) -> String {
        format!("lwc1 {}, {}({})", freg(t), sig(offset), reg(base))
    }
    fn swc1(&mut self, base: u8, t: u8, offset: u16) -> String {
        format!("swc1 {}, {}({})", freg(t), sig(offset), reg(base))
    }
}
