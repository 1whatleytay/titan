use crate::assembler::instructions::{BaseOpcode, CompressedOpcode};
use crate::assembler::registers::{CompressedRegisterSlot, RegisterSlot};
use num_traits::{ToPrimitive, Unsigned, WrappingShl, WrappingShr};
use std::ops::{BitAnd, Not};

fn instruction_base(op: &BaseOpcode) -> u32 {
    fn get_flags(flags: bool) -> u32 {
        if flags { 0b0100000 << 25 } else { 0 }
    }

    match op {
        BaseOpcode::Op(key) => *key as u32 & 0b1111111,
        BaseOpcode::BranchFunc(func) => ((*func as u32) << 12) | 0b1100011,
        BaseOpcode::LoadFunc(func) => ((*func as u32) << 12) | 0b0000011,
        BaseOpcode::ImmediateFunc(func, flags) => {
            ((*func as u32) << 12) | 0b0010011 | get_flags(*flags)
        }
        BaseOpcode::StoreFunc(func) => ((*func as u32) << 12) | 0b0100011,
        BaseOpcode::RegisterFunc(func, flags) => {
            ((*func as u32) << 12) | 0b0110011 | get_flags(*flags)
        }
        BaseOpcode::Executive(value) => ((*value as u32) << 20) | 0b1110011,
        BaseOpcode::MulDiv(func) => (0b0000001 << 25) | ((*func as u32) << 12) | 0b0110011,
    }
}

fn compressed_base(op: u8, func: u8) -> u16 {
    let op = op as u16;
    let func = func as u16;

    (op & 0b11) | ((func & 0b111) << 13)
}

fn compressed_bit12(bit12: bool) -> u16 {
    if bit12 { 0b1 << 12 } else { 0b0 }
}

fn instruction_compressed(op: &CompressedOpcode) -> u16 {
    match op {
        CompressedOpcode::OpFunc { op, func } => compressed_base(*op, *func),
        CompressedOpcode::SpecRd { op, func, rd } => {
            compressed_base(*op, *func) | ((compressed_source(*rd) & 0b11111) << 7)
        }
        CompressedOpcode::SpecHigh { op, func, high } => {
            compressed_base(*op, *func) | ((*high as u16 & 0b11) << 10)
        }
        CompressedOpcode::SpecHighLow {
            op,
            func,
            high,
            low,
            bit12,
        } => {
            compressed_base(*op, *func)
                | ((*high as u16 & 0b11) << 10)
                | ((*low as u16 & 0b11) << 5)
                | compressed_bit12(*bit12)
        }
        CompressedOpcode::Spec12 { op, func, bit12 } => {
            compressed_base(*op, *func) | compressed_bit12(*bit12)
        }
    }
}

fn register_source(slot: RegisterSlot) -> u32 {
    slot.to_u32().unwrap()
}

fn compressed_source(slot: RegisterSlot) -> u16 {
    slot.to_u16().unwrap()
}

fn compressed_source_small(slot: CompressedRegisterSlot) -> u16 {
    slot.to_u16().unwrap()
}

fn pick_bits<
    T: Sized
        + Unsigned
        + WrappingShl<Output = T>
        + WrappingShr<Output = T>
        + Default
        + BitAnd<Output = T>
        + Not<Output = T>,
>(
    value: T,
    start: u32,
    count: u32,
) -> T {
    let bits = size_of::<T>() * 8;

    let cut_off = bits as u32 - count;

    // Hopefully the type is unsigned!
    let mask = T::default()
        .not()
        .wrapping_shl(cut_off)
        .wrapping_shr(cut_off);

    value.bitand(mask.wrapping_shl(start)).wrapping_shr(start)
}

pub struct InstructionBuilder(pub u32);

impl InstructionBuilder {
    pub fn from_op(op: &BaseOpcode) -> InstructionBuilder {
        InstructionBuilder(instruction_base(op))
    }

    pub fn with_rs1(mut self, slot: RegisterSlot) -> InstructionBuilder {
        self.0 &= !(0b11111 << 15);
        self.0 |= register_source(slot) << 15;

        self
    }

    pub fn with_rs2(mut self, slot: RegisterSlot) -> InstructionBuilder {
        self.0 &= !(0b11111 << 20);
        self.0 |= register_source(slot) << 20;

        self
    }

    pub fn with_rd(mut self, slot: RegisterSlot) -> InstructionBuilder {
        self.0 &= !(0b11111 << 7);
        self.0 |= register_source(slot) << 7;

        self
    }

    pub fn with_normal_imm(mut self, value: i16) -> InstructionBuilder {
        // 12 bits
        let value = value as u16 as u32;

        self.0 &= !(0b111111111111 << 20);
        self.0 |= value << 20;

        self
    }

    pub fn with_branch_imm(mut self, value: i16) -> InstructionBuilder {
        // 12 bits
        let value = value as i32 as u32;
        // 12, 10:5, 4:1, 11

        self.0 &= !(0b1111111 << 25); // clear 25-31 (7 bits)
        self.0 &= !(0b11111 << 7); // clear 7-11 (5 bits)

        let bit11 = (value & (0b1 << 11)) >> 11;
        let bit10 = (value & (0b1 << 10)) >> 10;
        let bit4to9 = (value & (0b111111 << 4)) >> 4;
        let bit0to3 = value & (0b1111);

        self.0 |= bit10 << 7;
        self.0 |= bit0to3 << 8;
        self.0 |= bit4to9 << 25;
        self.0 |= bit11 << 31;

        self
    }

    pub fn with_store_imm(mut self, value: i16) -> InstructionBuilder {
        // 12 bits
        let value = value as i32 as u32;

        self.0 &= !(0b1111111 << 25); // clear 25-31 (7 bits)
        self.0 &= !(0b11111 << 7); // clear 7-11 (5 bits)

        let bit0to4 = value & 0b11111;
        let bit5to11 = (value & (0b111111 << 5)) >> 5;

        self.0 |= bit0to4 << 7;
        self.0 |= bit5to11 << 25;

        self
    }

    pub fn with_sham(mut self, value: u8) -> InstructionBuilder {
        // 5 bits
        self.0 &= !(0b11111 << 20); // clear 20-24

        self.0 |= (value as u32 & 0b11111) << 20;

        self
    }

    // Always signed input.
    pub fn with_jal_imm(mut self, value: i32) -> InstructionBuilder {
        // 20 bits
        let value = value as u32;

        self.0 &= !(0b11111111111111111111 << 20); // clear 12-31

        let bit19 = (value & (0b1 << 19)) >> 19;
        let bit0to9 = value & (0b1111111111);
        let bit10 = (value & (0b1 << 10)) >> 10;
        let bit11to18 = (value & (0b11111111 << 11)) >> 11;

        self.0 |= bit11to18 << 12;
        self.0 |= bit10 << 20;
        self.0 |= bit0to9 << 21;
        self.0 |= bit19 << 31;

        self
    }

    pub fn with_upper_imm(mut self, value: u32) -> InstructionBuilder {
        // 20 bits
        self.0 &= !(0b11111111111111111111 << 20); // clear 12-31

        self.0 |= value << 12;

        self
    }
}

pub struct CompressedInstructionBuilder(pub u16);

impl CompressedInstructionBuilder {
    pub fn from_op(op: &CompressedOpcode) -> CompressedInstructionBuilder {
        CompressedInstructionBuilder(instruction_compressed(op))
    }

    pub fn with_rd_small(mut self, slot: CompressedRegisterSlot) -> CompressedInstructionBuilder {
        self.0 &= !(0b111 << 2);
        self.0 |= compressed_source_small(slot) << 2;

        self
    }

    pub fn with_rs1_small(mut self, slot: CompressedRegisterSlot) -> CompressedInstructionBuilder {
        self.0 &= !(0b111 << 7);
        self.0 |= compressed_source_small(slot) << 7;

        self
    }

    pub fn with_rs2_small(mut self, slot: CompressedRegisterSlot) -> CompressedInstructionBuilder {
        self.0 &= !(0b111 << 2);
        self.0 |= compressed_source_small(slot) << 2;

        self
    }

    pub fn with_rd(mut self, slot: RegisterSlot) -> CompressedInstructionBuilder {
        self.0 &= !(0b11111 << 7);
        self.0 |= compressed_source(slot) << 7;

        self
    }

    pub fn with_rs1(mut self, slot: RegisterSlot) -> CompressedInstructionBuilder {
        self.0 &= !(0b11111 << 7);
        self.0 |= compressed_source(slot) << 7;

        self
    }

    pub fn with_rs2(mut self, slot: RegisterSlot) -> CompressedInstructionBuilder {
        self.0 &= !(0b11111 << 2);
        self.0 |= compressed_source(slot) << 2;

        self
    }

    pub fn with_uimm_549623(mut self, value: u8) -> CompressedInstructionBuilder {
        // 8 bit immediate
        let value = value as u16;

        let bit0 = pick_bits(value, 0, 1);
        let bit1 = pick_bits(value, 1, 1);
        let bit2to3 = pick_bits(value, 2, 2);
        let bit4to7 = pick_bits(value, 4, 4);

        self.0 &= !(0b11111111 << 5);

        self.0 |= bit1 << 5;
        self.0 |= bit0 << 6;
        self.0 |= bit4to7 << 7;
        self.0 |= bit2to3 << 11;

        self
    }

    pub fn with_uimm_5326(mut self, value: u8) -> CompressedInstructionBuilder {
        // 5 bit
        let value = value as u16;

        let bit1to3 = pick_bits(value, 1, 3);
        let bit0 = pick_bits(value, 0, 1);
        let bit4 = pick_bits(value, 4, 1);

        self.0 &= !(0b111 << 10);
        self.0 &= !(0b11 << 5);

        self.0 |= bit4 << 5;
        self.0 |= bit0 << 6;
        self.0 |= bit1to3 << 10;

        panic!()
    }

    pub fn with_imm_540(mut self, value: i8) -> CompressedInstructionBuilder {
        // 6 bit
        let value = value as u8 as u16;

        let bit0to4 = pick_bits(value, 0, 5);
        let bit5 = pick_bits(value, 5, 1);

        self.0 &= !(0b1 << 12);
        self.0 &= !(0b11111 << 2);

        self.0 |= bit0to4 << 2;
        self.0 |= bit5 << 2;

        self
    }

    pub fn with_uimm_54276(mut self, value: u8) -> CompressedInstructionBuilder {
        // 6 bit
        let value = value as u16;

        let bit0to2 = pick_bits(value, 0, 3);
        let bit3 = pick_bits(value, 3, 1);
        let bit4to5 = pick_bits(value, 4, 2);

        self.0 &= !(0b1 << 12);
        self.0 &= !(0b11111 << 2);

        self.0 |= bit4to5 << 2;
        self.0 |= bit0to2 << 4;
        self.0 |= bit3 << 12;

        self
    }

    pub fn with_uimm_5276(mut self, value: u8) -> CompressedInstructionBuilder {
        // 6 bit
        let value = value as u16;

        let bit0to3 = pick_bits(value, 0, 4);
        let bit4to5 = pick_bits(value, 4, 2);

        self.0 &= !(0b111111 << 7);

        self.0 |= bit4to5 << 7;
        self.0 |= bit0to3 << 9;

        self
    }

    pub fn with_jump_imm(mut self, value: i16) -> CompressedInstructionBuilder {
        let value = value as u16;

        // 11 bits
        let bit0to2 = pick_bits(value, 0, 3);
        let bit3 = pick_bits(value, 3, 1);
        let bit4 = pick_bits(value, 4, 1);
        let bit5 = pick_bits(value, 5, 1);
        let bit6 = pick_bits(value, 6, 1);
        let bit7to8 = pick_bits(value, 7, 2);
        let bit9 = pick_bits(value, 9, 1);
        let bit10 = pick_bits(value, 10, 1);

        self.0 &= !(0b11111111111 << 2);

        self.0 |= bit4 << 2;
        self.0 |= bit0to2 << 3;
        self.0 |= bit6 << 6;
        self.0 |= bit5 << 7;
        self.0 |= bit9 << 8;
        self.0 |= bit7to8 << 9;
        self.0 |= bit3 << 11;
        self.0 |= bit10 << 12;

        self
    }

    pub fn with_branch_imm(mut self, value: i8) -> CompressedInstructionBuilder {
        // 8 bits
        let value = value as u8 as u16;

        let bit0to1 = pick_bits(value, 0, 2);
        let bit2to3 = pick_bits(value, 2, 2);
        let bit4 = pick_bits(value, 4, 1);
        let bit5to6 = pick_bits(value, 5, 2);
        let bit7 = pick_bits(value, 7, 1);

        self.0 &= !(0b11111 << 2);
        self.0 &= !(0b111 << 10);

        self.0 |= bit4 << 2;
        self.0 |= bit0to1 << 3;
        self.0 |= bit5to6 << 5;
        self.0 |= bit2to3 << 10;
        self.0 |= bit7 << 12;

        self
    }

    pub fn with_sham(mut self, value: u8) -> CompressedInstructionBuilder {
        // technically 6 bit, but top bit must be zero for RV32
        let value = value as u16;

        let bit0to4 = pick_bits(value, 0, 5);
        let bit5 = pick_bits(value, 5, 1);

        self.0 &= !(0b1 << 12);
        self.0 &= !(0b11111 << 2);

        self.0 |= bit0to4 << 2;
        self.0 |= bit5 << 2;

        self
    }

    /*
    with_rd_small
    with_rs1_small
    with_rs2_small
    with_rd - non zero + for c.lui not 2
    with_rs1
    with_rs2
    with_uimm_549623 - c.addi4spn
    with_uimm_5326 - c.lw c.sw
    with_imm_540 - c.nop c.addi c.li
    with_imm_946875 - c.addi16sp
    with_uimm_54276 - c.lwsp
    with_uimm_5276 - c.swsp
    with_jump_imm - c.jal
    with_branch_imm
    with_sham - NonZero -> reserved for some hint instructions
     */
}

pub struct SplitImmediate {
    pub upper: u32, // 20 bits
    pub lower: i16, // 12 bits
}

impl SplitImmediate {
    pub fn from_immediate(value: u32) -> SplitImmediate {
        // Within signed 12 Bits
        let result = if (-0x800..=0x7ff).contains(&(value as i32)) {
            // Using this branch to handle negative 12-bit cases that the other branch does not handle too well.
            SplitImmediate {
                upper: 0,
                lower: value as i16,
            }
        } else {
            let upper = value >> 12;

            let difference = value.wrapping_sub(upper << 12) as i32;

            if difference < 0x7ff {
                SplitImmediate {
                    upper,
                    lower: difference as i16,
                }
            } else {
                SplitImmediate {
                    upper: upper + 1,
                    lower: (difference - 0x1000) as i16,
                }
            }
        };

        assert_eq!(
            (result.upper << 12).wrapping_add(result.lower as i32 as u32),
            value
        );
        assert!((-0x800..=0x7ff).contains(&result.lower));

        result
    }
}
