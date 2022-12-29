use crate::cpu::decoder::Decoder;
use crate::cpu::error::Error::{CpuInvalid, CpuTrap};
use crate::cpu::{Memory, State};
use crate::cpu::error::Result;

impl<Mem: Memory> State<Mem> {
    fn register(&mut self, index: u8) -> &mut u32 {
        &mut self.registers[index as usize]
    }

    fn skip(&mut self, imm: u16) {
        self.pc = (self.pc as i32 + ((imm as i16 as i32) << 2)) as u32;
    }

    fn jump(&mut self, bits: u32) {
        self.pc = (self.pc & 0xFC000000) | (bits << 2);
    }

    pub fn step(&mut self) -> Result<()> {
        let instruction = self.memory.get_u32(self.pc)?;

        self.pc += 4;

        self.dispatch(instruction)
            .unwrap_or(Err(CpuInvalid(instruction)))
    }
}

impl<Mem: Memory> Decoder<Result<()>> for State<Mem> {
    fn add(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        if let Some(value) = self.register(s).checked_add(*self.register(t)) {
            *self.register(d) = value;

            Ok(())
        } else {
            return self.trap()
        }
    }

    fn addu(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        *self.register(d) = *self.register(s) + *self.register(t);

        Ok(())
    }

    fn and(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        *self.register(d) = *self.register(s) & *self.register(t);

        Ok(())
    }

    fn div(&mut self, s: u8, t: u8) -> Result<()> {
        let (a, b) = (*self.register(s) as i32, *self.register(t) as i32);
        let (lo, hi) = if b != 0 { (a / b, a % b) } else { (0i32, 0i32) };

        (self.lo, self.hi) = (lo as u32, hi as u32);

        Ok(())
    }

    fn divu(&mut self, s: u8, t: u8) -> Result<()> {
        let (a, b) = (*self.register(s), *self.register(t));

        (self.lo, self.hi) = if b != 0 { (a / b, a % b) } else { (0u32, 0u32) };

        Ok(())
    }

    fn mult(&mut self, s: u8, t: u8) -> Result<()> {
        let (a, b) = (*self.register(s) as i64, *self.register(t) as i64);
        let value = (a * b) as u64;

        (self.lo, self.hi) = ((value & 0xFFFFFFFF) as u32, (value >> 32) as u32);

        Ok(())
    }

    fn multu(&mut self, s: u8, t: u8) -> Result<()> {
        let (a, b) = (*self.register(s) as u64, *self.register(t) as u64);
        let value = a * b;

        (self.lo, self.hi) = ((value & 0xFFFFFFFF) as u32, (value >> 32) as u32);

        Ok(())
    }

    fn nor(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        *self.register(d) = !(*self.register(s) | *self.register(t));

        Ok(())
    }

    fn or(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        *self.register(d) = *self.register(s) | *self.register(t);

        Ok(())
    }

    fn sll(&mut self, t: u8, d: u8, sham: u8) -> Result<()> {
        *self.register(d) = *self.register(t) << sham;

        Ok(())
    }

    fn sllv(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        *self.register(d) = *self.register(t) << *self.register(s);

        Ok(())
    }

    fn sra(&mut self, t: u8, d: u8, sham: u8) -> Result<()> {
        let source = *self.register(t) as i32;

        *self.register(d) = (source >> (sham as i32)) as u32;

        Ok(())
    }

    fn srav(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let source = *self.register(t) as i32;

        *self.register(d) = (source >> (*self.register(s) as i32)) as u32;

        Ok(())
    }

    fn srl(&mut self, t: u8, d: u8, sham: u8) -> Result<()> {
        *self.register(d) = *self.register(t) >> sham;

        Ok(())
    }

    fn srlv(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        *self.register(d) = *self.register(t) >> *self.register(s);

        Ok(())
    }

    fn sub(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        if let Some(value) = self.register(s).checked_sub(*self.register(t)) {
            *self.register(d) = value;

            Ok(())
        } else {
            self.trap()
        }
    }

    fn subu(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        *self.register(d) = *self.register(s) - *self.register(t);

        Ok(())
    }

    fn xor(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        *self.register(d) = *self.register(s) ^ *self.register(t);

        Ok(())
    }

    fn slt(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let value = (*self.register(s) as i32) < (*self.register(t) as i32);

        *self.register(d) = value as u32;

        Ok(())
    }

    fn sltu(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let value = *self.register(s) < *self.register(t);

        *self.register(d) = value as u32;

        Ok(())
    }

    fn jr(&mut self, s: u8) -> Result<()> {
        self.pc = *self.register(s);

        Ok(())
    }

    fn jalr(&mut self, s: u8) -> Result<()> {
        *self.register(31) = self.pc;

        self.pc = *self.register(s);

        Ok(())
    }

    fn addi(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let imm = imm as i16 as i32;

        if let Some(value) = (*self.register(s) as i32).checked_add(imm) {
            *self.register(t) = value as u32;

            Ok(())
        } else {
            self.trap()
        }
    }

    fn addiu(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let imm = imm as i16 as i32;

        *self.register(t) = ((*self.register(s) as i32) + imm) as u32;

        Ok(())
    }

    fn andi(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        *self.register(t) = *self.register(s) & (imm as u32);

        Ok(())
    }

    fn ori(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        *self.register(t) = *self.register(s) | (imm as u32);

        Ok(())
    }

    fn xori(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        *self.register(t) = *self.register(s) ^ (imm as u32);

        Ok(())
    }

    fn lui(&mut self, t: u8, imm: u16) -> Result<()> {
        *self.register(t) = (imm as u32) << 16;

        Ok(())
    }

    fn lhi(&mut self, t: u8, imm: u16) -> Result<()> {
        let value = (*self.register(t) & 0x0000FFFF) | ((imm as u32) << 16);

        *self.register(t) = value;

        Ok(())
    }

    fn llo(&mut self, t: u8, imm: u16) -> Result<()> {
        let value = (*self.register(t) & 0xFFFF) | (imm as u32);

        *self.register(t) = value;

        Ok(())
    }

    fn slti(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let value = (*self.register(s) as i32) < (imm as i16 as i32);

        *self.register(t) = value as u32;

        Ok(())
    }

    fn sltiu(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let value = (*self.register(s) as u32) < (imm as u32);

        *self.register(t) = value as u32;

        Ok(())
    }

    fn beq(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        if *self.register(s) == *self.register(t) {
            self.skip(imm);
        }

        Ok(())
    }

    fn bne(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        if *self.register(s) != *self.register(t) {
            self.skip(imm);
        }

        Ok(())
    }

    fn bgtz(&mut self, s: u8, imm: u16) -> Result<()> {
        if (*self.register(s) as i32) > 0 {
            self.skip(imm);
        }

        Ok(())
    }

    fn blez(&mut self, s: u8, imm: u16) -> Result<()> {
        if (*self.register(s) as i32) <= 0 {
            self.skip(imm);
        }

        Ok(())
    }

    fn bltz(&mut self, s: u8, imm: u16) -> Result<()> {
        if (*self.register(s) as i32) < 0 {
            self.skip(imm);
        }

        Ok(())
    }

    fn bgez(&mut self, s: u8, imm: u16) -> Result<()> {
        if (*self.register(s) as i32) >= 0 {
            self.skip(imm);
        }

        Ok(())
    }

    fn bltzal(&mut self, s: u8, imm: u16) -> Result<()> {
        if (*self.register(s) as i32) < 0 {
            *self.register(31) = self.pc;

            self.skip(imm);
        }

        Ok(())
    }

    fn bgezal(&mut self, s: u8, imm: u16) -> Result<()> {
        if (*self.register(s) as i32) >= 0 {
            *self.register(31) = self.pc;

            self.skip(imm);
        }

        Ok(())
    }

    fn j(&mut self, imm: u32) -> Result<()> {
        self.jump(imm);

        Ok(())
    }

    fn jal(&mut self, imm: u32) -> Result<()> {
        *self.register(31) = self.pc;

        self.jump(imm);

        Ok(())
    }

    fn lb(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);

        *self.register(t) = self.memory.get(address as u32)? as i8 as i32 as u32;

        Ok(())
    }

    fn lbu(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);

        *self.register(t) = self.memory.get(address as u32)? as u32;

        Ok(())
    }

    fn lh(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);

        *self.register(t) = self.memory.get_u16(address as u32)? as i16 as i32 as u32;

        Ok(())
    }

    fn lhu(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);

        *self.register(t) = self.memory.get_u16(address as u32)? as u32;

        Ok(())
    }

    fn lw(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);

        *self.register(t) = self.memory.get_u32(address as u32)?;

        Ok(())
    }

    fn sb(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);
        let value = *self.register(t) as u8;

        self.memory.set(address as u32, value)?;

        Ok(())
    }

    fn sh(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);
        let value = *self.register(t) as u16;

        self.memory.set_u16(address as u32, value)?;

        Ok(())
    }

    fn sw(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32) + (imm as i16 as i32);
        let value = *self.register(t);

        self.memory.set_u32(address as u32, value)?;

        Ok(())
    }

    fn mfhi(&mut self, d: u8) -> Result<()> {
        *self.register(d) = self.hi;

        Ok(())
    }

    fn mflo(&mut self, d: u8) -> Result<()> {
        *self.register(d) = self.lo;

        Ok(())
    }

    fn mthi(&mut self, s: u8) -> Result<()> {
        self.hi = *self.register(s);

        Ok(())
    }

    fn mtlo(&mut self, s: u8) -> Result<()> {
        self.lo = *self.register(s);

        Ok(())
    }

    fn trap(&mut self) -> Result<()> {
        Err(CpuTrap)
    }
}
