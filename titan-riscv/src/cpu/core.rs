use titan_shared::cpu::error::Error::CpuInvalid;
use titan_shared::cpu::Memory;
use titan_shared::cpu::error::Result;
use crate::cpu::decoder::{Decoder, InstructionSize};
use crate::cpu::registers::Registers;
use crate::cpu::registers::WhichRegister::{Line, Pc};
use crate::cpu::state::State;

impl<Mem: Memory, Reg: Registers> State<Mem, Reg> {
    fn reg(&mut self, index: u8) -> u32 {
        if index == 0 {
            0
        } else {
            self.registers.get(Line(index))
        }
    }

    fn set_reg(&mut self, index: u8, value: u32) {
        if index != 0 {
            self.registers.set(Line(index), value);
        }
    }

    // fn jump(&mut self, bits: u32) {
    //     self.registers.set(
    //         Pc,
    //         (self.registers.get(Pc) & 0xFC000000) | bits.wrapping_shl(2),
    //     );
    // }

    pub fn step(&mut self) -> Result<()> {
        let start = self.registers.get(Pc);
        let instruction = self.memory.get_u32(start)?;

        let (result, size) = self.dispatch(instruction)
            .unwrap_or((Err(CpuInvalid(instruction)), InstructionSize::Regular));

        self.registers.step_pc(size);

        result
            .inspect_err(|_| self.registers.set(Pc, start)); // if error, keep pc here

        todo!(); // more cleverness is needed for handling size a step_pc properly
    }
}


impl<Mem: Memory, Reg: Registers> Decoder<Result<()>> for State<Mem, Reg> {
    fn lui(&mut self, rd: u8, imm_upper: u32) -> Result<()> {
        todo!()
    }

    fn auipc(&mut self, rd: u8, imm_upper: u32) -> Result<()> {
        todo!()
    }

    fn jal(&mut self, rd: u8, imm_jump: i32) -> Result<()> {
        todo!()
    }

    fn beq(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> Result<()> {
        todo!()
    }

    fn bne(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> Result<()> {
        todo!()
    }

    fn blt(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> Result<()> {
        todo!()
    }

    fn bge(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> Result<()> {
        todo!()
    }

    fn bltu(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> Result<()> {
        todo!()
    }

    fn bgeu(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> Result<()> {
        todo!()
    }

    fn jalr(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        todo!()
    }

    fn lb(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        todo!()
    }

    fn lh(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        todo!()
    }

    fn lw(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        todo!()
    }

    fn lbu(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        todo!()
    }

    fn lhu(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        todo!()
    }

    fn addi(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        todo!()
    }

    fn slti(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        todo!()
    }

    fn sltiu(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        todo!()
    }

    fn xori(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        todo!()
    }

    fn ori(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        todo!()
    }

    fn andi(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        todo!()
    }

    fn sb(&mut self, rs1: u8, rs2: u8, imm_store: i16) -> Result<()> {
        todo!()
    }

    fn sh(&mut self, rs1: u8, rs2: u8, imm_store: i16) -> Result<()> {
        todo!()
    }

    fn sw(&mut self, rs1: u8, rs2: u8, imm_store: i16) -> Result<()> {
        todo!()
    }

    fn slli(&mut self, rd: u8, rs1: u8, sham: u8) -> Result<()> {
        todo!()
    }

    fn srli(&mut self, rd: u8, rs1: u8, sham: u8) -> Result<()> {
        todo!()
    }

    fn srai(&mut self, rd: u8, rs1: u8, sham: u8) -> Result<()> {
        todo!()
    }

    fn add(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn sub(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn sll(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn slt(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn sltu(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn xor(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn srl(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn sra(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn or(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn and(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn fence(&mut self) -> Result<()> {
        todo!()
    }

    fn ecall(&mut self) -> Result<()> {
        todo!()
    }

    fn ebreak(&mut self) -> Result<()> {
        todo!()
    }

    fn mul(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn mulh(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn mulhsu(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn mulhu(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn div(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn divu(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn rem(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn remu(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn c_lwsp(&mut self, rd: u8, uimm_54276: u8) -> Result<()> {
        todo!()
    }

    fn c_swsp(&mut self, rs2: u8, uimm_5276: u8) -> Result<()> {
        todo!()
    }

    fn c_lw(&mut self, rd_small: u8, rs1_small: u8, uimm_5326: u8) -> Result<()> {
        todo!()
    }

    fn c_sw(&mut self, rs1_small: u8, rs2_small: u8, uimm_5326: u8) -> Result<()> {
        todo!()
    }

    fn c_j(&mut self, imm_jump: i16) -> Result<()> {
        todo!()
    }

    fn c_jal(&mut self, imm_jump: i16) -> Result<()> {
        todo!()
    }

    fn c_jr(&mut self, rs1: u8) -> Result<()> {
        todo!()
    }

    fn c_jalr(&mut self, rs1: u8) -> Result<()> {
        todo!()
    }

    fn c_beqz(&mut self, rs1_small: u8, imm_branch: i8) -> Result<()> {
        todo!()
    }

    fn c_bnez(&mut self, rs1_small: u8, imm_branch: i8) -> Result<()> {
        todo!()
    }

    fn c_li(&mut self, rd: u8, imm_540: i8) -> Result<()> {
        todo!()
    }

    fn c_lui(&mut self, rd: u8, imm_540: i8) -> Result<()> {
        todo!()
    }

    fn c_addi(&mut self, rd: u8, imm_540: i8) -> Result<()> {
        todo!()
    }

    fn c_addi16sp(&mut self, imm_946875: i8) -> Result<()> {
        todo!()
    }

    fn c_addi4spn(&mut self, rd_small: u8, uimm_549623: u8) -> Result<()> {
        todo!()
    }

    fn c_slli(&mut self, rd: u8, sham: u8) -> Result<()> {
        todo!()
    }

    fn c_srli(&mut self, rs1_small: u8, sham: u8) -> Result<()> {
        todo!()
    }

    fn c_srai(&mut self, rs1_small: u8, sham: u8) -> Result<()> {
        todo!()
    }

    fn c_andi(&mut self, rs1_small: u8, imm_540: i8) -> Result<()> {
        todo!()
    }

    fn c_mv(&mut self, rd: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn c_add(&mut self, rd: u8, rs2: u8) -> Result<()> {
        todo!()
    }

    fn c_and(&mut self, rs1_small: u8, rs2_small: u8) -> Result<()> {
        todo!()
    }

    fn c_or(&mut self, rs1_small: u8, rs2_small: u8) -> Result<()> {
        todo!()
    }

    fn c_xor(&mut self, rs1_small: u8, rs2_small: u8) -> Result<()> {
        todo!()
    }

    fn c_sub(&mut self, rs1_small: u8, rs2_small: u8) -> Result<()> {
        todo!()
    }

    fn c_nop(&mut self) -> Result<()> {
        todo!()
    }

    fn c_ebreak(&mut self) -> Result<()> {
        todo!()
    }
}
