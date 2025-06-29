use crate::cpu::decoder::Decoder;
use titan_shared::cpu::disassemble::LabelProvider;
use titan_shared::execution::elf::inspection::{
    InspectionDisassembler, InspectionDisassemblerResult, InspectionReadStrategy,
};

fn jump_dest(pc: u32, imm: i32) -> u32 {
    (pc as i32).wrapping_add(imm << 1) as u32
}

fn reg(value: u8) -> &'static str {
    match value {
        0 => "zero",
        1 => "ra",
        2 => "sp",
        3 => "gp",
        4 => "tp",
        5 => "t0",
        6 => "t1",
        7 => "t2",
        8 => "s0",
        9 => "s1",
        10 => "a0",
        11 => "a1",
        12 => "a2",
        13 => "a3",
        14 => "a4",
        15 => "a5",
        16 => "a6",
        17 => "a7",
        18 => "s2",
        19 => "s3",
        20 => "s4",
        21 => "s5",
        22 => "s6",
        23 => "s7",
        24 => "s8",
        25 => "s9",
        26 => "s10",
        27 => "s11",
        28 => "t3",
        29 => "t4",
        30 => "t5",
        31 => "t6",

        _ => "$unk",
    }
}

fn reg_small(value: u8) -> &'static str {
    reg(value + 8) // small register slots start at 8
}

fn uns(imm: u32) -> String {
    if imm < 10 {
        format!("{imm}")
    } else {
        format!("{imm:#x}")
    }
}

fn sig(imm: i16) -> String {
    let value = imm as i64;

    if value.abs() < 10 {
        format!("{value}")
    } else {
        let sign = if value < 0 { "-" } else { "" };

        format!("{}{:#x}", sign, value.abs())
    }
}

pub struct Disassembler<Provider: LabelProvider> {
    pub pc: u32,
    pub labels: Provider,
}

impl<Provider: LabelProvider> Decoder<String> for Disassembler<Provider> {
    fn lui(&mut self, rd: u8, imm_upper: u32) -> String {
        format!("lui {}, {:#x}", reg(rd), imm_upper)
    }

    fn auipc(&mut self, rd: u8, imm_upper: u32) -> String {
        format!("auipc {}, {:#x}", reg(rd), imm_upper)
    }

    fn jal(&mut self, rd: u8, imm_jump: i32) -> String {
        let label = self.labels.label_for(jump_dest(self.pc, imm_jump));

        format!("jal {}, {}", reg(rd), label)
    }

    fn beq(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> String {
        let label = self.labels.label_for(jump_dest(self.pc, imm_branch as i32));

        format!("beq {}, {}, {}", reg(rs1), reg(rs2), label)
    }

    fn bne(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> String {
        let label = self.labels.label_for(jump_dest(self.pc, imm_branch as i32));

        format!("bne {}, {}, {}", reg(rs1), reg(rs2), label)
    }

    fn blt(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> String {
        let label = self.labels.label_for(jump_dest(self.pc, imm_branch as i32));

        format!("blt {}, {}, {}", reg(rs1), reg(rs2), label)
    }

    fn bge(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> String {
        let label = self.labels.label_for(jump_dest(self.pc, imm_branch as i32));

        format!("bge {}, {}, {}", reg(rs1), reg(rs2), label)
    }

    fn bltu(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> String {
        let label = self.labels.label_for(jump_dest(self.pc, imm_branch as i32));

        format!("bltu {}, {}, {}", reg(rs1), reg(rs2), label)
    }

    fn bgeu(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> String {
        let label = self.labels.label_for(jump_dest(self.pc, imm_branch as i32));

        format!("bgeu {}, {}, {}", reg(rs1), reg(rs2), label)
    }

    fn jalr(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> String {
        format!("jalr {}, {}({})", reg(rd), sig(imm_normal), reg(rs1))
    }

    fn lb(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> String {
        format!("lb {}, {}({})", reg(rd), sig(imm_normal), reg(rs1))
    }

    fn lh(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> String {
        format!("lh {}, {}({})", reg(rd), sig(imm_normal), reg(rs1))
    }

    fn lw(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> String {
        format!("lw {}, {}({})", reg(rd), sig(imm_normal), reg(rs1))
    }

    fn lbu(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> String {
        format!("lbu {}, {}({})", reg(rd), sig(imm_normal), reg(rs1))
    }

    fn lhu(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> String {
        format!("lhu {}, {}({})", reg(rd), sig(imm_normal), reg(rs1))
    }

    fn addi(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> String {
        format!("addi {}, {}, {}", reg(rd), reg(rs1), sig(imm_normal))
    }

    fn slti(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> String {
        format!("slti {}, {}, {}", reg(rd), reg(rs1), sig(imm_normal))
    }

    fn sltiu(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> String {
        format!("sltiu {}, {}, {}", reg(rd), reg(rs1), sig(imm_normal))
    }

    fn xori(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> String {
        format!(
            "xori {}, {}, {}",
            reg(rd),
            reg(rs1),
            imm_normal as i32 as u32
        )
    }

    fn ori(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> String {
        format!(
            "ori {}, {}, {}",
            reg(rd),
            reg(rs1),
            imm_normal as i32 as u32
        )
    }

    fn andi(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> String {
        format!(
            "andi {}, {}, {:#X}",
            reg(rd),
            reg(rs1),
            imm_normal as i32 as u32
        )
    }

    fn sb(&mut self, rs1: u8, rs2: u8, imm_store: i16) -> String {
        format!("sb {}, {}({})", reg(rs2), sig(imm_store), reg(rs1))
    }

    fn sh(&mut self, rs1: u8, rs2: u8, imm_store: i16) -> String {
        format!("sh {}, {}({})", reg(rs2), sig(imm_store), reg(rs1))
    }

    fn sw(&mut self, rs1: u8, rs2: u8, imm_store: i16) -> String {
        format!("sw {}, {}({})", reg(rs2), sig(imm_store), reg(rs1))
    }

    fn slli(&mut self, rd: u8, rs1: u8, sham: u8) -> String {
        format!("slli {}, {}, {}", reg(rd), reg(rs1), sham)
    }

    fn srli(&mut self, rd: u8, rs1: u8, sham: u8) -> String {
        format!("srli {}, {}, {}", reg(rd), reg(rs1), sham)
    }

    fn srai(&mut self, rd: u8, rs1: u8, sham: u8) -> String {
        format!("srai {}, {}, {}", reg(rd), reg(rs1), sham)
    }

    fn add(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("add {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn sub(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("sub {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn sll(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("sll {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn slt(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("slt {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn sltu(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("sltu {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn xor(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("xor {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn srl(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("srl {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn sra(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("sra {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn or(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("or {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn and(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("and {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn fence(&mut self) -> String {
        "fence".to_string()
    }

    fn ecall(&mut self) -> String {
        "ecall".to_string()
    }

    fn ebreak(&mut self) -> String {
        "ebreak".to_string()
    }

    // M Extension (MulDiv)
    fn mul(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("mul {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn mulh(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("mulh {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn mulhsu(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("mulhsu {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn mulhu(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("mulhu {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn div(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("div {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn divu(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("divu {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn rem(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("rem {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    fn remu(&mut self, rd: u8, rs1: u8, rs2: u8) -> String {
        format!("remu {}, {}, {}", reg(rd), reg(rs1), reg(rs2))
    }

    // C Extension (Compressed)
    fn c_lwsp(&mut self, rd: u8, uimm_54276: u8) -> String {
        format!("c.lwsp {}, {}", reg(rd), uns((uimm_54276 as u32) << 2))
    }

    fn c_swsp(&mut self, rs2: u8, uimm_5276: u8) -> String {
        format!("c.swsp {}, {}", reg(rs2), uns((uimm_5276 as u32) << 2))
    }

    fn c_lw(&mut self, rd_small: u8, rs1_small: u8, uimm_5326: u8) -> String {
        format!(
            "c.lw {}, {}({})",
            reg_small(rd_small),
            uns((uimm_5326 as u32) << 2),
            reg_small(rs1_small)
        )
    }

    fn c_sw(&mut self, rs1_small: u8, rs2_small: u8, uimm_5326: u8) -> String {
        format!(
            "c.sw {}, {}({})",
            reg_small(rs2_small),
            uns((uimm_5326 as u32) << 2),
            reg_small(rs1_small)
        )
    }

    fn c_j(&mut self, imm_jump: i16) -> String {
        let label = self.labels.label_for(jump_dest(self.pc, imm_jump as i32));

        format!("c.j {}", label)
    }

    fn c_jal(&mut self, imm_jump: i16) -> String {
        let label = self.labels.label_for(jump_dest(self.pc, imm_jump as i32));

        format!("c.jal {}", label)
    }

    fn c_jr(&mut self, rs1: u8) -> String {
        format!("c.jr {}", reg(rs1))
    }

    fn c_jalr(&mut self, rs1: u8) -> String {
        format!("c.jalr {}", reg(rs1))
    }

    fn c_beqz(&mut self, rs1_small: u8, imm_branch: i8) -> String {
        let label = self.labels.label_for(jump_dest(self.pc, imm_branch as i32));

        format!("c.beqz {}, {}", reg_small(rs1_small), label)
    }

    fn c_bnez(&mut self, rs1_small: u8, imm_branch: i8) -> String {
        let label = self.labels.label_for(jump_dest(self.pc, imm_branch as i32));

        format!("c.bnez {}, {}", reg_small(rs1_small), label)
    }

    fn c_li(&mut self, rd: u8, imm_540: i8) -> String {
        format!("c.li {}, {}", reg(rd), sig(imm_540 as i16))
    }

    fn c_lui(&mut self, rd: u8, imm_540: i8) -> String {
        format!("c.lui {}, {}", reg(rd), sig(imm_540 as i16))
    }

    fn c_addi(&mut self, rd: u8, imm_540: i8) -> String {
        format!("c.addi {}, {}", reg(rd), sig(imm_540 as i16))
    }

    fn c_addi16sp(&mut self, imm_946875: i8) -> String {
        format!("c.addi16sp {}", sig((imm_946875 as i16) << 4))
    }

    fn c_addi4spn(&mut self, rd_small: u8, uimm_549623: u8) -> String {
        format!(
            "c.addi4spn {}, {}",
            reg_small(rd_small),
            uns((uimm_549623 as u32) << 2)
        )
    }

    fn c_slli(&mut self, rd: u8, sham: u8) -> String {
        format!("c.slli {}, {}", reg(rd), sham)
    }

    fn c_srli(&mut self, rs1_small: u8, sham: u8) -> String {
        format!("c.srli {}, {}", reg_small(rs1_small), sham)
    }

    fn c_srai(&mut self, rs1_small: u8, sham: u8) -> String {
        format!("c.srai {}, {}", reg_small(rs1_small), sham)
    }

    fn c_andi(&mut self, rs1_small: u8, imm_540: i8) -> String {
        format!("c.andi {}, {}", reg_small(rs1_small), sig(imm_540 as i16))
    }

    fn c_mv(&mut self, rd: u8, rs2: u8) -> String {
        format!("c.mv {}, {}", reg(rd), reg(rs2))
    }

    fn c_add(&mut self, rd: u8, rs2: u8) -> String {
        format!("c.add {}, {}", reg(rd), reg(rs2))
    }

    fn c_and(&mut self, rs1_small: u8, rs2_small: u8) -> String {
        format!("c.and {}, {}", reg_small(rs1_small), reg_small(rs2_small))
    }

    fn c_or(&mut self, rs1_small: u8, rs2_small: u8) -> String {
        format!("c.or {}, {}", reg_small(rs1_small), reg_small(rs2_small))
    }

    fn c_xor(&mut self, rs1_small: u8, rs2_small: u8) -> String {
        format!("c.xor {}, {}", reg_small(rs1_small), reg_small(rs2_small))
    }

    fn c_sub(&mut self, rs1_small: u8, rs2_small: u8) -> String {
        format!("c.sub {}, {}", reg_small(rs1_small), reg_small(rs2_small))
    }

    fn c_nop(&mut self) -> String {
        "c.nop".to_string()
    }

    fn c_ebreak(&mut self) -> String {
        "c.ebreak".to_string()
    }
}

pub struct RiscVInspectionDisassembler;

impl InspectionDisassembler for RiscVInspectionDisassembler {
    fn read_strategy() -> InspectionReadStrategy {
        InspectionReadStrategy::ReadU32PadU16
    }

    fn disassemble(
        &mut self,
        pc: u32,
        instruction: u32,
        labels: &mut impl LabelProvider,
    ) -> Option<InspectionDisassemblerResult> {
        let mut disassembler = Disassembler { pc, labels };

        disassembler
            .dispatch(instruction)
            .map(|(line, size)| InspectionDisassemblerResult {
                line,
                next_pc: pc + size.byte_size(),
            })
    }
}
