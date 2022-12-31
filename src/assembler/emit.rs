use std::collections::HashMap;
use byteorder::{LittleEndian, WriteBytesExt};
use num_traits::ToPrimitive;
use Opcode::Algebra;
use crate::assembler::binary::{BinaryBuilder, InstructionLabel};
use crate::assembler::binary::InstructionLabel::{BranchLabel, JumpLabel, LowerLabel, UpperLabel};
use crate::assembler::instructions::{Encoding, Instruction, Opcode};
use crate::assembler::instructions::Opcode::{Op, Func, Special};
use crate::assembler::lexer_seek::{LexerSeek, LexerSeekPeekable};
use crate::assembler::registers::RegisterSlot;
use crate::assembler::registers::RegisterSlot::{AssemblerTemporary, Zero};
use crate::assembler::util::{expect_left_brace, expect_right_brace, expect_newline, get_constant, get_label, get_register, get_value, AssemblerReason, InstructionValue, maybe_get_value};
use crate::assembler::util::AssemblerReason::{MissingRegion, UnknownInstruction};

fn instruction_base(op: &Opcode) -> u32 {
    match op {
        Op(key) => (*key as u32 & 0b111111) << 26,
        Func(key) => *key as u32 & 0b111111, // opcode: 0
        Special(key) => (*key as u32 & 0b111111) << 16 | (1 << 26), // opcode: 1
        Algebra(key) => *key as u32 & 0b111111 | (28 << 26)
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
        self.0 &= !(0b11111 << OFFSET);
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
        self.0 &= 0xFFFF0000;
        self.0 |= imm as u32;

        self
    }

    fn with_sham(mut self, sham: u8) -> InstructionBuilder {
        self.0 &= !(0b11111 << 6);
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

fn load_immediate(constant: u64, into: RegisterSlot) -> Vec<u32> {
    let constant = constant as u32; // redefine
    let signed = constant as i32;

    if signed < 0x8000 && signed >= -0x8000 {
        let add = InstructionBuilder::from_op(&Op(9)) // addu
            .with_dest(into)
            .with_source(Zero)
            .with_immediate(constant as u16)
            .0;

        vec![add]
    } else {
        // This branch does NOT handle zero.
        let top = (constant & 0xFFFF0000) >> 16;
        let bottom = constant & 0x0000FFFF;

        let mut layer = Zero;
        let mut instructions = vec![];

        if top != 0 {
            let lui = InstructionBuilder::from_op(&Op(15))
                .with_temp(into)
                .with_immediate(top as u16)
                .0;

            layer = into;

            instructions.push(lui);
        }

        if bottom != 0 {
            let xori = InstructionBuilder::from_op(&Op(13))
                .with_temp(layer)
                .with_source(into)
                .with_immediate(bottom as u16)
                .0;

            instructions.push(xori);
        }

        instructions
    }
}

fn unpack_value(value: InstructionValue) -> (RegisterSlot, Vec<u32>) {
    match value {
        InstructionValue::Slot(slot) =>
            (slot, vec![]),
        InstructionValue::Literal(constant) =>
            (AssemblerTemporary, load_immediate(constant, AssemblerTemporary))
    }
}

fn emit_unpack_value(value: InstructionValue)
    -> (RegisterSlot, Vec<(u32, Option<InstructionLabel>)>) {
    let (slot, instructions) = unpack_value(value);

    (slot, instructions.into_iter().map(|value| (value, None)).collect())
}

fn do_register_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;
    let temp = get_value(iter)?;

    let (slot, mut instructions) = emit_unpack_value(temp);

    let inst = InstructionBuilder::from_op(op)
        .with_dest(dest)
        .with_source(source)
        .with_temp(slot)
        .0;

    instructions.push((inst, None));

    Ok(EmitInstruction { instructions })
}

fn do_source_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let source = get_register(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_destination_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let dest = get_register(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_dest(dest)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_inputs_instruction<'a, T: LexerSeekPeekable<'a>>(
    op: &Opcode, iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let first = get_register(iter)?;
    let second = get_register(iter)?;
    let div = maybe_get_value(iter);

    if let Some(value) = div {
        let (slot, mut instructions) = emit_unpack_value(value);

        let inst = InstructionBuilder::from_op(op)
            .with_source(second)
            .with_temp(slot)
            .0;

        let mflo = InstructionBuilder::from_op(&Func(18)) // mflo
            .with_dest(first)
            .0;

        instructions.append(&mut vec![(inst, None), (mflo, None)]);

        Ok(EmitInstruction { instructions })
    } else {
        let inst = InstructionBuilder::from_op(op)
            .with_source(first)
            .with_temp(second)
            .0;

        Ok(EmitInstruction::with(inst))
    }
}

fn do_sham_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
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

fn do_special_branch_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let source = get_register(iter)?;
    let label = get_label(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .0;

    Ok(EmitInstruction { instructions: vec![(inst, Some(BranchLabel(label)))] })
}

fn do_immediate_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
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

fn do_load_immediate_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let temp = get_register(iter)?;
    let constant = get_constant(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_temp(temp)
        .with_immediate(constant as u16)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_jump_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let label = get_label(iter)?;

    let inst = InstructionBuilder::from_op(op).0;

    Ok(EmitInstruction { instructions: vec![(inst, Some(JumpLabel(label)))] })
}

fn do_branch_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let source = get_register(iter)?;
    let temp = get_value(iter)?;
    let label = get_label(iter)?;

    let (slot, mut instructions) = emit_unpack_value(temp);

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .with_temp(slot)
        .0;

    instructions.push((inst, Some(BranchLabel(label))));

    Ok(EmitInstruction { instructions })
}

fn do_branch_zero_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let source = get_register(iter)?;
    let label = get_label(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .0;

    Ok(EmitInstruction { instructions: vec![(inst, Some(BranchLabel(label)))] })
}

fn do_parameterless_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, _: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let inst = InstructionBuilder::from_op(op).0;

    Ok(EmitInstruction::with(inst))
}

fn do_offset_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
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

fn do_abs_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;

    // Instruction Pattern from MARS (e.g. branchless)
    let shift = InstructionBuilder::from_op(&Func(3)) // sra
        .with_dest(AssemblerTemporary)
        .with_temp(source)
        .with_immediate(31)
        .0;

    let xor = InstructionBuilder::from_op(&Func(38)) // xor
        .with_dest(dest)
        .with_temp(AssemblerTemporary)
        .with_source(source)
        .0;

    let sub = InstructionBuilder::from_op(&Func(35)) // subu
        .with_dest(dest)
        .with_temp(AssemblerTemporary)
        .with_source(dest)
        .0;

    let instructions = vec![(shift, None), (xor, None), (sub, None)];

    Ok(EmitInstruction { instructions })
}

fn do_blt_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let source = get_register(iter)?;
    let temp = get_value(iter)?;
    let label = get_label(iter)?;

    let (slot, mut instructions) = emit_unpack_value(temp);

    let slt = InstructionBuilder::from_op(&Func(42)) // slt
        .with_source(source)
        .with_temp(slot)
        .with_dest(AssemblerTemporary)
        .0;

    let bne = InstructionBuilder::from_op(&Op(5)) // bne
        .with_source(AssemblerTemporary)
        .with_temp(Zero)
        .0;

    instructions.append(&mut vec![(slt, None), (bne, Some(BranchLabel(label)))]);

    Ok(EmitInstruction { instructions })
}

fn do_bgt_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let source = get_register(iter)?;
    let temp = get_value(iter)?;
    let label = get_label(iter)?;

    let (slot, mut instructions) = emit_unpack_value(temp);

    let slt = InstructionBuilder::from_op(&Func(42)) // slt
        .with_source(slot)
        .with_temp(source)
        .with_dest(AssemblerTemporary)
        .0;

    let bne = InstructionBuilder::from_op(&Op(5)) // bne
        .with_source(AssemblerTemporary)
        .with_temp(Zero)
        .0;

    instructions.append(&mut vec![(slt, None), (bne, Some(BranchLabel(label)))]);

    Ok(EmitInstruction { instructions })
}

fn do_ble_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let source = get_register(iter)?;
    let temp = get_value(iter)?;
    let label = get_label(iter)?;

    let (slot, mut instructions) = emit_unpack_value(temp);

    let slt = InstructionBuilder::from_op(&Func(42)) // slt
        .with_source(slot)
        .with_temp(source)
        .with_dest(AssemblerTemporary)
        .0;

    let beq = InstructionBuilder::from_op(&Op(4)) // slt
        .with_source(AssemblerTemporary)
        .with_temp(Zero)
        .0;

    instructions.append(&mut vec![(slt, None), (beq, Some(BranchLabel(label)))]);

    Ok(EmitInstruction { instructions })
}

fn do_bge_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let source = get_register(iter)?;
    let temp = get_value(iter)?;
    let label = get_label(iter)?;

    let (slot, mut instructions) = emit_unpack_value(temp);

    let slt = InstructionBuilder::from_op(&Func(42)) // slt
        .with_source(source)
        .with_temp(slot)
        .with_dest(AssemblerTemporary)
        .0;

    let beq = InstructionBuilder::from_op(&Op(4)) // slt
        .with_source(AssemblerTemporary)
        .with_temp(Zero)
        .0;

    instructions.append(&mut vec![(slt, None), (beq, Some(BranchLabel(label)))]);

    Ok(EmitInstruction { instructions })
}

fn do_neg_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;

    let sub = InstructionBuilder::from_op(&Func(34)) // sub
        .with_dest(dest)
        .with_source(Zero)
        .with_temp(source)
        .0;

    Ok(EmitInstruction::with(sub))
}

fn do_negu_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;

    let subu = InstructionBuilder::from_op(&Func(35)) // subu
        .with_dest(dest)
        .with_source(Zero)
        .with_temp(source)
        .0;

    Ok(EmitInstruction::with(subu))
}

fn do_not_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;

    let nor = InstructionBuilder::from_op(&Func(39))
        .with_dest(dest)
        .with_source(source)
        .with_temp(Zero)
        .0;

    Ok(EmitInstruction::with(nor))
}

fn do_li_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let dest = get_register(iter)?;
    let constant = get_constant(iter)?;

    let instructions = load_immediate(constant, dest).into_iter()
        .map(|inst| (inst, None))
        .collect();

    Ok(EmitInstruction { instructions })
}

fn do_la_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    // Load Address may not know the label location yet.
    // So we will never optimize away the size of this instruction,
    // as this might change the label location.

    let dest = get_register(iter)?;
    let label_upper = get_label(iter)?;
    let label_lower = label_upper.clone();

    let lui = InstructionBuilder::from_op(&Op(15))
        .with_temp(dest)
        .0;

    let ori = InstructionBuilder::from_op(&Op(13))
        .with_temp(dest)
        .with_source(dest)
        .0;

    let instructions = vec![
        (lui, Some(UpperLabel(label_upper))),
        (ori, Some(LowerLabel(label_lower)))
    ];

    Ok(EmitInstruction { instructions })
}

fn do_move_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;

    let addu = InstructionBuilder::from_op(&Func(33)) // addu
        .with_dest(dest)
        .with_temp(Zero)
        .with_source(source)
        .0;

    Ok(EmitInstruction::with(addu))
}

fn do_sge_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;
    let temp = get_register(iter)?;

    // s >= t -> !(s < t)

    let slt = InstructionBuilder::from_op(&Func(42)) // slt
        .with_dest(dest)
        .with_source(source)
        .with_temp(temp)
        .0;

    let xori = InstructionBuilder::from_op(&Op(14)) // xori
        .with_temp(dest)
        .with_source(dest)
        .with_immediate(1)
        .0;

    let instructions = vec![(slt, None), (xori, None)];

    Ok(EmitInstruction { instructions })
}

fn do_sgt_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;
    let temp = get_register(iter)?;

    let slt = InstructionBuilder::from_op(&Func(42)) // slt
        .with_dest(dest)
        .with_source(temp)
        .with_temp(source)
        .0;

    Ok(EmitInstruction::with(slt))
}

fn do_b_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let label = get_label(iter)?;

    let beq = InstructionBuilder::from_op(&Op(4)) // beq
        .with_source(Zero)
        .with_temp(Zero)
        .0;

    let instructions = vec![(beq, Some(BranchLabel(label)))];

    Ok(EmitInstruction { instructions })
}

// MARS seems to load the instruction itself like `li`. I'm not sure about this! Do it yourself!
fn do_subi_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let dest = get_register(iter)?;
    let temp = get_register(iter)?;
    let constant = get_constant(iter)?;

    let addi = InstructionBuilder::from_op(&Op(8)) // addi
        .with_source(temp)
        .with_temp(dest)
        .with_immediate((-(constant as i16)) as u16)
        .0;

    Ok(EmitInstruction::with(addi))
}

fn do_subiu_instruction<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Result<EmitInstruction, AssemblerReason> {
    let dest = get_register(iter)?;
    let temp = get_register(iter)?;
    let constant = get_constant(iter)?;

    let addiu = InstructionBuilder::from_op(&Op(9)) // addiu
        .with_source(temp)
        .with_temp(dest)
        .with_immediate((-(constant as i16)) as u16)
        .0;

    Ok(EmitInstruction::with(addiu))
}

fn dispatch_pseudo<'a, T: LexerSeekPeekable<'a>>(
    instruction: &str, iter: &mut T
) -> Result<Option<EmitInstruction>, AssemblerReason> {
    Ok(Some(match instruction {
        "abs" => do_abs_instruction(iter)?,
        "blt" => do_blt_instruction(iter)?,
        "bgt" => do_bgt_instruction(iter)?,
        "ble" => do_ble_instruction(iter)?,
        "bge" => do_bge_instruction(iter)?,
        "neg" => do_neg_instruction(iter)?,
        "negu" => do_negu_instruction(iter)?,
        "not" => do_not_instruction(iter)?,
        "li" => do_li_instruction(iter)?,
        "la" => do_la_instruction(iter)?,
        "move" => do_move_instruction(iter)?,
        "sge" => do_sge_instruction(iter)?,
        "sgt" => do_sgt_instruction(iter)?,
        "b" => do_b_instruction(iter)?,
        "subi" => do_subi_instruction(iter)?,
        "subiu" => do_subiu_instruction(iter)?,
        _ => return Ok(None)
    }))
}

fn dispatch_instruction<'a, T: LexerSeekPeekable<'a>>(
    instruction: &str, iter: &mut T, map: &HashMap<&str, &Instruction>
) -> Result<EmitInstruction, AssemblerReason> {
    let Some(instruction) = map.get(&instruction) else {
        return dispatch_pseudo(instruction, iter)?
            .ok_or_else(|| UnknownInstruction(instruction.to_string()));
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

    Ok(emit)
}

pub fn do_instruction<'a, T: LexerSeekPeekable<'a>>(
    instruction: &str, iter: &mut T,
    builder: &mut BinaryBuilder, map: &HashMap<&str, &Instruction>
) -> Result<(), AssemblerReason> {
    let lowercase = instruction.to_lowercase();

    let emit = dispatch_instruction(&lowercase, iter, map)?;

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