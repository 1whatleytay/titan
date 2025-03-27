use crate::cpu::decoder::Decoder;
use crate::cpu::error::Error::{CpuInvalid, CpuSyscall, CpuTrap};
use crate::cpu::error::Result;
use crate::cpu::{Memory, State};

impl<T: Memory> State<T> {
    fn hilo(&self) -> u64 {
        (self.registers.hi as u64).wrapping_shl(32) | (self.registers.lo as u64)
    }

    fn load_hilo_or_trap(&mut self, result: Option<u64>) -> Result<()> {
        if let Some(result) = result {
            self.registers.hi = result.wrapping_shr(32) as u32;
            self.registers.lo = result as u32;

            Ok(())
        } else {
            self.trap()
        }
    }
}

impl<Mem: Memory> State<Mem> {
    fn register(&mut self, index: u8) -> &mut u32 {
        if index == 0 {
            self.zero = 0;

            &mut self.zero
        } else {
            &mut self.registers.line[index as usize]
        }
    }

    fn fp_register(&mut self, index: u8) -> &mut u32 {
        &mut self.registers.fp[index as usize]
    }

    fn skip(&mut self, imm: u16) {
        // ((pc + 4) as i32 + ((imm as i16 as i32) << 2)) as u32
        let offset = (imm as i16 as i32).wrapping_shl(2);
        let destination = (self.registers.pc as i32).wrapping_add(offset);

        self.registers.pc = destination as u32
    }

    fn jump(&mut self, bits: u32) {
        self.registers.pc = (self.registers.pc & 0xFC000000) | bits.wrapping_shl(2);
    }

    pub fn step(&mut self) -> Result<()> {
        let start = self.registers.pc;
        let instruction = self.memory.get_u32(self.registers.pc)?;

        self.registers.pc = start.wrapping_add(4);

        self.dispatch(instruction)
            .unwrap_or(Err(CpuInvalid(instruction)))
            .inspect_err(|_| self.registers.pc = start) // if error, keep pc here
    }
}

impl<Mem: Memory> Decoder<Result<()>> for State<Mem> {
    fn add(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let (a, b) = (*self.register(s) as i32, *self.register(t) as i32);

        if let Some(value) = a.checked_add(b) {
            *self.register(d) = value as u32;

            Ok(())
        } else {
            self.trap()
        }
    }

    fn addu(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        *self.register(d) = self.register(s).wrapping_add(*self.register(t));

        Ok(())
    }

    fn and(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        *self.register(d) = *self.register(s) & *self.register(t);

        Ok(())
    }

    fn div(&mut self, s: u8, t: u8) -> Result<()> {
        let (a, b) = (*self.register(s) as i32, *self.register(t) as i32);
        let (lo, hi) = if b != 0 {
            (a.wrapping_div(b), a % b)
        } else {
            return self.trap();
        };

        (self.registers.lo, self.registers.hi) = (lo as u32, hi as u32);

        Ok(())
    }

    fn divu(&mut self, s: u8, t: u8) -> Result<()> {
        let (a, b) = (*self.register(s), *self.register(t));

        if b != 0 {
            (self.registers.lo, self.registers.hi) = (a.wrapping_div(b), a % b);

            Ok(())
        } else {
            self.trap()
        }
    }

    fn mult(&mut self, s: u8, t: u8) -> Result<()> {
        let (a, b) = (*self.register(s) as i64, *self.register(t) as i64);
        let value = (a * b) as u64;

        (self.registers.lo, self.registers.hi) = (value as u32, value.wrapping_shr(32) as u32);

        Ok(())
    }

    fn multu(&mut self, s: u8, t: u8) -> Result<()> {
        let (a, b) = (*self.register(s) as u64, *self.register(t) as u64);
        let value = a * b;

        (self.registers.lo, self.registers.hi) = (value as u32, value.wrapping_shr(32) as u32);

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
        *self.register(d) = self.register(t).wrapping_shl(sham as u32);

        Ok(())
    }

    fn sllv(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        *self.register(d) = self.register(t).wrapping_shl(*self.register(s));

        Ok(())
    }

    fn sra(&mut self, t: u8, d: u8, sham: u8) -> Result<()> {
        let source = *self.register(t) as i32;

        *self.register(d) = source.wrapping_shr(sham as u32) as u32;

        Ok(())
    }

    fn srav(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let source = *self.register(t) as i32;

        *self.register(d) = source.wrapping_shr(*self.register(s)) as u32;

        Ok(())
    }

    fn srl(&mut self, t: u8, d: u8, sham: u8) -> Result<()> {
        *self.register(d) = self.register(t).wrapping_shr(sham as u32);

        Ok(())
    }

    fn srlv(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        *self.register(d) = self.register(t).wrapping_shr(*self.register(s));

        Ok(())
    }

    fn sub(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let (a, b) = (*self.register(s) as i32, *self.register(t) as i32);

        if let Some(value) = a.checked_sub(b) {
            *self.register(d) = value as u32;

            Ok(())
        } else {
            self.trap()
        }
    }

    fn subu(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        *self.register(d) = self.register(s).wrapping_sub(*self.register(t));

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
        self.registers.pc = *self.register(s);

        Ok(())
    }

    fn jalr(&mut self, s: u8) -> Result<()> {
        *self.register(31) = self.registers.pc;

        self.registers.pc = *self.register(s);

        Ok(())
    }

    fn madd(&mut self, s: u8, t: u8) -> Result<()> {
        let a = *self.register(s) as i32 as i64;
        let b = *self.register(t) as i32 as i64;

        let result = a
            .checked_mul(b)
            .and_then(|ab| ab.checked_add(self.hilo() as i64))
            .map(|result| result as u64);

        self.load_hilo_or_trap(result)
    }

    fn maddu(&mut self, s: u8, t: u8) -> Result<()> {
        let a = *self.register(s) as u64;
        let b = *self.register(t) as u64;
        let result = a.wrapping_mul(b).wrapping_add(self.hilo());

        self.registers.hi = result.wrapping_shr(32) as u32;
        self.registers.lo = result as u32;

        Ok(())
    }

    fn mul(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let (a, b) = (*self.register(s) as i32, *self.register(t) as i32);

        let value = a.wrapping_mul(b);

        *self.register(d) = value as u32;

        Ok(())
    }

    fn msub(&mut self, s: u8, t: u8) -> Result<()> {
        let a = *self.register(s) as i32 as i64;
        let b = *self.register(t) as i32 as i64;

        let result = a
            .checked_mul(b)
            .and_then(|ab| (self.hilo() as i64).checked_sub(ab))
            .map(|result| result as u64);

        self.load_hilo_or_trap(result)
    }

    fn msubu(&mut self, s: u8, t: u8) -> Result<()> {
        let a = *self.register(s) as u64;
        let b = *self.register(t) as u64;
        let result = self.hilo().wrapping_sub(a.wrapping_mul(b));

        self.registers.hi = result.wrapping_shr(32) as u32;
        self.registers.lo = result as u32;

        Ok(())
    }

    fn addi(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let imm = imm as i16 as i32;
        let a = *self.register(s) as i32;

        if let Some(value) = a.checked_add(imm) {
            *self.register(t) = value as u32;

            Ok(())
        } else {
            self.trap()
        }
    }

    fn addiu(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let imm = imm as i16 as i32;
        let a = *self.register(s) as i32;

        *self.register(t) = a.wrapping_add(imm) as u32;

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
        *self.register(t) = (imm as u32).wrapping_shl(16);

        Ok(())
    }

    fn lhi(&mut self, t: u8, imm: u16) -> Result<()> {
        let value = (*self.register(t) & 0x0000FFFF) | ((imm as u32).wrapping_shl(16));

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
        let value = *self.register(s) < (imm as u32);

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
            *self.register(31) = self.registers.pc;

            self.skip(imm);
        }

        Ok(())
    }

    fn bgezal(&mut self, s: u8, imm: u16) -> Result<()> {
        if (*self.register(s) as i32) >= 0 {
            *self.register(31) = self.registers.pc;

            self.skip(imm);
        }

        Ok(())
    }

    fn j(&mut self, imm: u32) -> Result<()> {
        self.jump(imm);

        Ok(())
    }

    fn jal(&mut self, imm: u32) -> Result<()> {
        *self.register(31) = self.registers.pc;

        self.jump(imm);

        Ok(())
    }

    fn lb(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32).wrapping_add(imm as i16 as i32);

        *self.register(t) = self.memory.get(address as u32)? as i8 as i32 as u32;

        Ok(())
    }

    fn lbu(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32).wrapping_add(imm as i16 as i32);

        *self.register(t) = self.memory.get(address as u32)? as u32;

        Ok(())
    }

    fn lh(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32).wrapping_add(imm as i16 as i32);

        *self.register(t) = self.memory.get_u16(address as u32)? as i16 as i32 as u32;

        Ok(())
    }

    fn lhu(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32).wrapping_add(imm as i16 as i32);

        *self.register(t) = self.memory.get_u16(address as u32)? as u32;

        Ok(())
    }

    fn lw(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32).wrapping_add(imm as i16 as i32);

        *self.register(t) = self.memory.get_u32(address as u32)?;

        Ok(())
    }

    fn sb(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32).wrapping_add(imm as i16 as i32);
        let value = *self.register(t) as u8;

        self.memory.set(address as u32, value)?;

        Ok(())
    }

    fn sh(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32).wrapping_add(imm as i16 as i32);
        let value = *self.register(t) as u16;

        self.memory.set_u16(address as u32, value)?;

        Ok(())
    }

    fn sw(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        let address = (*self.register(s) as i32).wrapping_add(imm as i16 as i32);
        let value = *self.register(t);

        self.memory.set_u32(address as u32, value)?;

        Ok(())
    }

    fn mfhi(&mut self, d: u8) -> Result<()> {
        *self.register(d) = self.registers.hi;

        Ok(())
    }

    fn mflo(&mut self, d: u8) -> Result<()> {
        *self.register(d) = self.registers.lo;

        Ok(())
    }

    fn mthi(&mut self, s: u8) -> Result<()> {
        self.registers.hi = *self.register(s);

        Ok(())
    }

    fn mtlo(&mut self, s: u8) -> Result<()> {
        self.registers.lo = *self.register(s);

        Ok(())
    }

    fn trap(&mut self) -> Result<()> {
        Err(CpuTrap)
    }

    fn syscall(&mut self) -> Result<()> {
        Err(CpuSyscall)
    }

    fn add_s(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(*self.fp_register(s));
        let b = f32::from_bits(*self.fp_register(t));

        *self.fp_register(d) = (a + b).to_bits();

        Ok(())
    }
    fn sub_s(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(*self.fp_register(s));
        let b = f32::from_bits(*self.fp_register(t));

        *self.fp_register(d) = (a - b).to_bits();
        
        Ok(())
    }
    fn mul_s(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(*self.fp_register(s));
        let b = f32::from_bits(*self.fp_register(t));
        
        *self.fp_register(d) = (a * b).to_bits();
        
        Ok(())
    }
    fn div_s(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(*self.fp_register(s));
        let b = f32::from_bits(*self.fp_register(t));
        
        *self.fp_register(d) = (a / b).to_bits();
        
        Ok(())
    }
    fn sqrt_s(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(*self.fp_register(s));
        
        *self.fp_register(d) = a.sqrt().to_bits();
        
        Ok(())
    }
    fn abs_s(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(*self.fp_register(s));
        
        *self.fp_register(d) = a.abs().to_bits();
        
        Ok(())
    }
    fn neg_s(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(*self.fp_register(s));
        
        *self.fp_register(d) = (-a).to_bits();
        
        Ok(())
    }
    fn floor_w_s(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(*self.fp_register(s));
        
        *self.fp_register(d) = u32::from_le_bytes((a.floor() as i32).to_le_bytes());
        
        Ok(())
    }
    fn ceil_w_s(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(*self.fp_register(s));
        
        *self.fp_register(d) = u32::from_le_bytes((a.ceil() as i32).to_le_bytes());
        
        Ok(())
    }
    fn round_w_s(&mut self, s: u8, d: u8) -> Result<()> {
        let a = f32::from_bits(*self.fp_register(s));
        
        *self.fp_register(d) = u32::from_le_bytes((a.round() as i32).to_le_bytes());
        
        Ok(())
    }
    fn trunc_w_s(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn add_d(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn sub_d(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn mul_d(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn div_d(&mut self, s: u8, t: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn sqrt_d(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn abs_d(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn neg_d(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn floor_w_d(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn ceil_w_d(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn round_w_d(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn trunc_w_d(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn c_eq_s(&mut self, s: u8, t: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn c_le_s(&mut self, s: u8, t: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn c_lt_s(&mut self, s: u8, t: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn c_eq_d(&mut self, s: u8, t: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn c_le_d(&mut self, s: u8, t: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn c_lt_d(&mut self, s: u8, t: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn bc1t(&mut self, imm: u8, addr: u16) -> Result<()> {
        Ok(())
    }
    fn bc1f(&mut self, imm: u8, addr: u16) -> Result<()> {
        Ok(())
    }
    fn mov_s(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn movf_s(&mut self, s: u8, d: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn movt_s(&mut self, s: u8, d: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn movn_s(&mut self, s: u8, d: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn movz_s(&mut self, s: u8, d: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn mov_d(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn movf_d(&mut self, s: u8, d: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn movt_d(&mut self, s: u8, d: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn movn_d(&mut self, s: u8, d: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn movz_d(&mut self, s: u8, d: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn movf(&mut self, s: u8, d: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn movt(&mut self, s: u8, d: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn movn(&mut self, s: u8, d: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn movz(&mut self, s: u8, d: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn cvt_s_w(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn cvt_w_s(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn cvt_s_d(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn cvt_d_s(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn cvt_w_d(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn cvt_d_w(&mut self, s: u8, d: u8) -> Result<()> {
        Ok(())
    }
    fn mtc0(&mut self, s: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn mfc0(&mut self, s: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn mtc1(&mut self, s: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn mfc1(&mut self, s: u8, imm: u8) -> Result<()> {
        Ok(())
    }
    fn ldc1(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        Ok(())
    }
    fn sdc1(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        Ok(())
    }
    fn lwc1(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        Ok(())
    }
    fn swc1(&mut self, s: u8, t: u8, imm: u16) -> Result<()> {
        Ok(())
    }
}
