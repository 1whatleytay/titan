use crate::assembler::instructions::Size;

// noinspection SpellCheckingInspection
pub trait Decoder<T> {
    fn add(&mut self, s: u8, t: u8, d: u8) -> T;
    fn addu(&mut self, s: u8, t: u8, d: u8) -> T;
    fn and(&mut self, s: u8, t: u8, d: u8) -> T;
    fn div(&mut self, s: u8, t: u8) -> T;
    fn divu(&mut self, s: u8, t: u8) -> T;
    fn mult(&mut self, s: u8, t: u8) -> T;
    fn multu(&mut self, s: u8, t: u8) -> T;
    fn nor(&mut self, s: u8, t: u8, d: u8) -> T;
    fn or(&mut self, s: u8, t: u8, d: u8) -> T;
    fn sll(&mut self, t: u8, d: u8, sham: u8) -> T;
    fn sllv(&mut self, s: u8, t: u8, d: u8) -> T;
    fn sra(&mut self, t: u8, d: u8, sham: u8) -> T;
    fn srav(&mut self, s: u8, t: u8, d: u8) -> T;
    fn srl(&mut self, t: u8, d: u8, sham: u8) -> T;
    fn srlv(&mut self, s: u8, t: u8, d: u8) -> T;
    fn sub(&mut self, s: u8, t: u8, d: u8) -> T;
    fn subu(&mut self, s: u8, t: u8, d: u8) -> T;
    fn xor(&mut self, s: u8, t: u8, d: u8) -> T;
    fn slt(&mut self, s: u8, t: u8, d: u8) -> T;
    fn sltu(&mut self, s: u8, t: u8, d: u8) -> T;
    fn jr(&mut self, s: u8) -> T;
    fn jalr(&mut self, s: u8) -> T;

    fn madd(&mut self, s: u8, t: u8) -> T;
    fn maddu(&mut self, s: u8, t: u8) -> T;
    fn mul(&mut self, s: u8, t: u8, d: u8) -> T;
    fn msub(&mut self, s: u8, t: u8) -> T;
    fn msubu(&mut self, s: u8, t: u8) -> T;

    fn addi(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn addiu(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn andi(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn ori(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn xori(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn lui(&mut self, s: u8, imm: u16) -> T;
    fn lhi(&mut self, t: u8, imm: u16) -> T;
    fn llo(&mut self, t: u8, imm: u16) -> T;
    fn slti(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn sltiu(&mut self, s: u8, t: u8, imm: u16) -> T;

    fn beq(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn bne(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn bgtz(&mut self, s: u8, imm: u16) -> T;
    fn blez(&mut self, s: u8, imm: u16) -> T;

    fn bltz(&mut self, s: u8, imm: u16) -> T;
    fn bgez(&mut self, s: u8, imm: u16) -> T;
    fn bltzal(&mut self, s: u8, imm: u16) -> T;
    fn bgezal(&mut self, s: u8, imm: u16) -> T;

    fn j(&mut self, imm: u32) -> T;
    fn jal(&mut self, imm: u32) -> T;

    fn lb(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn lbu(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn lh(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn lhu(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn lw(&mut self, s: u8, t: u8, imm: u16) -> T;

    fn sb(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn sh(&mut self, s: u8, t: u8, imm: u16) -> T;
    fn sw(&mut self, s: u8, t: u8, imm: u16) -> T;

    fn mfhi(&mut self, d: u8) -> T;
    fn mflo(&mut self, d: u8) -> T;
    fn mthi(&mut self, s: u8) -> T;
    fn mtlo(&mut self, s: u8) -> T;

    fn trap(&mut self) -> T;
    fn syscall(&mut self) -> T;

    fn add_s(&mut self, t: u8, s: u8, d: u8) -> T;
    fn sub_s(&mut self, t: u8, s: u8, d: u8) -> T;
    fn mul_s(&mut self, t: u8, s: u8, d: u8) -> T;
    fn div_s(&mut self, t: u8, s: u8, d: u8) -> T;
    fn sqrt_s(&mut self, s: u8, d: u8) -> T;
    fn abs_s(&mut self, s: u8, d: u8) -> T;
    fn neg_s(&mut self, s: u8, d: u8) -> T;
    fn floor_w_s(&mut self, s: u8, d: u8) -> T;
    fn ceil_w_s(&mut self, s: u8, d: u8) -> T;
    fn round_w_s(&mut self, s: u8, d: u8) -> T;
    fn trunc_w_s(&mut self, s: u8, d: u8) -> T;
    fn add_d(&mut self, t: u8, s: u8, d: u8) -> T;
    fn sub_d(&mut self, t: u8, s: u8, d: u8) -> T;
    fn mul_d(&mut self, t: u8, s: u8, d: u8) -> T;
    fn div_d(&mut self, t: u8, s: u8, d: u8) -> T;
    fn sqrt_d(&mut self, s: u8, d: u8) -> T;
    fn abs_d(&mut self, s: u8, d: u8) -> T;
    fn neg_d(&mut self, s: u8, d: u8) -> T;
    fn floor_w_d(&mut self, s: u8, d: u8) -> T;
    fn ceil_w_d(&mut self, s: u8, d: u8) -> T;
    fn round_w_d(&mut self, s: u8, d: u8) -> T;
    fn trunc_w_d(&mut self, s: u8, d: u8) -> T;
    fn c_eq_s(&mut self, t: u8, s: u8, cc: u8) -> T;
    fn c_le_s(&mut self, t: u8, s: u8, cc: u8) -> T;
    fn c_lt_s(&mut self, t: u8, s: u8, cc: u8) -> T;
    fn c_eq_d(&mut self, t: u8, s: u8, cc: u8) -> T;
    fn c_le_d(&mut self, t: u8, s: u8, cc: u8) -> T;
    fn c_lt_d(&mut self, t: u8, s: u8, cc: u8) -> T;
    fn bc1t(&mut self, cc: u8, address: u16) -> T;
    fn bc1f(&mut self, cc: u8, address: u16) -> T;
    fn mov_s(&mut self, s: u8, d: u8) -> T;
    fn movf_s(&mut self, cc: u8, s: u8, d: u8) -> T;
    fn movt_s(&mut self, cc: u8, s: u8, d: u8) -> T;
    fn movn_s(&mut self, t: u8, s: u8, d: u8) -> T;
    fn movz_s(&mut self, t: u8, s: u8, d: u8) -> T;
    fn mov_d(&mut self, s: u8, d: u8) -> T;
    fn movf_d(&mut self, cc: u8, s: u8, d: u8) -> T;
    fn movt_d(&mut self, cc: u8, s: u8, d: u8) -> T;
    fn movn_d(&mut self, t: u8, s: u8, d: u8) -> T;
    fn movz_d(&mut self, t: u8, s: u8, d: u8) -> T;
    fn movf(&mut self, s: u8, cc: u8, d: u8) -> T;
    fn movt(&mut self, s: u8, cc: u8, d: u8) -> T;
    fn movn(&mut self, s: u8, t: u8, d: u8) -> T;
    fn movz(&mut self, s: u8, t: u8, d: u8) -> T;
    fn cvt_s_w(&mut self, s: u8, d: u8) -> T;
    fn cvt_w_s(&mut self, s: u8, d: u8) -> T;
    fn cvt_s_d(&mut self, s: u8, d: u8) -> T;
    fn cvt_d_s(&mut self, s: u8, d: u8) -> T;
    fn cvt_d_w(&mut self, s: u8, d: u8) -> T;
    fn cvt_w_d(&mut self, s: u8, d: u8) -> T;
    fn mtc1(&mut self, t: u8, s: u8) -> T;
    fn mfc1(&mut self, t: u8, s: u8) -> T;
    fn lwc1(&mut self, base: u8, t: u8, offset: u16) -> T;
    fn swc1(&mut self, base: u8, t: u8, offset: u16) -> T;
    fn ldc1(&mut self, base: u8, t: u8, offset: u16) -> T;
    fn sdc1(&mut self, base: u8, t: u8, offset: u16) -> T;

    fn dispatch_rtype(&mut self, instruction: u32) -> Option<T> {
        let func = instruction & 0x3F;

        let s = ((instruction >> 21) & 0x1F) as u8;
        let t = ((instruction >> 16) & 0x1F) as u8;
        let d = ((instruction >> 11) & 0x1F) as u8;
        let sham = ((instruction >> 6) & 0x1F) as u8;

        Some(match func {
            0 => self.sll(t, d, sham),
            1 => match t & 0b11 {
                0b00 => self.movf(s, d, t >> 2),
                0b01 => self.movt(s, d, t >> 2),
                _ => unreachable!(),
            },
            2 => self.srl(t, d, sham),
            3 => self.sra(t, d, sham),
            4 => self.sllv(s, t, d),
            6 => self.srlv(s, t, d),
            7 => self.srav(s, t, d),
            8 => self.jr(s),
            9 => self.jalr(s),
            10 => self.movz(s, t, d),
            11 => self.movn(s, t, d),
            12 => self.syscall(),
            16 => self.mfhi(d),
            17 => self.mthi(s),
            18 => self.mflo(d),
            19 => self.mtlo(s),
            24 => self.mult(s, t),
            25 => self.multu(s, t),
            26 => self.div(s, t),
            27 => self.divu(s, t),
            32 => self.add(s, t, d),
            33 => self.addu(s, t, d),
            34 => self.sub(s, t, d),
            35 => self.subu(s, t, d),
            36 => self.and(s, t, d),
            37 => self.or(s, t, d),
            38 => self.xor(s, t, d),
            39 => self.nor(s, t, d),
            41 => self.sltu(s, t, d),
            42 => self.slt(s, t, d),

            _ => return None,
        })
    }

    fn dispatch_special(&mut self, instruction: u32) -> Option<T> {
        let s = ((instruction >> 21) & 0x1F) as u8;
        let t = ((instruction >> 16) & 0x1F) as u8;
        let imm = (instruction & 0xFFFF) as u16;

        Some(match t {
            0 => self.bltz(s, imm),
            1 => self.bgez(s, imm),
            16 => self.bltzal(s, imm),
            17 => self.bgezal(s, imm),

            _ => return None,
        })
    }

    fn dispatch_algebra(&mut self, instruction: u32) -> Option<T> {
        let func = instruction & 0x3F;

        let s = ((instruction >> 21) & 0x1F) as u8;
        let t = ((instruction >> 16) & 0x1F) as u8;
        let d = ((instruction >> 11) & 0x1F) as u8;

        Some(match func {
            0 => self.madd(s, t),
            1 => self.maddu(s, t),
            2 => self.mul(s, t, d),
            4 => self.msub(s, t),
            5 => self.msubu(s, t),

            _ => return None,
        })
    }

    fn dispatch_cop1(&mut self, instruction: u32) -> Option<T> {
        let fmt = (instruction >> 21) & 0b11111;

        let t = ((instruction >> 16) & 0x1F) as u8;
        let s = ((instruction >> 11) & 0x1F) as u8;
        let d = ((instruction >> 6) & 0x1F) as u8;
        Some(match fmt {
            16 | 17 | 20 | 21 => {
                let instr = instruction & 0b11111;
                let ifmt = match fmt {
                    16 => Size::Single,
                    17 => Size::Double,
                    20 => Size::Word,
                    21 => unimplemented!(),
                    _ => unreachable!(),
                };
                match (instr, ifmt) {
                    (0, Size::Single) => self.add_s(t, s, d),
                    (1, Size::Single) => self.sub_s(t, s, d),
                    (2, Size::Single) => self.mul_s(t, s, d),
                    (3, Size::Single) => self.div_s(t, s, d),
                    (4, Size::Single) => self.sqrt_s(s, d),
                    (5, Size::Single) => self.abs_s(s, d),
                    (6, Size::Single) => self.mov_s(s, d),
                    (7, Size::Single) => self.neg_s(s, d),
                    (12, Size::Single) => self.round_w_s(s, d),
                    (13, Size::Single) => self.trunc_w_s(s, d),
                    (14, Size::Single) => self.ceil_w_s(s, d),
                    (15, Size::Single) => self.floor_w_s(s, d),
                    (17, Size::Single) => match t & 0b11 {
                        0b00 => self.movf_s(t >> 2, s, d),
                        0b01 => self.movt_s(t >> 2, s, d),
                        _ => unreachable!(),
                    },
                    (18, Size::Single) => self.movz_s(t, s, d),
                    (19, Size::Single) => self.movn_s(t, s, d),
                    (50, Size::Single) => self.c_eq_s(t, s, d >> 2),
                    (60, Size::Single) => self.c_lt_s(t, s, d >> 2),
                    (62, Size::Single) => self.c_le_s(t, s, d >> 2),

                    (0, Size::Double) => self.add_d(t, s, d),
                    (1, Size::Double) => self.sub_d(t, s, d),
                    (2, Size::Double) => self.mul_d(t, s, d),
                    (3, Size::Double) => self.div_d(t, s, d),
                    (4, Size::Double) => self.sqrt_d(s, d),
                    (5, Size::Double) => self.abs_d(s, d),
                    (6, Size::Double) => self.mov_d(s, d),
                    (7, Size::Double) => self.neg_d(s, d),
                    (12, Size::Double) => self.round_w_d(s, d),
                    (13, Size::Double) => self.trunc_w_d(s, d),
                    (14, Size::Double) => self.ceil_w_d(s, d),
                    (15, Size::Double) => self.floor_w_d(s, d),
                    (17, Size::Double) => match t & 0b11 {
                        0b00 => self.movf_d(t >> 2, s, d),
                        0b01 => self.movt_d(t >> 2, s, d),
                        _ => unreachable!(),
                    },
                    (18, Size::Double) => self.movz_d(t, s, d),
                    (19, Size::Double) => self.movn_d(t, s, d),
                    (50, Size::Double) => self.c_eq_d(t, s, d >> 2),
                    (60, Size::Double) => self.c_lt_d(t, s, d >> 2),
                    (62, Size::Double) => self.c_le_d(t, s, d >> 2),

                    (33, Size::Single) => self.cvt_d_s(s, d),
                    (33, Size::Word) => self.cvt_d_w(s, d),
                    (32, Size::Double) => self.cvt_s_d(s, d),
                    (32, Size::Word) => self.cvt_s_w(s, d),
                    (36, Size::Single) => self.cvt_w_s(s, d),
                    (36, Size::Double) => self.cvt_w_d(s, d),

                    _ => return None,
                }
            }
            0b00000 => self.mfc1(t, s),
            0b00100 => self.mtc1(t, s),
            0b01000 => {
                let tf = t & 0b11;
                let cc = (t >> 2) & 0b111;

                let addr = (instruction & 0xFFFF) as u16;
                match tf {
                    0 => return Some(self.bc1f(cc, addr)),
                    1 => return Some(self.bc1t(cc, addr)),
                    _ => unreachable!(),
                }
            }
            _ => return None,
        })
    }

    fn dispatch(&mut self, instruction: u32) -> Option<T> {
        let opcode = instruction >> 26;

        let s = ((instruction >> 21) & 0x1F) as u8;
        let t = ((instruction >> 16) & 0x1F) as u8;
        let imm = (instruction & 0xFFFF) as u16;
        let address = instruction & 0x03FFFFFF;

        Some(match opcode {
            0 => return self.dispatch_rtype(instruction),
            1 => return self.dispatch_special(instruction),
            2 => self.j(address),
            3 => self.jal(address),
            4 => self.beq(s, t, imm),
            5 => self.bne(s, t, imm),
            6 => self.blez(s, imm),
            7 => self.bgtz(s, imm),
            8 => self.addi(s, t, imm),
            9 => self.addiu(s, t, imm),
            10 => self.slti(s, t, imm),
            11 => self.sltiu(s, t, imm),
            12 => self.andi(s, t, imm),
            13 => self.ori(s, t, imm),
            14 => self.xori(s, t, imm),
            15 => self.lui(t, imm),
            17 => return self.dispatch_cop1(instruction),
            24 => self.llo(t, imm),
            25 => self.lhi(t, imm),
            26 => self.trap(),
            28 => return self.dispatch_algebra(instruction),
            32 => self.lb(s, t, imm),
            33 => self.lh(s, t, imm),
            35 => self.lw(s, t, imm),
            36 => self.lbu(s, t, imm),
            37 => self.lhu(s, t, imm),
            40 => self.sb(s, t, imm),
            41 => self.sh(s, t, imm),
            43 => self.sw(s, t, imm),

            49 => self.lwc1(s, t, imm),
            53 => self.ldc1(s, t, imm),
            57 => self.swc1(s, t, imm),
            61 => self.sdc1(s, t, imm),
            _ => return None,
        })
    }
}
