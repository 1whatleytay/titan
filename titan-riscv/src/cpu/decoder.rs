use titan_shared::utilities::bitwise::{pick_bits, sign_extend};

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum InstructionSize {
    Compressed, // 16-bit
    #[default]
    Regular, // 32-bit
}

impl InstructionSize {
    pub fn byte_size(self) -> u32 {
        match self {
            InstructionSize::Compressed => 2,
            InstructionSize::Regular => 4,
        }
    }
}

#[derive(Copy, Clone)]
struct InstructionParts(u32);

impl InstructionParts {
    fn op(self) -> u8 {
        pick_bits(self.0, 0, 7) as u8
    }

    fn rd(self) -> u8 {
        pick_bits(self.0, 7, 5) as u8
    }

    fn rs1(self) -> u8 {
        pick_bits(self.0, 15, 5) as u8
    }

    fn rs2(self) -> u8 {
        pick_bits(self.0, 20, 5) as u8
    }

    fn imm_upper(self) -> u32 {
        // 20-bits unsigned
        pick_bits(self.0, 12, 20)
    }

    fn imm_jump(self) -> i32 {
        // 20-bit
        let bits0to9 = pick_bits(self.0, 21, 10);
        let bits10 = pick_bits(self.0, 20, 1);
        let bits11to18 = pick_bits(self.0, 12, 8);
        let bits19 = pick_bits(self.0, 31, 1);

        let result = bits0to9 | (bits10 << 10) | (bits11to18 << 11) | (bits19 << 19);

        sign_extend(result, 20) as i32
    }

    fn imm_branch(self) -> i16 {
        // 12-bit
        let bits0to3 = pick_bits(self.0, 8, 4);
        let bits4to9 = pick_bits(self.0, 25, 6);
        let bits10 = pick_bits(self.0, 7, 1);
        let bits11 = pick_bits(self.0, 31, 1);

        let result = (bits0to3 | (bits4to9 << 4) | (bits10 << 10) | (bits11 << 11)) as u16;

        sign_extend(result, 12) as i16
    }

    fn imm_normal(self) -> i16 {
        // 12-bit
        sign_extend(pick_bits(self.0, 20, 12) as u16, 12) as i16
    }

    fn imm_store(self) -> i16 {
        // 12-bits
        let bits0to4 = pick_bits(self.0, 7, 5);
        let bits5to11 = pick_bits(self.0, 25, 7);

        let result = (bits0to4 | (bits5to11 << 5)) as u16;

        sign_extend(result, 12) as i16
    }

    fn sham(self) -> u8 {
        pick_bits(self.0, 20, 5) as u8
    }

    fn func_12(self) -> u8 {
        // 3-bit func
        pick_bits(self.0, 12, 3) as u8
    }

    fn func_20(self) -> u16 {
        // 12-bit func
        pick_bits(self.0, 20, 12) as u16
    }

    fn func_25(self) -> u8 {
        // 7-bit func
        pick_bits(self.0, 25, 7) as u8
    }
}

#[derive(Copy, Clone)]
struct CompressedInstructionParts(u16);

impl CompressedInstructionParts {
    fn op(self) -> u8 {
        pick_bits(self.0, 0, 2) as u8
    }

    // rd regular size is sharing spot with rs1
    fn rd(self) -> u8 {
        pick_bits(self.0, 7, 5) as u8
    }

    fn rs1(self) -> u8 {
        pick_bits(self.0, 7, 5) as u8
    }

    fn rs2(self) -> u8 {
        pick_bits(self.0, 2, 5) as u8
    }

    // BUT rd small is sharing spot with rs2
    // This is intentional, and instructions marked as rs1/rd in the manual should be treated as rs1_small
    fn rd_small(self) -> u8 {
        pick_bits(self.0, 2, 3) as u8
    }

    fn rs1_small(self) -> u8 {
        pick_bits(self.0, 7, 3) as u8
    }

    fn rs2_small(self) -> u8 {
        pick_bits(self.0, 2, 3) as u8
    }

    fn uimm_54276(self) -> u8 {
        // 6-bit imm
        let bits0to2 = pick_bits(self.0, 4, 3);
        let bits3 = pick_bits(self.0, 12, 1);
        let bits4to5 = pick_bits(self.0, 2, 2);

        (bits0to2 | (bits3 << 3) | (bits4to5 << 4)) as u8
    }

    fn uimm_5276(self) -> u8 {
        // 6-bit imm
        let bits0to3 = pick_bits(self.0, 9, 4);
        let bits4to5 = pick_bits(self.0, 7, 2);

        (bits0to3 | (bits4to5 << 4)) as u8
    }

    fn uimm_5326(self) -> u8 {
        // 5-bit imm
        let bits0 = pick_bits(self.0, 6, 1);
        let bits1to3 = pick_bits(self.0, 10, 3);
        let bits4 = pick_bits(self.0, 5, 1);

        (bits0 | (bits1to3 << 1) | (bits4 << 4)) as u8
    }

    fn imm_jump(self) -> i16 {
        // 11-bit imm
        let bits0to2 = pick_bits(self.0, 3, 3);
        let bits3 = pick_bits(self.0, 11, 1);
        let bits4 = pick_bits(self.0, 2, 1);
        let bits5 = pick_bits(self.0, 7, 1);
        let bits6 = pick_bits(self.0, 6, 1);
        let bits7to8 = pick_bits(self.0, 9, 2);
        let bits9 = pick_bits(self.0, 8, 1);
        let bits10 = pick_bits(self.0, 12, 1);

        let result = bits0to2
            | (bits3 << 3)
            | (bits4 << 4)
            | (bits5 << 5)
            | (bits6 << 6)
            | (bits7to8 << 7)
            | (bits9 << 9)
            | (bits10 << 10);

        sign_extend(result, 11) as i16
    }

    fn imm_branch(self) -> i8 {
        // 8-bit imm
        let bits0to1 = pick_bits(self.0, 3, 2);
        let bits2to3 = pick_bits(self.0, 10, 2);
        let bits4 = pick_bits(self.0, 2, 1);
        let bits5to6 = pick_bits(self.0, 5, 2);
        let bits7 = pick_bits(self.0, 12, 1);

        let result = bits0to1 | (bits2to3 << 2) | (bits4 << 4) | (bits5to6 << 5) | (bits7 << 7);

        sign_extend(result, 8) as i8 // yes this can be done by the language, just being explicit here
    }

    fn imm_540(self) -> i8 {
        // 6-bit imm
        let bits0to4 = pick_bits(self.0, 2, 5);
        let bits5 = pick_bits(self.0, 12, 1);

        let result = bits0to4 | (bits5 << 5);

        sign_extend(result, 6) as i8
    }

    fn imm_946875(self) -> i8 {
        // 6-bit imm
        let bits0 = pick_bits(self.0, 6, 1);
        let bits1 = pick_bits(self.0, 2, 1);
        let bits2 = pick_bits(self.0, 5, 1);
        let bits3to4 = pick_bits(self.0, 3, 2);
        let bits5 = pick_bits(self.0, 12, 1);

        let result = bits0 | (bits1 << 1) | (bits2 << 2) | (bits3to4 << 3) | (bits5 << 5);

        sign_extend(result, 6) as i8
    }

    fn uimm_549623(self) -> u8 {
        // 8-bit imm
        let bits0 = pick_bits(self.0, 6, 1);
        let bits1 = pick_bits(self.0, 5, 1);
        let bits2to3 = pick_bits(self.0, 11, 2);
        let bits4to7 = pick_bits(self.0, 7, 4);

        (bits0 | (bits1 << 1) | (bits2to3 << 2) | (bits4to7 << 4)) as u8
    }

    fn sham(self) -> u8 {
        let bits0to4 = pick_bits(self.0, 2, 5);
        let bits5 = pick_bits(self.0, 12, 1);

        (bits0to4 | (bits5 << 5)) as u8
    }

    fn func_5(self) -> u8 {
        pick_bits(self.0, 5, 2) as u8
    }

    fn func_10(self) -> u8 {
        pick_bits(self.0, 10, 2) as u8
    }

    fn func_12(self) -> bool {
        pick_bits(self.0, 12, 1) != 0
    }

    fn func_13(self) -> u8 {
        pick_bits(self.0, 13, 3) as u8
    }
}

// noinspection SpellCheckingInspection
pub trait Decoder<T> {
    fn lui(&mut self, rd: u8, imm_upper: u32) -> T;
    fn auipc(&mut self, rd: u8, imm_upper: u32) -> T;
    fn jal(&mut self, rd: u8, imm_jump: i32) -> T;
    fn beq(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> T;
    fn bne(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> T;
    fn blt(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> T;
    fn bge(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> T;
    fn bltu(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> T;
    fn bgeu(&mut self, rs1: u8, rs2: u8, imm_branch: i16) -> T;
    fn jalr(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> T;
    fn lb(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> T;
    fn lh(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> T;
    fn lw(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> T;
    fn lbu(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> T;
    fn lhu(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> T;
    fn addi(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> T;
    fn slti(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> T;
    fn sltiu(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> T;
    fn xori(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> T;
    fn ori(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> T;
    fn andi(&mut self, rd: u8, rs1: u8, imm_normal: i16) -> T;
    fn sb(&mut self, rs1: u8, rs2: u8, imm_store: i16) -> T;
    fn sh(&mut self, rs1: u8, rs2: u8, imm_store: i16) -> T;
    fn sw(&mut self, rs1: u8, rs2: u8, imm_store: i16) -> T;
    fn slli(&mut self, rd: u8, rs1: u8, sham: u8) -> T;
    fn srli(&mut self, rd: u8, rs1: u8, sham: u8) -> T;
    fn srai(&mut self, rd: u8, rs1: u8, sham: u8) -> T;
    fn add(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn sub(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn sll(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn slt(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn sltu(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn xor(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn srl(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn sra(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn or(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn and(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn fence(&mut self) -> T;
    fn ecall(&mut self) -> T;
    fn ebreak(&mut self) -> T;

    // M Extension (MulDiv)
    fn mul(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn mulh(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn mulhsu(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn mulhu(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn div(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn divu(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn rem(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;
    fn remu(&mut self, rd: u8, rs1: u8, rs2: u8) -> T;

    // C Extension (Compressed)
    fn c_lwsp(&mut self, rd: u8, uimm_54276: u8) -> T;
    fn c_swsp(&mut self, rs2: u8, uimm_5276: u8) -> T;
    fn c_lw(&mut self, rd_small: u8, rs1_small: u8, uimm_5326: u8) -> T;
    fn c_sw(&mut self, rs1_small: u8, rs2_small: u8, uimm_5326: u8) -> T;
    fn c_j(&mut self, imm_jump: i16) -> T;
    fn c_jal(&mut self, imm_jump: i16) -> T;
    fn c_jr(&mut self, rs1: u8) -> T; // rs2 must be zero?
    fn c_jalr(&mut self, rs1: u8) -> T; // rs2 must be zero?
    fn c_beqz(&mut self, rs1_small: u8, imm_branch: i8) -> T;
    fn c_bnez(&mut self, rs1_small: u8, imm_branch: i8) -> T;
    fn c_li(&mut self, rd: u8, imm_540: i8) -> T;
    fn c_lui(&mut self, rd: u8, imm_540: i8) -> T;
    fn c_addi(&mut self, rd: u8, imm_540: i8) -> T;
    fn c_addi16sp(&mut self, imm_946875: i8) -> T;
    fn c_addi4spn(&mut self, rd_small: u8, uimm_549623: u8) -> T;
    fn c_slli(&mut self, rd: u8, sham: u8) -> T;
    fn c_srli(&mut self, rs1_small: u8, sham: u8) -> T;
    fn c_srai(&mut self, rs1_small: u8, sham: u8) -> T;
    fn c_andi(&mut self, rs1_small: u8, imm_540: i8) -> T;
    fn c_mv(&mut self, rd: u8, rs2: u8) -> T;
    fn c_add(&mut self, rd: u8, rs2: u8) -> T;
    fn c_and(&mut self, rs1_small: u8, rs2_small: u8) -> T;
    fn c_or(&mut self, rs1_small: u8, rs2_small: u8) -> T;
    fn c_xor(&mut self, rs1_small: u8, rs2_small: u8) -> T;
    fn c_sub(&mut self, rs1_small: u8, rs2_small: u8) -> T;
    fn c_nop(&mut self) -> T;
    fn c_ebreak(&mut self) -> T;

    fn dispatch_branch(&mut self, instruction: u32) -> Option<T> {
        let parts = InstructionParts(instruction);

        let rs1 = parts.rs1();
        let rs2 = parts.rs2();
        let imm_branch = parts.imm_branch();

        Some(match parts.func_12() {
            0b000 => self.beq(rs1, rs2, imm_branch),
            0b001 => self.bne(rs1, rs2, imm_branch),
            0b100 => self.blt(rs1, rs2, imm_branch),
            0b101 => self.bge(rs1, rs2, imm_branch),
            0b110 => self.bltu(rs1, rs2, imm_branch),
            0b111 => self.bgeu(rs1, rs2, imm_branch),
            _ => return None,
        })
    }

    fn dispatch_load(&mut self, instruction: u32) -> Option<T> {
        let parts = InstructionParts(instruction);

        let rd = parts.rd();
        let rs1 = parts.rs1();
        let imm_normal = parts.imm_normal();

        Some(match parts.func_12() {
            0b000 => self.lb(rd, rs1, imm_normal),
            0b001 => self.lh(rd, rs1, imm_normal),
            0b010 => self.lw(rd, rs1, imm_normal),
            0b100 => self.lbu(rd, rs1, imm_normal),
            0b101 => self.lhu(rd, rs1, imm_normal),

            _ => return None,
        })
    }

    fn dispatch_store(&mut self, instruction: u32) -> Option<T> {
        let parts = InstructionParts(instruction);

        let rs1 = parts.rs1();
        let rs2 = parts.rs2();
        let imm_store = parts.imm_store();

        Some(match parts.func_12() {
            0b000 => self.sb(rs1, rs2, imm_store),
            0b001 => self.sh(rs1, rs2, imm_store),
            0b010 => self.sw(rs1, rs2, imm_store),
            _ => return None,
        })
    }

    fn dispatch_immediate(&mut self, instruction: u32) -> Option<T> {
        let parts = InstructionParts(instruction);

        let rd = parts.rd();
        let rs1 = parts.rs1();
        let imm_normal = parts.imm_normal();
        let sham = parts.sham();

        Some(match parts.func_12() {
            0b000 => self.addi(rd, rs1, imm_normal),
            0b010 => self.slti(rd, rs1, imm_normal),
            0b011 => self.sltiu(rd, rs1, imm_normal),
            0b100 => self.xori(rd, rs1, imm_normal),
            0b110 => self.ori(rd, rs1, imm_normal),
            0b111 => self.andi(rd, rs1, imm_normal),
            0b001 if parts.func_25() == 0 => self.slli(rd, rs1, sham),
            0b101 if parts.func_25() == 0 => self.srli(rd, rs1, sham),
            0b101 if parts.func_25() == 0b0100000 => self.srai(rd, rs1, sham),
            _ => return None,
        })
    }

    fn dispatch_register(&mut self, instruction: u32) -> Option<T> {
        let parts = InstructionParts(instruction);

        let rd = parts.rd();
        let rs1 = parts.rs1();
        let rs2 = parts.rs2();

        let func_12 = parts.func_12();
        let func_25 = parts.func_25();

        Some(match func_25 {
            0b0000000 => match func_12 {
                0b000 => self.add(rd, rs1, rs2),
                0b001 => self.sll(rd, rs1, rs2),
                0b010 => self.slt(rd, rs1, rs2),
                0b011 => self.sltu(rd, rs1, rs2),
                0b100 => self.xor(rd, rs1, rs2),
                0b101 => self.srl(rd, rs1, rs2),
                0b110 => self.or(rd, rs1, rs2),
                0b111 => self.and(rd, rs1, rs2),
                _ => return None,
            },
            0b0100000 => match func_12 {
                0b000 => self.sub(rd, rs1, rs2),
                0b101 => self.sra(rd, rs1, rs2),
                _ => return None,
            },
            0b0000001 => match func_12 {
                0b000 => self.mul(rd, rs1, rs2),
                0b001 => self.mulh(rd, rs1, rs2),
                0b010 => self.mulhsu(rd, rs1, rs2),
                0b011 => self.mulhu(rd, rs1, rs2),
                0b100 => self.div(rd, rs1, rs2),
                0b101 => self.divu(rd, rs1, rs2),
                0b110 => self.rem(rd, rs1, rs2),
                0b111 => self.remu(rd, rs1, rs2),
                _ => return None,
            },
            _ => return None,
        })
    }

    fn dispatch_system(&mut self, instruction: u32) -> Option<T> {
        let parts = InstructionParts(instruction);

        // These should be zero!
        if parts.rd() != 0 || parts.rs1() != 0 || parts.func_12() != 0 {
            return None;
        }

        Some(match parts.func_20() {
            0 => self.ecall(),
            1 => self.ebreak(),
            _ => return None,
        })
    }

    fn dispatch_regular(&mut self, instruction: u32) -> Option<T> {
        let parts = InstructionParts(instruction);

        Some(match parts.op() {
            0b0110111 => self.lui(parts.rd(), parts.imm_upper()),
            0b0010111 => self.auipc(parts.rd(), parts.imm_upper()),
            0b1101111 => self.jal(parts.rd(), parts.imm_jump()),
            0b1100011 => return self.dispatch_branch(instruction),
            0b1100111 => self.jalr(parts.rd(), parts.rs1(), parts.imm_normal()),
            0b0000011 => return self.dispatch_load(instruction),
            0b0010011 => return self.dispatch_immediate(instruction),
            0b0100011 => return self.dispatch_store(instruction),
            0b0110011 => return self.dispatch_register(instruction),
            0b0001111 if parts.func_12() == 0 => self.fence(),
            0b1110011 => return self.dispatch_system(instruction),

            _ => return None,
        })
    }

    fn dispatch_compressed(&mut self, instruction: u16) -> Option<T> {
        let parts = CompressedInstructionParts(instruction);

        if instruction == 0 {
            return None; // explicitely marked as invalid - not c.addi4spn
        }

        let func_13 = parts.func_13();

        Some(match parts.op() {
            0b00 => match func_13 {
                0b000 => self.c_addi4spn(parts.rd_small(), parts.uimm_549623()),
                0b010 => self.c_lw(parts.rd_small(), parts.rs1_small(), parts.uimm_5326()),
                0b110 => self.c_sw(parts.rs1_small(), parts.rs2_small(), parts.uimm_5326()),
                _ => return None,
            },
            0b01 => match func_13 {
                0b000 => match parts.rs1() {
                    0 => self.c_nop(),
                    rs1 => self.c_addi(rs1, parts.imm_540()),
                },
                0b001 => self.c_jal(parts.imm_jump()),
                0b010 => self.c_li(parts.rd(), parts.imm_540()),
                0b011 => match parts.rd() {
                    2 => self.c_addi16sp(parts.imm_946875()), // 2 -> sp
                    rd => self.c_lui(rd, parts.imm_540()),
                },
                0b100 => match parts.func_10() {
                    0b00 => self.c_srli(parts.rs1_small(), parts.sham()),
                    0b01 => self.c_srai(parts.rs1_small(), parts.sham()),
                    0b10 => self.c_andi(parts.rs1_small(), parts.imm_540()),
                    0b11 => match (parts.func_12(), parts.func_5()) {
                        (false, 0b00) => self.c_sub(parts.rs1_small(), parts.rs2_small()),
                        (false, 0b01) => self.c_xor(parts.rs1_small(), parts.rs2_small()),
                        (false, 0b11) => self.c_and(parts.rs1_small(), parts.rs2_small()),
                        _ => return None,
                    },
                    _ => return None,
                },
                0b101 => self.c_j(parts.imm_jump()),
                0b110 => self.c_beqz(parts.rs1_small(), parts.imm_branch()),
                0b111 => self.c_bnez(parts.rs1_small(), parts.imm_branch()),
                _ => return None,
            },
            0b10 => match func_13 {
                0b000 => self.c_slli(parts.rd(), parts.sham()),
                0b010 => self.c_lwsp(parts.rd(), parts.uimm_54276()),
                0b100 => {
                    if parts.func_12() {
                        match (parts.rs1(), parts.rs2()) {
                            (0, 0) => self.c_ebreak(),
                            (rs1, 0) => self.c_jalr(rs1),
                            (rs1, rs2) => self.c_add(rs1, rs2),
                        }
                    } else {
                        match (parts.rs1(), parts.rs2()) {
                            (0, 0) => return None,
                            (rs1, 0) => self.c_jr(rs1),
                            (rs1, rs2) => self.c_mv(rs1, rs2),
                        }
                    }
                }
                0b110 => self.c_swsp(parts.rs2(), parts.uimm_5276()),
                _ => return None,
            },
            _ => return None, // 0b11
        })
    }

    // If you only have 16-bits of opcode, just cast to u32.
    // decoder.dispatch(instruction_u16 as u32)
    fn dispatch(&mut self, instruction: u32) -> Option<(T, InstructionSize)> {
        let regular_opcode = 0b11;

        if (instruction & regular_opcode) == regular_opcode {
            self.dispatch_regular(instruction)
                .map(|value| (value, InstructionSize::Regular))
        } else {
            self.dispatch_compressed(instruction as u16)
                .map(|value| (value, InstructionSize::Compressed))
        }
    }
}
