use crate::cpu::decoder::Decoder;
use crate::cpu::error::Error::{CpuInvalid, CpuSyscall, CpuTrap};
use crate::cpu::error::Result;
use crate::cpu::registers::WhichRegister::{Cf, Fp, Hi, Line, Lo, Pc};
use crate::cpu::{Memory, Registers, State};

impl<Mem: Memory, Reg: Registers> State<Mem, Reg> {
    fn hilo(&self) -> u64 {
        (self.registers.get(Hi) as u64).wrapping_shl(32) | (self.registers.get(Lo) as u64)
    }

    fn load_hilo_or_trap(&mut self, result: Option<u64>) -> Result<()> {
        if let Some(result) = result {
            self.registers.set(Hi, result.wrapping_shr(32) as u32);
            self.registers.set(Lo, result as u32);

            Ok(())
        } else {
            self.trap()
        }
    }
}

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

    fn fp(&mut self, index: u8) -> u32 {
        self.registers.get(Fp(index))
    }

    fn set_fp(&mut self, index: u8, value: u32) {
        self.registers.set(Fp(index), value);
    }

    fn skip(&mut self, imm: u16) {
        // ((pc + 4) as i32 + ((imm as i16 as i32) << 2)) as u32
        let offset = (imm as i16 as i32).wrapping_shl(2);
        let destination = (self.registers.get(Pc) as i32).wrapping_add(offset);

        self.registers.set(Pc, destination as u32)
    }

    fn jump(&mut self, bits: u32) {
        self.registers.set(
            Pc,
            (self.registers.get(Pc) & 0xFC000000) | bits.wrapping_shl(2),
        );
    }

    pub fn step(&mut self) -> Result<()> {
        let start = self.registers.get(Pc);
        let instruction = self.memory.get_u32(start)?;

        self.registers.step_pc();

        self.dispatch(instruction)
            .unwrap_or(Err(CpuInvalid(instruction)))
            .inspect_err(|_| self.registers.set(Pc, start)) // if error, keep pc here
    }
}

impl<Mem: Memory, Reg: Registers> Decoder<Result<()>> for State<Mem, Reg> {
    fn add(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let (a, b) = (self.reg(s) as i32, self.reg(t) as i32);

        if let Some(value) = a.checked_add(b) {
            self.set_reg(d, value as u32);

            Ok(())
        } else {
            self.trap()
        }
    }

    fn addu(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let value = self.reg(s).wrapping_add(self.reg(t));
        self.set_reg(d, value);

        Ok(())
    }

    fn and(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let value = self.reg(s) & self.reg(t);
        self.set_reg(d, value);

        Ok(())
    }

    fn div(&mut self, s: u8, t: u8) -> Result<()> {
        let (a, b) = (self.reg(s) as i32, self.reg(t) as i32);
        let (lo, hi) = if b != 0 {
            (a.wrapping_div(b), a % b)
        } else {
            return self.trap();
        };

        self.registers.set(Lo, lo as u32);
        self.registers.set(Hi, hi as u32);

        Ok(())
    }

    fn divu(&mut self, s: u8, t: u8) -> Result<()> {
        let (a, b) = (self.reg(s), self.reg(t));

        if b != 0 {
            self.registers.set(Lo, a.wrapping_div(b));
            self.registers.set(Hi, a % b);

            Ok(())
        } else {
            self.trap()
        }
    }

    fn mult(&mut self, s: u8, t: u8) -> Result<()> {
        let (a, b) = (self.reg(s) as i64, self.reg(t) as i64);
        let value = (a * b) as u64;

        self.registers.set(Lo, value as u32);
        self.registers.set(Hi, value.wrapping_shr(32) as u32);

        Ok(())
    }

    fn multu(&mut self, s: u8, t: u8) -> Result<()> {
        let (a, b) = (self.reg(s) as u64, self.reg(t) as u64);
        let value = a * b;

        self.registers.set(Lo, value as u32);
        self.registers.set(Hi, value.wrapping_shr(32) as u32);

        Ok(())
    }

    fn nor(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let value = !(self.reg(s) | self.reg(t));
        self.set_reg(d, value);

        Ok(())
    }

    fn or(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let value = self.reg(s) | self.reg(t);
        self.set_reg(d, value);

        Ok(())
    }

    fn sll(&mut self, t: u8, d: u8, sham: u8) -> Result<()> {
        let value = self.reg(t).wrapping_shl(sham as u32);
        self.set_reg(d, value);

        Ok(())
    }

    fn sllv(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let value = self.reg(t).wrapping_shl(self.reg(s));
        self.set_reg(d, value);

        Ok(())
    }

    fn sra(&mut self, t: u8, d: u8, sham: u8) -> Result<()> {
        let source = self.reg(t) as i32;

        let value = source.wrapping_shr(sham as u32) as u32;
        self.set_reg(d, value);

        Ok(())
    }

    fn srav(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let source = self.reg(t) as i32;

        let value = source.wrapping_shr(self.reg(s)) as u32;
        self.set_reg(d, value);

        Ok(())
    }

    fn srl(&mut self, t: u8, d: u8, sham: u8) -> Result<()> {
        let value = self.reg(t).wrapping_shr(sham as u32);
        self.set_reg(d, value);

        Ok(())
    }

    fn srlv(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let value = self.reg(t).wrapping_shr(self.reg(s));
        self.set_reg(d, value);

        Ok(())
    }

    fn sub(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let (a, b) = (self.reg(s) as i32, self.reg(t) as i32);

        if let Some(value) = a.checked_sub(b) {
            let result = value as u32;
            self.set_reg(d, result);

            Ok(())
        } else {
            self.trap()
        }
    }

    fn subu(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let value = self.reg(s).wrapping_sub(self.reg(t));
        self.set_reg(d, value);

        Ok(())
    }

    fn xor(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let value = self.reg(s) ^ self.reg(t);
        self.set_reg(d, value);

        Ok(())
    }

    fn slt(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let value = (self.reg(s) as i32) < (self.reg(t) as i32);

        self.set_reg(d, value as u32);

        Ok(())
    }

    fn sltu(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let value = self.reg(s) < self.reg(t);

        let result = value as u32;
        self.set_reg(d, result);

        Ok(())
    }

    fn jr(&mut self, s: u8) -> Result<()> {
        let value = self.reg(s);
        self.registers.set(Pc, value);

        Ok(())
    }

    fn jalr(&mut self, s: u8) -> Result<()> {
        self.set_reg(31, self.registers.get(Pc));

        let value = self.reg(s);
        self.registers.set(Pc, value);

        Ok(())
    }

    fn madd(&mut self, s: u8, t: u8) -> Result<()> {
        let a = self.reg(s) as i32 as i64;
        let b = self.reg(t) as i32 as i64;

        let result = a
            .checked_mul(b)
            .and_then(|ab| ab.checked_add(self.hilo() as i64))
            .map(|result| result as u64);

        self.load_hilo_or_trap(result)
    }

    fn maddu(&mut self, s: u8, t: u8) -> Result<()> {
        let a = self.reg(s) as u64;
        let b = self.reg(t) as u64;
        let result = a.wrapping_mul(b).wrapping_add(self.hilo());

        self.registers.set(Hi, result.wrapping_shr(32) as u32);
        self.registers.set(Lo, result as u32);

        Ok(())
    }

    fn mul(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let (a, b) = (self.reg(s) as i32, self.reg(t) as i32);

        let value = a.wrapping_mul(b);

        self.set_reg(d, value as u32);

        Ok(())
    }

    fn msub(&mut self, s: u8, t: u8) -> Result<()> {
        let a = self.reg(s) as i32 as i64;
        let b = self.reg(t) as i32 as i64;

        let result = a
            .checked_mul(b)
            .and_then(|ab| (self.hilo() as i64).checked_sub(ab))
            .map(|result| result as u64);

        self.load_hilo_or_trap(result)
    }

    fn msubu(&mut self, s: u8, t: u8) -> Result<()> {
        let a = self.reg(s) as u64;
        let b = self.reg(t) as u64;
        let result = self.hilo().wrapping_sub(a.wrapping_mul(b));

        self.registers.set(Hi, result.wrapping_shr(32) as u32);
        self.registers.set(Lo, result as u32);

        Ok(())
    }

    fn addi(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let imm = imm as i16 as i32;
        let a = self.reg(s) as i32;

        if let Some(value) = a.checked_add(imm) {
            self.set_reg(t, value as u32);

            Ok(())
        } else {
            self.trap()
        }
    }

    fn addiu(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let imm = imm as i16 as i32;
        let a = self.reg(s) as i32;

        self.set_reg(t, a.wrapping_add(imm) as u32);

        Ok(())
    }

    fn andi(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let value = self.reg(s) & (imm as u32);
        self.set_reg(t, value);

        Ok(())
    }

    fn ori(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let value = self.reg(s) | (imm as u32);
        self.set_reg(t, value);

        Ok(())
    }

    fn xori(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let value = self.reg(s) ^ (imm as u32);
        self.set_reg(t, value);

        Ok(())
    }

    fn lui(&mut self, t: u8, imm: u16) -> Result<()> {
        let value = (imm as u32).wrapping_shl(16);
        self.set_reg(t, value);

        Ok(())
    }

    fn lhi(&mut self, t: u8, imm: u16) -> Result<()> {
        let value = (self.reg(t) & 0x0000FFFF) | ((imm as u32).wrapping_shl(16));

        self.set_reg(t, value);

        Ok(())
    }

    fn llo(&mut self, t: u8, imm: u16) -> Result<()> {
        let value = (self.reg(t) & 0xFFFF) | (imm as u32);

        self.set_reg(t, value);

        Ok(())
    }

    fn slti(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let value = (self.reg(s) as i32) < (imm as i16 as i32);

        self.set_reg(t, value as u32);

        Ok(())
    }

    fn sltiu(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let value = self.reg(s) < (imm as u32);

        self.set_reg(t, value as u32);

        Ok(())
    }

    fn beq(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        if self.reg(s) == self.reg(t) {
            self.skip(imm);
        }

        Ok(())
    }

    fn bne(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        if self.reg(s) != self.reg(t) {
            self.skip(imm);
        }

        Ok(())
    }

    fn bgtz(&mut self, s: u8, imm: u16) -> Result<()> {
        if (self.reg(s) as i32) > 0 {
            self.skip(imm);
        }

        Ok(())
    }

    fn blez(&mut self, s: u8, imm: u16) -> Result<()> {
        if (self.reg(s) as i32) <= 0 {
            self.skip(imm);
        }

        Ok(())
    }

    fn bltz(&mut self, s: u8, imm: u16) -> Result<()> {
        if (self.reg(s) as i32) < 0 {
            self.skip(imm);
        }

        Ok(())
    }

    fn bgez(&mut self, s: u8, imm: u16) -> Result<()> {
        if (self.reg(s) as i32) >= 0 {
            self.skip(imm);
        }

        Ok(())
    }

    fn bltzal(&mut self, s: u8, imm: u16) -> Result<()> {
        if (self.reg(s) as i32) < 0 {
            self.set_reg(31, self.registers.get(Pc));

            self.skip(imm);
        }

        Ok(())
    }

    fn bgezal(&mut self, s: u8, imm: u16) -> Result<()> {
        if (self.reg(s) as i32) >= 0 {
            self.set_reg(31, self.registers.get(Pc));

            self.skip(imm);
        }

        Ok(())
    }

    fn j(&mut self, imm: u32) -> Result<()> {
        self.jump(imm);

        Ok(())
    }

    fn jal(&mut self, imm: u32) -> Result<()> {
        self.set_reg(31, self.registers.get(Pc));

        self.jump(imm);

        Ok(())
    }

    fn lb(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (self.reg(s) as i32).wrapping_add(imm as i16 as i32);

        self.set_reg(t, self.memory.get(address as u32)? as i8 as i32 as u32);

        Ok(())
    }

    fn lbu(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (self.reg(s) as i32).wrapping_add(imm as i16 as i32);

        self.set_reg(t, self.memory.get(address as u32)? as u32);

        Ok(())
    }

    fn lh(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (self.reg(s) as i32).wrapping_add(imm as i16 as i32);

        self.set_reg(t, self.memory.get_u16(address as u32)? as i16 as i32 as u32);

        Ok(())
    }

    fn lhu(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (self.reg(s) as i32).wrapping_add(imm as i16 as i32);

        self.set_reg(t, self.memory.get_u16(address as u32)? as u32);

        Ok(())
    }

    fn lw(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (self.reg(s) as i32).wrapping_add(imm as i16 as i32);

        self.set_reg(t, self.memory.get_u32(address as u32)?);

        Ok(())
    }

    fn sb(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (self.reg(s) as i32).wrapping_add(imm as i16 as i32);
        let value = self.reg(t) as u8;

        self.memory.set(address as u32, value)?;

        Ok(())
    }

    fn sh(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (self.reg(s) as i32).wrapping_add(imm as i16 as i32);
        let value = self.reg(t) as u16;

        self.memory.set_u16(address as u32, value)?;

        Ok(())
    }

    fn sw(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (self.reg(s) as i32).wrapping_add(imm as i16 as i32);
        let value = self.reg(t);

        self.memory.set_u32(address as u32, value)?;

        Ok(())
    }

    fn mfhi(&mut self, d: u8) -> Result<()> {
        self.set_reg(d, self.registers.get(Hi));

        Ok(())
    }

    fn mflo(&mut self, d: u8) -> Result<()> {
        self.set_reg(d, self.registers.get(Lo));

        Ok(())
    }

    fn mthi(&mut self, s: u8) -> Result<()> {
        let value = self.reg(s);
        self.registers.set(Hi, value);

        Ok(())
    }

    fn mtlo(&mut self, s: u8) -> Result<()> {
        let value = self.reg(s);
        self.registers.set(Lo, value);

        Ok(())
    }

    fn trap(&mut self) -> Result<()> {
        Err(CpuTrap)
    }

    fn syscall(&mut self) -> Result<()> {
        Err(CpuSyscall)
    }

    fn add_s(&mut self, t: u8, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(self.fp(s));
        let b = f32::from_bits(self.fp(t));

        self.set_fp(d, (a + b).to_bits());

        Ok(())
    }
    fn sub_s(&mut self, t: u8, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(self.fp(s));
        let b = f32::from_bits(self.fp(t));

        self.set_fp(d, (a - b).to_bits());

        Ok(())
    }
    fn mul_s(&mut self, t: u8, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(self.fp(s));
        let b = f32::from_bits(self.fp(t));

        self.set_fp(d, (a * b).to_bits());

        Ok(())
    }
    fn div_s(&mut self, t: u8, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(self.fp(s));
        let b = f32::from_bits(self.fp(t));

        self.set_fp(d, (a / b).to_bits());

        Ok(())
    }
    fn sqrt_s(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(self.fp(s));

        self.set_fp(d, a.sqrt().to_bits());

        Ok(())
    }
    fn abs_s(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(self.fp(s));

        self.set_fp(d, a.abs().to_bits());

        Ok(())
    }
    fn neg_s(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(self.fp(s));

        self.set_fp(d, (-a).to_bits());

        Ok(())
    }
    fn floor_w_s(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(self.fp(s));

        self.set_fp(d, u32::from_le_bytes((a.floor() as i32).to_le_bytes()));

        Ok(())
    }
    fn ceil_w_s(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(self.fp(s));

        self.set_fp(d, u32::from_le_bytes((a.ceil() as i32).to_le_bytes()));

        Ok(())
    }
    fn round_w_s(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(self.fp(s));

        self.set_fp(d, u32::from_le_bytes((a.round() as i32).to_le_bytes()));

        Ok(())
    }
    fn trunc_w_s(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(self.fp(s));

        self.set_fp(d, u32::from_le_bytes((a.trunc() as i32).to_le_bytes()));
        Ok(())
    }
    fn add_d(&mut self, t: u8, s: u8, d: u8) -> Result<()> {
        let a = f64::from_bits(self.fp(s) as u64 | ((self.fp(s + 1) as u64) << 32));
        let b = f64::from_bits(self.fp(t) as u64 | ((self.fp(t + 1) as u64) << 32));

        let result = (a + b).to_bits();
        let lower = result as u32;
        let upper = (result >> 32) as u32;
        self.set_fp(d, lower);
        self.set_fp(d + 1, upper);
        Ok(())
    }
    fn sub_d(&mut self, t: u8, s: u8, d: u8) -> Result<()> {
        let a = f64::from_bits(self.fp(s) as u64 | ((self.fp(s + 1) as u64) << 32));
        let b = f64::from_bits(self.fp(t) as u64 | ((self.fp(t + 1) as u64) << 32));

        let result = (a - b).to_bits();
        let lower = result as u32;
        let upper = (result >> 32) as u32;
        self.set_fp(d, lower);
        self.set_fp(d + 1, upper);
        Ok(())
    }
    fn mul_d(&mut self, t: u8, s: u8, d: u8) -> Result<()> {
        let a = f64::from_bits(self.fp(s) as u64 | ((self.fp(s + 1) as u64) << 32));
        let b = f64::from_bits(self.fp(t) as u64 | ((self.fp(t + 1) as u64) << 32));
        let result = (a * b).to_bits();
        let lower = result as u32;
        let upper = (result >> 32) as u32;
        self.set_fp(d, lower);
        self.set_fp(d + 1, upper);
        Ok(())
    }
    fn div_d(&mut self, t: u8, s: u8, d: u8) -> Result<()> {
        let a = f64::from_bits(self.fp(s) as u64 | ((self.fp(s + 1) as u64) << 32));
        let b = f64::from_bits(self.fp(t) as u64 | ((self.fp(t + 1) as u64) << 32));
        let result = (a / b).to_bits();
        let lower = result as u32;
        let upper = (result >> 32) as u32;
        self.set_fp(d, lower);
        self.set_fp(d + 1, upper);
        Ok(())
    }
    fn sqrt_d(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f64::from_bits(self.fp(s) as u64 | ((self.fp(s + 1) as u64) << 32));
        let result = a.sqrt().to_bits();
        let lower = result as u32;
        let upper = (result >> 32) as u32;
        self.set_fp(d, lower);
        self.set_fp(d + 1, upper);
        Ok(())
    }
    fn abs_d(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f64::from_bits(self.fp(s) as u64 | ((self.fp(s + 1) as u64) << 32));
        let result = a.abs().to_bits();
        let lower = result as u32;
        let upper = (result >> 32) as u32;
        self.set_fp(d, lower);
        self.set_fp(d + 1, upper);
        Ok(())
    }
    fn neg_d(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f64::from_bits(self.fp(s) as u64 | ((self.fp(s + 1) as u64) << 32));
        let result = (-a).to_bits();
        let lower = result as u32;
        let upper = (result >> 32) as u32;
        self.set_fp(d, lower);
        self.set_fp(d + 1, upper);
        Ok(())
    }
    fn floor_w_d(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f64::from_bits(self.fp(s) as u64 | ((self.fp(s + 1) as u64) << 32));
        let val = u64::from_le_bytes((a.floor() as i64).to_le_bytes());
        self.set_fp(d, val as u32);
        self.set_fp(d + 1, (val >> 32) as u32);
        Ok(())
    }
    fn ceil_w_d(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f64::from_bits(self.fp(s) as u64 | ((self.fp(s + 1) as u64) << 32));
        let val = u64::from_le_bytes((a.ceil() as i64).to_le_bytes());
        self.set_fp(d, val as u32);
        self.set_fp(d + 1, (val >> 32) as u32);
        Ok(())
    }
    fn round_w_d(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f64::from_bits(self.fp(s) as u64 | ((self.fp(s + 1) as u64) << 32));
        let val = u64::from_le_bytes((a.round() as i64).to_le_bytes());
        self.set_fp(d, val as u32);
        self.set_fp(d + 1, (val >> 32) as u32);
        Ok(())
    }
    fn trunc_w_d(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f64::from_bits(self.fp(s) as u64 | ((self.fp(s + 1) as u64) << 32));
        let val = u64::from_le_bytes((a.trunc() as i64).to_le_bytes());
        self.set_fp(d, val as u32);
        self.set_fp(d + 1, (val >> 32) as u32);
        Ok(())
    }
    fn c_eq_s(&mut self, t: u8, s: u8, cc: u8) -> Result<()> {
        let a = f32::from_bits(self.fp(s));
        let b = f32::from_bits(self.fp(t));
        let value = a == b;
        let bit = 1 << cc;
        let cf = self.registers.get(Cf);
        self.registers.set(Cf, (cf & !bit) | (value as u32) << cc);
        Ok(())
    }
    fn c_le_s(&mut self, t: u8, s: u8, cc: u8) -> Result<()> {
        let a = f32::from_bits(self.fp(s));
        let b = f32::from_bits(self.fp(t));
        let value = a <= b;
        let bit = 1 << cc;
        let cf = self.registers.get(Cf);
        self.registers.set(Cf, (cf & !bit) | (value as u32) << cc);
        Ok(())
    }
    fn c_lt_s(&mut self, t: u8, s: u8, cc: u8) -> Result<()> {
        let a = f32::from_bits(self.fp(s));
        let b = f32::from_bits(self.fp(t));
        let value = a < b;
        let bit = 1 << cc;
        let cf = self.registers.get(Cf);
        self.registers.set(Cf, (cf & !bit) | (value as u32) << cc);
        Ok(())
    }
    fn c_eq_d(&mut self, t: u8, s: u8, cc: u8) -> Result<()> {
        let a = f64::from_bits(self.fp(s) as u64 | (self.fp(s + 1) as u64) << 32);
        let b = f64::from_bits(self.fp(t) as u64 | (self.fp(t + 1) as u64) << 32);
        let value = a == b;
        let bit = 1 << cc;
        let cf = self.registers.get(Cf);
        self.registers.set(Cf, (cf & !bit) | (value as u32) << cc);
        Ok(())
    }
    fn c_le_d(&mut self, t: u8, s: u8, cc: u8) -> Result<()> {
        let a = f64::from_bits(self.fp(s) as u64 | (self.fp(s + 1) as u64) << 32);
        let b = f64::from_bits(self.fp(t) as u64 | (self.fp(t + 1) as u64) << 32);
        let value = a <= b;
        let bit = 1 << cc;
        let cf = self.registers.get(Cf);
        self.registers.set(Cf, (cf & !bit) | (value as u32) << cc);
        Ok(())
    }
    fn c_lt_d(&mut self, t: u8, s: u8, cc: u8) -> Result<()> {
        let a = f64::from_bits(self.fp(s) as u64 | (self.fp(s + 1) as u64) << 32);
        let b = f64::from_bits(self.fp(t) as u64 | (self.fp(t + 1) as u64) << 32);
        let value = a < b;
        let bit = 1 << cc;
        let cf = self.registers.get(Cf);
        self.registers.set(Cf, (cf & !bit) | (value as u32) << cc);
        Ok(())
    }
    fn bc1t(&mut self, cc: u8, addr: u16) -> Result<()> {
        let bit = 1 << cc;
        if (self.registers.get(Cf) & bit) != 0 {
            self.skip(addr);
        }
        Ok(())
    }
    fn bc1f(&mut self, cc: u8, addr: u16) -> Result<()> {
        let bit = 1 << cc;
        if (self.registers.get(Cf) & bit) == 0 {
            self.skip(addr);
        }
        Ok(())
    }
    fn mov_s(&mut self, s: u8, d: u8) -> Result<()> {
        let value = self.fp(s);
        self.set_fp(d, value);
        Ok(())
    }
    fn movf_s(&mut self, cc: u8, s: u8, d: u8) -> Result<()> {
        let bit = 1 << cc;
        if (self.registers.get(Cf) & bit) == 0 {
            return self.mov_s(s, d);
        }
        Ok(())
    }
    fn movt_s(&mut self, cc: u8, s: u8, d: u8) -> Result<()> {
        let bit = 1 << cc;
        if (self.registers.get(Cf) & bit) == 0 {
            return self.mov_s(s, d);
        }
        Ok(())
    }
    fn movn_s(&mut self, t: u8, s: u8, d: u8) -> Result<()> {
        let value = f32::from_bits(self.fp(t));
        if value != 0.0 {
            return self.mov_s(s, d);
        }
        Ok(())
    }
    fn movz_s(&mut self, t: u8, s: u8, d: u8) -> Result<()> {
        let value = f32::from_bits(self.fp(t));
        if value == 0.0 {
            return self.mov_s(s, d);
        }
        Ok(())
    }
    fn mov_d(&mut self, s: u8, d: u8) -> Result<()> {
        let value = self.fp(s);
        self.set_fp(d, value);
        let value = self.fp(s + 1);
        self.set_fp(d + 1, value);
        Ok(())
    }
    fn movf_d(&mut self, cc: u8, s: u8, d: u8) -> Result<()> {
        let bit = 1 << cc;
        if (self.registers.get(Cf) & bit) == 0 {
            return self.mov_d(s, d);
        }
        Ok(())
    }
    fn movt_d(&mut self, cc: u8, s: u8, d: u8) -> Result<()> {
        if (self.registers.get(Cf) & (1 << cc)) == 0 {
            return self.mov_d(s, d);
        }
        Ok(())
    }
    fn movn_d(&mut self, t: u8, s: u8, d: u8) -> Result<()> {
        let value = f64::from_bits((self.fp(t) as u64) | (self.fp(t + 1) as u64) << 32);
        if value != 0.0 {
            return self.mov_d(s, d);
        }
        Ok(())
    }
    fn movz_d(&mut self, t: u8, s: u8, d: u8) -> Result<()> {
        let value = f64::from_bits((self.fp(t) as u64) | (self.fp(t + 1) as u64) << 32);
        if value == 0.0 {
            return self.mov_d(s, d);
        }
        Ok(())
    }
    fn movf(&mut self, s: u8, cc: u8, d: u8) -> Result<()> {
        let bit = 1 << cc;
        if (self.registers.get(Cf) & bit) == 0 {
            let value = self.reg(s);
            self.set_reg(d, value);
        }
        Ok(())
    }
    fn movt(&mut self, s: u8, cc: u8, d: u8) -> Result<()> {
        let bit = 1 << cc;
        if (self.registers.get(Cf) & bit) == 0 {
            let value = self.reg(s);
            self.set_reg(d, value);
        }
        Ok(())
    }
    fn movn(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let value = f32::from_bits(self.fp(t));
        if value != 0.0 {
            let value = self.reg(s);
            self.set_reg(d, value);
        }
        Ok(())
    }
    fn movz(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let value = f32::from_bits(self.fp(t));
        if value == 0.0 {
            let value = self.reg(s);
            self.set_reg(d, value);
        }
        Ok(())
    }
    fn cvt_s_w(&mut self, s: u8, d: u8) -> Result<()> {
        let value = self.fp(s);
        self.set_fp(d, (value as f32).to_bits());
        Ok(())
    }
    fn cvt_w_s(&mut self, s: u8, d: u8) -> Result<()> {
        let value = f32::from_bits(self.fp(s));
        self.set_fp(d, value as i32 as u32);
        Ok(())
    }
    fn cvt_s_d(&mut self, s: u8, d: u8) -> Result<()> {
        let value = f64::from_bits((self.fp(s) as u64) | (self.fp(s + 1) as u64) << 32);
        self.set_fp(d, (value as f32).to_bits());
        Ok(())
    }
    fn cvt_d_s(&mut self, s: u8, d: u8) -> Result<()> {
        let value = f32::from_bits(self.fp(s));
        let double_cast = value as f64;
        let result = double_cast.to_bits();
        let lower = result as u32;
        let upper = (result >> 32) as u32;
        self.set_fp(d, lower);
        self.set_fp(d + 1, upper);
        Ok(())
    }
    fn cvt_w_d(&mut self, s: u8, d: u8) -> Result<()> {
        let value = f64::from_bits((self.fp(s) as u64) | (self.fp(s + 1) as u64) << 32);
        self.set_fp(d, value as i32 as u32);
        Ok(())
    }
    fn cvt_d_w(&mut self, s: u8, d: u8) -> Result<()> {
        let value = self.fp(s);
        let double_cast = value as f64;
        let result = double_cast.to_bits();
        let lower = result as u32;
        let upper = (result >> 32) as u32;
        self.set_fp(d, lower);
        self.set_fp(d + 1, upper);
        Ok(())
    }
    fn mtc1(&mut self, t: u8, s: u8) -> Result<()> {
        let value = self.reg(s);
        self.set_fp(t, value);

        Ok(())
    }
    fn mfc1(&mut self, t: u8, s: u8) -> Result<()> {
        let value = self.fp(s);
        self.set_reg(t, value);

        Ok(())
    }
    fn ldc1(&mut self, base: u8, t: u8, offset: u16) -> Result<()> {
        let address = (self.reg(base) as i32).wrapping_add(offset as i16 as i32);
        self.set_fp(t, self.memory.get_u32(address as u32)?);
        self.set_fp(t + 1, self.memory.get_u32(address as u32 + 4)?);
        Ok(())
    }
    fn sdc1(&mut self, base: u8, t: u8, offset: u16) -> Result<()> {
        let address = (self.reg(base) as i32).wrapping_add(offset as i16 as i32);
        let value = self.fp(t);
        let value2 = self.fp(t + 1);
        self.memory.set_u32(address as u32, value)?;
        self.memory.set_u32(address as u32 + 4, value2)?;
        Ok(())
    }
    fn lwc1(&mut self, base: u8, t: u8, offset: u16) -> Result<()> {
        let address = (self.reg(base) as i32).wrapping_add(offset as i16 as i32);
        let value = self.memory.get_u32(address as u32)?;
        self.set_fp(t, value);
        Ok(())
    }
    fn swc1(&mut self, base: u8, t: u8, offset: u16) -> Result<()> {
        let address = (self.reg(base) as i32).wrapping_add(offset as i16 as i32);
        let value = self.fp(t);
        self.memory.set_u32(address as u32, value)?;
        Ok(())
    }
}
