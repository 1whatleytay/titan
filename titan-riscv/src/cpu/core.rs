use titan_shared::cpu::error::Error::{CpuBreak, CpuInvalid, CpuSyscall};
use titan_shared::cpu::Memory;
use titan_shared::cpu::error::Result;
use crate::assembler::registers::RegisterSlot;
use crate::cpu::decoder::{Decoder, InstructionSize};
use crate::cpu::registers::Registers;
use crate::cpu::registers::WhichRegister::{Line, Pc};
use crate::cpu::state::State;

impl<Mem: Memory, Reg: Registers> State<Mem, Reg> {
    fn reg(&self, index: u8) -> u32 {
        if index == 0 {
            0
        } else {
            self.registers.get(Line(index))
        }
    }

    fn reg_small(&self, index_small: u8) -> u32 {
        self.reg(index_small + 8)
    }

    fn set_reg(&mut self, index: u8, value: u32) {
        if index != 0 {
            self.registers.set(Line(index), value);
        }
    }

    fn set_reg_small(&mut self, index_small: u8, value: u32) {
        self.set_reg(index_small + 8, value)
    }

    fn jump_address(&mut self, address: u32, size: InstructionSize) {
        // sub instruction size as we are going to add that via step_pc
        let target = address
            .wrapping_sub(size.byte_size());

        self.registers.set(
            Pc,
            target,
        );
    }

    fn jump(&mut self, immediate: i32, size: InstructionSize) {
        let address = (self.registers.get(Pc) as i32)
            .wrapping_add(immediate.wrapping_shl(1)) as u32;

        self.jump_address(address, size)
    }

    pub fn step(&mut self) -> Result<()> {
        let start = self.registers.get(Pc);

        // We want to be able to read the "last u16" at the end of a section.
        let instruction = match self.memory.get_u32(start) {
            Ok(value) => value,
            Err(err) => self.memory.get_u16(start)
                .map(|value| value as u32)
                .map_err(|_| err)?, // Transparent u16 readings.
        };

        let (result, size) = self.dispatch(instruction)
            .unwrap_or((Err(CpuInvalid(instruction)), InstructionSize::Regular));

        // Pass the error upwards. Our PC should still be in the right spot.
        result?;

        // Even if a jump occurred, we will increment the PC.
        // Maybe in the future,
        self.registers.step_pc(size);

        Ok(())
    }
}


impl<Mem: Memory, Reg: Registers> Decoder<Result<()>> for State<Mem, Reg> {
    fn lui(&mut self, rd: u8, imm_upper: u32) -> Result<()> {
        self.set_reg(rd, imm_upper << 12);

        Ok(())
    }

    fn auipc(&mut self, rd: u8, imm_upper: u32) -> Result<()> {
        let result = self.registers.get(Pc).wrapping_add(imm_upper << 12);

        self.set_reg(rd, result);

        Ok(())
    }

    fn jal(&mut self, rd: u8, imm_jump: i32) -> Result<()> {
        let return_address = self.registers.get(Pc)
            .wrapping_add(InstructionSize::Regular.byte_size());

        self.set_reg(rd, return_address);

        self.jump(imm_jump, InstructionSize::Regular);

        Ok(())
    }

    fn beq(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> Result<()> {
        if self.reg(rs1) == self.reg(rs2) {
            self.jump(imm_branch as i32, InstructionSize::Regular)
        }

        Ok(())
    }

    fn bne(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> Result<()> {
        if self.reg(rs1) != self.reg(rs2) {
            self.jump(imm_branch as i32, InstructionSize::Regular)
        }

        Ok(())
    }

    fn blt(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> Result<()> {
        if (self.reg(rs1) as i32) < (self.reg(rs2) as i32) {
            self.jump(imm_branch as i32, InstructionSize::Regular)
        }

        Ok(())
    }

    fn bge(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> Result<()> {
        if (self.reg(rs1) as i32) >= (self.reg(rs2) as i32) {
            self.jump(imm_branch as i32, InstructionSize::Regular)
        }

        Ok(())
    }

    fn bltu(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> Result<()> {
        if self.reg(rs1) < self.reg(rs2) {
            self.jump(imm_branch as i32, InstructionSize::Regular)
        }

        Ok(())
    }

    fn bgeu(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> Result<()> {
        if self.reg(rs1) >= self.reg(rs2) {
            self.jump(imm_branch as i32, InstructionSize::Regular)
        }

        Ok(())
    }

    fn jalr(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        let return_address = self.registers.get(Pc)
            .wrapping_add(InstructionSize::Regular.byte_size());

        let address = (self.reg(rs1) as i32)
            .wrapping_add(imm_normal as i32) as u32;

        self.jump_address(address, InstructionSize::Regular);

        self.set_reg(rd, return_address);

        Ok(())
    }

    fn lb(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        let address = (self.reg(rs1) as i32)
            .wrapping_add(imm_normal as i32) as u32;

        self.set_reg(rd, self.memory.get(address)? as i8 as i32 as u32);

        Ok(())
    }

    fn lh(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        let address = (self.reg(rs1) as i32)
            .wrapping_add(imm_normal as i32) as u32;

        self.set_reg(rd, self.memory.get_u16(address)? as i16 as i32 as u32);

        Ok(())
    }

    fn lw(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        let address = (self.reg(rs1) as i32)
            .wrapping_add(imm_normal as i32) as u32;

        self.set_reg(rd, self.memory.get_u32(address)? as i32 as u32);

        Ok(())
    }

    fn lbu(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        let address = (self.reg(rs1) as i32)
            .wrapping_add(imm_normal as i32) as u32;

        self.set_reg(rd, self.memory.get(address)? as u32);

        Ok(())
    }

    fn lhu(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        let address = (self.reg(rs1) as i32)
            .wrapping_add(imm_normal as i32) as u32;

        self.set_reg(rd, self.memory.get_u16(address)? as u32);

        Ok(())
    }

    fn addi(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        let result = (self.reg(rs1) as i32)
            .wrapping_add(imm_normal as i32) as u32;

        self.set_reg(rd, result);

        Ok(())
    }

    fn slti(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        let result = if (self.reg(rs1) as i32) < (imm_normal as i32) {
            1
        } else {
            0
        };

        self.set_reg(rd, result);

        Ok(())
    }

    fn sltiu(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        let result = if self.reg(rs1) < (imm_normal as i32 as u32) {
            1
        } else {
            0
        };

        self.set_reg(rd, result);

        Ok(())
    }

    fn xori(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        self.set_reg(rd, self.reg(rs1) ^ (imm_normal as i32 as u32));

        Ok(())
    }

    fn ori(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        self.set_reg(rd, self.reg(rs1) | (imm_normal as i32 as u32));

        Ok(())
    }

    fn andi(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> Result<()> {
        self.set_reg(rd, self.reg(rs1) & (imm_normal as i32 as u32));

        Ok(())
    }

    fn sb(&mut self, rs1: u8, rs2: u8, imm_store: i16) -> Result<()> {
        let address = (self.reg(rs1) as i32)
            .wrapping_add(imm_store as i32) as u32;

        self.memory.set(address, self.reg(rs2) as u8)?;

        Ok(())
    }

    fn sh(&mut self, rs1: u8, rs2: u8, imm_store: i16) -> Result<()> {
        let address = (self.reg(rs1) as i32)
            .wrapping_add(imm_store as i32) as u32;

        self.memory.set_u16(address, self.reg(rs2) as u16)?;

        Ok(())
    }

    fn sw(&mut self, rs1: u8, rs2: u8, imm_store: i16) -> Result<()> {
        let address = (self.reg(rs1) as i32)
            .wrapping_add(imm_store as i32) as u32;

        self.memory.set_u32(address, self.reg(rs2))?;

        Ok(())
    }

    fn slli(&mut self, rd: u8, rs1: u8, sham: u8) -> Result<()> {
        self.set_reg(rd, self.reg(rs1).wrapping_shl(sham as u32));

        Ok(())
    }

    fn srli(&mut self, rd: u8, rs1: u8, sham: u8) -> Result<()> {
        self.set_reg(rd, self.reg(rs1).wrapping_shr(sham as u32));

        Ok(())
    }

    fn srai(&mut self, rd: u8, rs1: u8, sham: u8) -> Result<()> {
        self.set_reg(rd, (self.reg(rs1) as i32).wrapping_shr(sham as u32) as u32);

        Ok(())
    }

    fn add(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        self.set_reg(rd, self.reg(rs1).wrapping_add(self.reg(rs2)));

        Ok(())
    }

    fn sub(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        self.set_reg(rd, self.reg(rs1).wrapping_sub(self.reg(rs2)));

        Ok(())
    }

    fn sll(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        // Mask lower 5-bits - we are a 32-bit arch
        self.set_reg(rd, self.reg(rs1).wrapping_shl(self.reg(rs2) & 0b11111));

        Ok(())
    }

    fn slt(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        let result = if (self.reg(rs1) as i32) < (self.reg(rs2) as i32) {
            1
        } else {
            0
        };

        self.set_reg(rd, result);

        Ok(())
    }

    fn sltu(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        let result = if self.reg(rs1) < self.reg(rs2) {
            1
        } else {
            0
        };

        self.set_reg(rd, result);

        Ok(())
    }

    fn xor(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        self.set_reg(rd, self.reg(rs1) ^ self.reg(rs2));

        Ok(())
    }

    fn srl(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        // Mask lower 5-bits - we are a 32-bit arch
        self.set_reg(rd, self.reg(rs1).wrapping_shr(self.reg(rs2) & 0b11111));

        Ok(())
    }

    fn sra(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        // Mask lower 5-bits - we are a 32-bit arch
        self.set_reg(rd, (self.reg(rs1) as i32).wrapping_shr(self.reg(rs2) & 0b11111) as u32);

        Ok(())
    }

    fn or(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        self.set_reg(rd, self.reg(rs1) | self.reg(rs2));

        Ok(())
    }

    fn and(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        self.set_reg(rd, self.reg(rs1) & self.reg(rs2));

        Ok(())
    }

    fn fence(&mut self) -> Result<()> {
        // Do nothing. No sync work needed.

        Ok(())
    }

    fn ecall(&mut self) -> Result<()> {
        Err(CpuSyscall)
    }

    fn ebreak(&mut self) -> Result<()> {
        Err(CpuBreak)
    }

    fn mul(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        // Low half of multiplication of signed and unsigned is the same.
        // So we don't need mulu or special handling of signed here!
        self.set_reg(rd, self.reg(rs1).wrapping_mul(self.reg(rs2)));

        Ok(())
    }

    fn mulh(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        let result = (self.reg(rs1) as i32 as i64)
            .wrapping_mul(self.reg(rs2) as i32 as i64) as u64;

        self.set_reg(rd, (result >> 32) as u32);

        Ok(())
    }

    fn mulhsu(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        // Signed rs1 and Unsigned rs2
        let result = (self.reg(rs1) as i32 as i64)
            .wrapping_mul(self.reg(rs2) as u64 as i64) as u64;

        self.set_reg(rd, (result >> 32) as u32);

        Ok(())
    }

    fn mulhu(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        let result = (self.reg(rs1) as u64)
            .wrapping_mul(self.reg(rs2) as u64);

        self.set_reg(rd, (result >> 32) as u32);

        Ok(())
    }

    fn div(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        let dividend = self.reg(rs2) as i32;

        let result = if dividend == 0 {
            // Yes, RISC-V says a dividend of zero will set -1.
            -1i32
        } else {
            // If you overflow div, then the result should still be i32::MIN.
            (self.reg(rs1) as i32).checked_div(dividend)
                .unwrap_or(i32::MIN)
        } as u32;

        self.set_reg(rd, result);

        Ok(())
    }

    fn divu(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        let dividend = self.reg(rs2);

        let result = if dividend == 0 {
            // Yes, RISC-V says a dividend of zero will set -1.
            u32::MAX
        } else {
            self.reg(rs1) / dividend
        };

        self.set_reg(rd, result);

        Ok(())
    }

    fn rem(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        let value = self.reg(rs1) as i32;
        let dividend = self.reg(rs2) as i32;

        let result = if dividend == 0 {
            // RISC-V says a dividend of zero will forward the numerator to the remainder.
            value
        } else {
            // If you overflow div, then the result should still be i32::MIN.
            value.checked_rem(dividend)
                .unwrap_or(0)
        } as u32;

        self.set_reg(rd, result);

        Ok(())
    }

    fn remu(&mut self, rd: u8, rs1: u8, rs2: u8) -> Result<()> {
        let value = self.reg(rs1);
        let dividend = self.reg(rs2);

        let result = if dividend == 0 {
            // RISC-V says a dividend of zero will forward the numerator to the remainder.
            value
        } else {
            value % dividend
        };

        self.set_reg(rd, result);

        Ok(())
    }

    fn c_lwsp(&mut self, rd: u8, uimm_54276: u8) -> Result<()> {
        let sp = self.registers.get_l(RegisterSlot::StackPointer);

        let address = sp.wrapping_add((uimm_54276 as u32) << 2);

        self.set_reg(rd, self.memory.get_u32(address)?);

        Ok(())
    }

    fn c_swsp(&mut self, rs2: u8, uimm_5276: u8) -> Result<()> {
        let sp = self.registers.get_l(RegisterSlot::StackPointer);

        let address = sp.wrapping_add((uimm_5276 as u32) << 2);

        self.memory.set_u32(address, self.reg(rs2))?;

        Ok(())
    }

    fn c_lw(&mut self, rd_small: u8, rs1_small: u8, uimm_5326: u8) -> Result<()> {
        let address = self.reg_small(rs1_small)
            .wrapping_add((uimm_5326 as u32) << 2);

        self.set_reg_small(rd_small, self.memory.get_u32(address)?);

        Ok(())
    }

    fn c_sw(&mut self, rs1_small: u8, rs2_small: u8, uimm_5326: u8) -> Result<()> {
        let address = self.reg_small(rs1_small)
            .wrapping_add((uimm_5326 as u32) << 2);

        self.memory.set_u32(address, self.reg_small(rs2_small))?;

        Ok(())
    }

    fn c_j(&mut self, imm_jump: i16) -> Result<()> {
        self.jump(imm_jump as i32, InstructionSize::Compressed);

        Ok(())
    }

    fn c_jal(&mut self, imm_jump: i16) -> Result<()> {
        let return_address = self.registers.get(Pc)
            .wrapping_add(InstructionSize::Compressed.byte_size());

        self.jump(imm_jump as i32, InstructionSize::Compressed);

        self.registers.set_l(RegisterSlot::ReturnAddress, return_address);

        Ok(())
    }

    fn c_jr(&mut self, rs1: u8) -> Result<()> {
        self.jump_address(self.reg(rs1), InstructionSize::Compressed);

        Ok(())
    }

    fn c_jalr(&mut self, rs1: u8) -> Result<()> {
        let return_address = self.registers.get(Pc)
            .wrapping_add(InstructionSize::Compressed.byte_size());

        self.jump_address(self.reg(rs1), InstructionSize::Compressed);

        self.registers.set_l(RegisterSlot::ReturnAddress, return_address);

        Ok(())
    }

    fn c_beqz(&mut self, rs1_small: u8, imm_branch: i8) -> Result<()> {
        if self.reg_small(rs1_small) == 0 {
            self.jump(imm_branch as i32, InstructionSize::Compressed);
        }

        Ok(())
    }

    fn c_bnez(&mut self, rs1_small: u8, imm_branch: i8) -> Result<()> {
        if self.reg_small(rs1_small) != 0 {
            self.jump(imm_branch as i32, InstructionSize::Compressed);
        }

        Ok(())
    }

    fn c_li(&mut self, rd: u8, imm_540: i8) -> Result<()> {
        self.set_reg(rd, imm_540 as i32 as u32);

        Ok(())
    }

    fn c_lui(&mut self, rd: u8, imm_540: i8) -> Result<()> {
        self.set_reg(rd, (imm_540 as i32 as u32) << 12);

        Ok(())
    }

    fn c_addi(&mut self, rd: u8, imm_540: i8) -> Result<()> {
        self.set_reg(rd, (self.reg(rd) as i32).wrapping_add(imm_540 as i32) as u32);

        Ok(())
    }

    fn c_addi16sp(&mut self, imm_946875: i8) -> Result<()> {
        let result = (self.registers.get_l(RegisterSlot::StackPointer) as i32)
            .wrapping_add((imm_946875 as i32) << 4) as u32;

        self.registers.set_l(RegisterSlot::StackPointer, result);

        Ok(())
    }

    fn c_addi4spn(&mut self, rd_small: u8, uimm_549623: u8) -> Result<()> {
        let result = (self.registers.get_l(RegisterSlot::StackPointer) as i32)
            .wrapping_add((uimm_549623 as i32) << 2) as u32;

        self.set_reg_small(rd_small, result);

        Ok(())
    }

    fn c_slli(&mut self, rd: u8, sham: u8) -> Result<()> {
        self.set_reg(rd, self.reg(rd).wrapping_shl(sham as u32));

        Ok(())
    }

    fn c_srli(&mut self, rs1_small: u8, sham: u8) -> Result<()> {
        self.set_reg_small(rs1_small, self.reg_small(rs1_small).wrapping_shr(sham as u32));

        Ok(())
    }

    fn c_srai(&mut self, rs1_small: u8, sham: u8) -> Result<()> {
        let result = (self.reg_small(rs1_small) as i32)
            .wrapping_shr(sham as u32) as u32;

        self.set_reg_small(rs1_small, result);

        Ok(())
    }

    fn c_andi(&mut self, rs1_small: u8, imm_540: i8) -> Result<()> {
        self.set_reg_small(rs1_small, self.reg_small(rs1_small) & (imm_540 as i32 as u32));

        Ok(())
    }

    fn c_mv(&mut self, rd: u8, rs2: u8) -> Result<()> {
        self.set_reg(rd, self.reg(rs2));

        Ok(())
    }

    fn c_add(&mut self, rd: u8, rs2: u8) -> Result<()> {
        self.set_reg(rd, self.reg(rd).wrapping_add(self.reg(rs2)));

        Ok(())
    }

    fn c_and(&mut self, rs1_small: u8, rs2_small: u8) -> Result<()> {
        self.set_reg_small(rs1_small, self.reg_small(rs1_small) & self.reg_small(rs2_small));

        Ok(())
    }

    fn c_or(&mut self, rs1_small: u8, rs2_small: u8) -> Result<()> {
        self.set_reg_small(rs1_small, self.reg_small(rs1_small) | self.reg_small(rs2_small));

        Ok(())
    }

    fn c_xor(&mut self, rs1_small: u8, rs2_small: u8) -> Result<()> {
        self.set_reg_small(rs1_small, self.reg_small(rs1_small) ^ self.reg_small(rs2_small));

        Ok(())
    }

    fn c_sub(&mut self, rs1_small: u8, rs2_small: u8) -> Result<()> {
        self.set_reg_small(rs1_small, self.reg_small(rs1_small)
            .wrapping_sub(self.reg_small(rs2_small)));

        Ok(())
    }

    fn c_nop(&mut self) -> Result<()> {
        // Do nothing.

        Ok(())
    }

    fn c_ebreak(&mut self) -> Result<()> {
        Err(CpuBreak)
    }
}
