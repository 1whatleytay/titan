use std::collections::HashMap;
use byteorder::{LittleEndian, WriteBytesExt};
use num_traits::ToPrimitive;
use crate::assembler::binary::{BinaryBuilder, InstructionLabel};
use crate::assembler::binary::InstructionLabel::{BranchLabel, JumpLabel};
use crate::assembler::instructions::{Encoding, Instruction, Opcode};
use crate::assembler::instructions::Opcode::{Op, Func, Special};
use crate::assembler::lexer_seek::{LexerSeek, LexerSeekPeekable};
use crate::assembler::registers::RegisterSlot;
use crate::assembler::util::{
    AssemblerReason,
    expect_left_brace,
    expect_right_brace,
    expect_newline,
    get_constant,
    get_label,
    get_register
};
use crate::assembler::util::AssemblerReason::{MissingRegion, UnknownInstruction};

fn instruction_base(op: &Opcode) -> u32 {
    match op {
        Op(key) => (*key as u32 & 0b111111) << 26,
        Func(key) => *key as u32 & 0b111111, // opcode: 0
        Special(key) => (*key as u32 & 0b111111) << 16 | (0b000001) << 26 // opcode: 1
    }
}

fn register_source(slot: RegisterSlot) -> u32 {
    slot.to_u32().unwrap()
}

struct InstructionBuilder(u32);

impl InstructionBuilder {
    fn from_op(op: &Opcode) -> InstructionBuilder {
        InstructionBuilder(instruction_base(op))
    }

    fn with_slot_offset<const OFFSET: u32>(mut self, slot: RegisterSlot) -> InstructionBuilder {
        self.0 &= 0b11111 << OFFSET;
        self.0 |= register_source(slot) << OFFSET;

        self
    }

    fn with_dest(self, slot: RegisterSlot) -> InstructionBuilder {
        self.with_slot_offset::<11>(slot)
    }

    fn with_temp(self, slot: RegisterSlot) -> InstructionBuilder {
        self.with_slot_offset::<16>(slot)
    }

    fn with_source(self, slot: RegisterSlot) -> InstructionBuilder {
        self.with_slot_offset::<21>(slot)
    }

    fn with_immediate(mut self, imm: u16) -> InstructionBuilder {
        self.0 &= 0xFF;
        self.0 |= imm as u32;

        self
    }

    fn with_sham(mut self, sham: u8) -> InstructionBuilder {
        self.0 &= 0b11111 << 6;
        self.0 |= (sham as u32) << 6;

        self
    }
}

struct EmitInstruction {
    instructions: Vec<(u32, Option<InstructionLabel>)>,
}

impl EmitInstruction {
    fn with(instruction: u32) -> EmitInstruction {
        EmitInstruction {
            instructions: vec![(instruction, None)],
        }
    }
}

fn do_register_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
                                                 -> Result<EmitInstruction, AssemblerReason> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;
    let temp = get_register(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_dest(dest)
        .with_source(source)
        .with_temp(temp)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_source_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T) -> Result<EmitInstruction, AssemblerReason> {
    let source = get_register(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_destination_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
                                                    -> Result<EmitInstruction, AssemblerReason> {
    let dest = get_register(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_dest(dest)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_inputs_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
                                               -> Result<EmitInstruction, AssemblerReason> {
    let source = get_register(iter)?;
    let temp = get_register(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .with_temp(temp)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_sham_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
                                             -> Result<EmitInstruction, AssemblerReason> {
    let dest = get_register(iter)?;
    let temp = get_register(iter)?;
    let sham = get_constant(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_dest(dest)
        .with_temp(temp)
        .with_sham(sham as u8)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_special_branch_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
                                                       -> Result<EmitInstruction, AssemblerReason> {
    let source = get_register(iter)?;
    let label = get_label(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .0;

    Ok(EmitInstruction { instructions: vec![(inst, Some(BranchLabel(label)))] })
}

fn do_immediate_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
                                                  -> Result<EmitInstruction, AssemblerReason> {
    let temp = get_register(iter)?;
    let source = get_register(iter)?;
    let constant = get_constant(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .with_temp(temp)
        .with_immediate(constant as u16)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_load_immediate_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
                                                       -> Result<EmitInstruction, AssemblerReason> {
    let temp = get_register(iter)?;
    let constant = get_constant(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_temp(temp)
        .with_immediate(constant as u16)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_jump_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
                                             -> Result<EmitInstruction, AssemblerReason> {
    let label = get_label(iter)?;

    let inst = InstructionBuilder::from_op(op).0;

    Ok(EmitInstruction { instructions: vec![(inst, Some(JumpLabel(label)))] })
}

fn do_branch_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
                                               -> Result<EmitInstruction, AssemblerReason> {
    let source = get_register(iter)?;
    let temp = get_register(iter)?;
    let label = get_label(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .with_temp(temp)
        .0;

    Ok(EmitInstruction { instructions: vec![(inst, Some(BranchLabel(label)))] })
}

fn do_branch_zero_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
                                                    -> Result<EmitInstruction, AssemblerReason> {
    let source = get_register(iter)?;
    let label = get_label(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .0;

    Ok(EmitInstruction { instructions: vec![(inst, Some(BranchLabel(label)))] })
}

fn do_parameterless_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, _: &mut T)
                                                      -> Result<EmitInstruction, AssemblerReason> {
    let inst = InstructionBuilder::from_op(op).0;

    Ok(EmitInstruction::with(inst))
}

fn do_offset_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
                                               -> Result<EmitInstruction, AssemblerReason> {
    let temp = get_register(iter)?;
    let constant = get_constant(iter)?;
    expect_left_brace(iter)?;
    let source = get_register(iter)?;
    expect_right_brace(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .with_temp(temp)
        .with_immediate(constant as u16)
        .0;

    Ok(EmitInstruction::with(inst))
}

pub fn do_instruction<'a, T: LexerSeekPeekable<'a>>(
    instruction: &'a str, iter: &mut T, builder: &mut BinaryBuilder, map: &HashMap<&str, &Instruction>
) -> Result<(), AssemblerReason> {
    let lowercase = instruction.to_lowercase();
    let lowercase_ref: &str = &lowercase;

    let Some(instruction) = map.get(&lowercase_ref) else {
        return Err(UnknownInstruction(instruction.to_string()))
    };

    let op = &instruction.opcode;

    let emit = match instruction.encoding {
        Encoding::Register => do_register_instruction(op, iter),
        Encoding::Source => do_source_instruction(op, iter),
        Encoding::Destination => do_destination_instruction(op, iter),
        Encoding::Inputs => do_inputs_instruction(op, iter),
        Encoding::Sham => do_sham_instruction(op, iter),
        Encoding::SpecialBranch => do_special_branch_instruction(op, iter),
        Encoding::Immediate => do_immediate_instruction(op, iter),
        Encoding::LoadImmediate => do_load_immediate_instruction(op, iter),
        Encoding::Jump => do_jump_instruction(op, iter),
        Encoding::Branch => do_branch_instruction(op, iter),
        Encoding::BranchZero => do_branch_zero_instruction(op, iter),
        Encoding::Parameterless => do_parameterless_instruction(op, iter),
        Encoding::Offset => do_offset_instruction(op, iter),
    }?;

    expect_newline(iter)?;

    let region = builder.region().ok_or(MissingRegion)?;

    for (word, branch) in emit.instructions {
        let offset = region.raw.data.len();

        if let Some(label) = branch {
            region.labels.insert(offset, label);
        }

        region.raw.data.write_u32::<LittleEndian>(word).unwrap();
    }

    Ok(())
}