use crate::assembler::instructions::{BaseOpcode, CompressedOpcode};
use crate::assembler::registers::RegisterSlot;
use num_traits::ToPrimitive;

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
        BaseOpcode::RegisterFunc(func, flags) => ((*func as u32) << 12) | 0b0110011 | get_flags(*flags),
        BaseOpcode::Executive(value) => ((*value as u32) << 20) | 0b1110011,
    }
}

fn instruction_compressed(op: &CompressedOpcode) -> u16 {
    panic!()
}

fn register_source(slot: RegisterSlot) -> u32 {
    slot.to_u32().unwrap()
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
