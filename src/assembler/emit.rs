use std::collections::HashMap;
use byteorder::{LittleEndian, WriteBytesExt};
use num_traits::ToPrimitive;
use Opcode::Algebra;
use crate::assembler::binary_builder::{BinaryBuilderLabel, InstructionLabel};
use crate::assembler::binary_builder::InstructionLabel::{BranchLabel, JumpLabel, LowerLabel, UpperLabel};
use crate::assembler::instructions::{Encoding, Instruction, Opcode};
use crate::assembler::instructions::Opcode::{Op, Func, Special};
use crate::assembler::registers::RegisterSlot;
use crate::assembler::registers::RegisterSlot::{AssemblerTemporary, Zero};
use crate::assembler::assembler_util::{get_constant, get_label, get_register, get_value, get_offset_or_label, maybe_get_value, InstructionValue, OffsetOrLabel, AssemblerError, default_start};
use crate::assembler::assembler_util::AssemblerReason::{MissingRegion, UnknownInstruction};
use crate::assembler::binary::AddressLabel;
use crate::assembler::binary_builder::BinaryBuilder;
use crate::assembler::cursor::LexerCursor;

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

type InstructionPair = (u32, Option<InstructionLabel>);

struct EmitInstruction {
    instructions: Vec<InstructionPair>,
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
        let add = InstructionBuilder::from_op(&Op(9)) // addiu
            .with_temp(into)
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
                .with_temp(into)
                .with_source(layer)
                .with_immediate(bottom as u16)
                .0;

            instructions.push(xori);
        }

        instructions
    }
}

fn make_label(label: AddressLabel, dest: RegisterSlot) -> Vec<InstructionPair> {
    // Load Address may not know the label location yet.
    // So we will never optimize away the size of this instruction,
    // as this might change the label location.

    let label_upper = label.clone();
    let label_lower = label;

    let lui = InstructionBuilder::from_op(&Op(15))
        .with_temp(dest)
        .0;

    let ori = InstructionBuilder::from_op(&Op(13))
        .with_temp(dest)
        .with_source(dest)
        .0;

    vec![
        (lui, Some(UpperLabel(label_upper))),
        (ori, Some(LowerLabel(label_lower)))
    ]
}

fn make_offset_or_label(offset: OffsetOrLabel) -> (u16, RegisterSlot, Vec<InstructionPair>) {
    match offset {
        OffsetOrLabel::Offset(value, register) => (value as u16, register, vec![]),
        OffsetOrLabel::Address(label) => {
            let instructions = make_label(label, AssemblerTemporary);

            (0, AssemblerTemporary, instructions)
        }
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

fn do_register_instruction(
    op: &Opcode, iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
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

fn do_register_shift_instruction(
    op: &Opcode, iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let temp = get_register(iter)?;
    let source = get_register(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_dest(dest)
        .with_source(source)
        .with_temp(temp)
        .0;

    let instructions = vec![(inst, None)];

    Ok(EmitInstruction { instructions })
}

fn do_source_instruction(
    op: &Opcode, iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let source = get_register(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_destination_instruction(
    op: &Opcode, iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_dest(dest)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_inputs_instruction(
    op: &Opcode, iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
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

fn do_sham_instruction(
    op: &Opcode, iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
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

fn do_special_branch_instruction(
    op: &Opcode, iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let source = get_register(iter)?;
    let label = get_label(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .0;

    Ok(EmitInstruction { instructions: vec![(inst, Some(BranchLabel(label)))] })
}

fn do_immediate_instruction(
    op: &Opcode, iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
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

fn do_load_immediate_instruction(
    op: &Opcode, iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let temp = get_register(iter)?;
    let constant = get_constant(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_temp(temp)
        .with_immediate(constant as u16)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_jump_instruction(
    op: &Opcode, iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let label = get_label(iter)?;

    let inst = InstructionBuilder::from_op(op).0;

    Ok(EmitInstruction { instructions: vec![(inst, Some(JumpLabel(label)))] })
}

fn do_branch_instruction(
    op: &Opcode, iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
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

fn do_branch_zero_instruction(
    op: &Opcode, iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let source = get_register(iter)?;
    let label = get_label(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .0;

    Ok(EmitInstruction { instructions: vec![(inst, Some(BranchLabel(label)))] })
}

fn do_parameterless_instruction(
    op: &Opcode, _: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let inst = InstructionBuilder::from_op(op).0;

    Ok(EmitInstruction::with(inst))
}

fn do_offset_instruction(
    op: &Opcode, iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let temp = get_register(iter)?;

    // let constant = get_constant(iter)?;
    // expect_left_brace(iter)?;
    // let source = get_register(iter)?;
    // expect_right_brace(iter)?;

    let offset = get_offset_or_label(iter)?;

    let (immediate, register, mut instructions) = make_offset_or_label(offset);

    let inst = InstructionBuilder::from_op(op)
        .with_source(register)
        .with_temp(temp)
        .with_immediate(immediate)
        .0;

    instructions.push((inst, None));

    Ok(EmitInstruction { instructions })
}

fn do_nop_instruction(
    _: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let instruction = InstructionBuilder::from_op(&Func(0))
        .0;

    Ok(EmitInstruction::with(instruction))
}

fn do_abs_instruction(
    iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
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

fn do_branch_custom_instruction(
    iter: &mut LexerCursor,
    greater_than: bool,
    result_true: bool,
    unsigned: bool
) -> Result<EmitInstruction, AssemblerError> {
    let source = get_register(iter)?;
    let temp = get_value(iter)?;
    let label = get_label(iter)?;

    let (slot, mut instructions) = emit_unpack_value(temp);

    let (first, second) = if greater_than { (slot, source) } else { (source, slot) };
    let set_op = if unsigned { &Func(41) } else { &Func(42) };
    let branch_op = if result_true { &Op(5) } else { &Op(4) };

    let compare = InstructionBuilder::from_op(set_op) // slt
        .with_source(first)
        .with_temp(second)
        .with_dest(AssemblerTemporary)
        .0;

    let branch = InstructionBuilder::from_op(branch_op) // bne
        .with_source(AssemblerTemporary)
        .with_temp(Zero)
        .0;

    instructions.append(&mut vec![(compare, None), (branch, Some(BranchLabel(label)))]);

    Ok(EmitInstruction { instructions })
}

fn do_set_custom_instruction(
    iter: &mut LexerCursor,
    greater_than: bool,
    result_true: bool,
    unsigned: bool
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;
    let temp = get_value(iter)?;

    let (slot, mut instructions) = emit_unpack_value(temp);

    let (first, second) = if greater_than { (source, slot) } else { (slot, source) };
    let set_op = if unsigned { &Func(41) } else { &Func(42) };

    let set = InstructionBuilder::from_op(set_op)
        .with_dest(dest)
        .with_source(first)
        .with_temp(second)
        .0;

    instructions.push((set, None));

    if !result_true {
        let xori = InstructionBuilder::from_op(&Op(14)) // xori
            .with_temp(dest)
            .with_source(dest)
            .with_immediate(1)
            .0;

        instructions.push((xori, None))
    }

    Ok(EmitInstruction { instructions })
}

fn do_seq_instruction(
    iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;
    let temp = get_value(iter)?;

    let (slot, mut instructions) = emit_unpack_value(temp);

    let subu = InstructionBuilder::from_op(&Func(35))
        .with_dest(dest)
        .with_source(source)
        .with_temp(slot)
        .0;

    let sltu = InstructionBuilder::from_op(&Func(41))
        .with_dest(dest)
        .with_source(source)
        .with_temp(Zero)
        .0;

    let xori = InstructionBuilder::from_op(&Op(14))
        .with_dest(dest)
        .with_source(dest)
        .with_immediate(1)
        .0;

    instructions.extend([(subu, None), (sltu, None), (xori, None)]);

    Ok(EmitInstruction { instructions })
}

fn do_sne_instruction(
    iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;
    let temp = get_value(iter)?;

    let (slot, mut instructions) = emit_unpack_value(temp);

    let subu = InstructionBuilder::from_op(&Func(35))
        .with_dest(dest)
        .with_source(source)
        .with_temp(slot)
        .0;

    let sltu = InstructionBuilder::from_op(&Func(41))
        .with_dest(dest)
        .with_source(source)
        .with_temp(Zero)
        .0;

    instructions.extend([(subu, None), (sltu, None)]);

    Ok(EmitInstruction { instructions })
}

fn do_neg_instruction(
    iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;

    let sub = InstructionBuilder::from_op(&Func(34)) // sub
        .with_dest(dest)
        .with_source(Zero)
        .with_temp(source)
        .0;

    Ok(EmitInstruction::with(sub))
}

fn do_negu_instruction(
    iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;

    let subu = InstructionBuilder::from_op(&Func(35)) // subu
        .with_dest(dest)
        .with_source(Zero)
        .with_temp(source)
        .0;

    Ok(EmitInstruction::with(subu))
}

fn do_not_instruction(
    iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;

    let nor = InstructionBuilder::from_op(&Func(39))
        .with_dest(dest)
        .with_source(source)
        .with_temp(Zero)
        .0;

    Ok(EmitInstruction::with(nor))
}

fn do_li_instruction(
    iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let constant = get_constant(iter)?;

    let instructions = load_immediate(constant, dest).into_iter()
        .map(|inst| (inst, None))
        .collect();

    Ok(EmitInstruction { instructions })
}

fn do_la_instruction(
    iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let label = get_label(iter)?;

    let instructions = make_label(label, dest);

    Ok(EmitInstruction { instructions })
}

fn do_move_instruction(
    iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;

    let addu = InstructionBuilder::from_op(&Func(33)) // addu
        .with_dest(dest)
        .with_temp(Zero)
        .with_source(source)
        .0;

    Ok(EmitInstruction::with(addu))
}

fn do_b_instruction(
    iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
    let label = get_label(iter)?;

    let beq = InstructionBuilder::from_op(&Op(4)) // beq
        .with_source(Zero)
        .with_temp(Zero)
        .0;

    let instructions = vec![(beq, Some(BranchLabel(label)))];

    Ok(EmitInstruction { instructions })
}

// MARS seems to load the instruction itself like `li`. I'm not sure about this! Do it yourself!
fn do_subi_instruction(
    iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
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

fn do_subiu_instruction(
    iter: &mut LexerCursor
) -> Result<EmitInstruction, AssemblerError> {
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

fn dispatch_pseudo(
    instruction: &str, iter: &mut LexerCursor
) -> Result<Option<EmitInstruction>, AssemblerError> {
    Ok(Some(match instruction {
        "nop" => do_nop_instruction(iter),
        "abs" => do_abs_instruction(iter),
        "blt" => do_branch_custom_instruction(iter, false, true, false),
        "bgt" => do_branch_custom_instruction(iter, true, true, false),
        "ble" => do_branch_custom_instruction(iter, true, false, false),
        "bge" => do_branch_custom_instruction(iter, false, false, false),
        "bltu" => do_branch_custom_instruction(iter, false, true, true),
        "bgtu" => do_branch_custom_instruction(iter, true, true, true),
        "bleu" => do_branch_custom_instruction(iter, true, false, true),
        "bgeu" => do_branch_custom_instruction(iter, false, false, true),
        "sge" => do_set_custom_instruction(iter, false, false, false),
        "sgt" => do_set_custom_instruction(iter, true, true, false),
        "sle" => do_set_custom_instruction(iter, true, false, false),
        "sgeu" => do_set_custom_instruction(iter, false, false, true),
        "sgtu" => do_set_custom_instruction(iter, true, true, true),
        "sleu" => do_set_custom_instruction(iter, true, false, true),
        "beqz" => do_branch_zero_instruction(&Op(4), iter),
        "bnez" => do_branch_zero_instruction(&Op(5), iter),
        "seq" => do_seq_instruction(iter),
        "sne" => do_sne_instruction(iter),
        "neg" => do_neg_instruction(iter),
        "negu" => do_negu_instruction(iter),
        "not" => do_not_instruction(iter),
        "li" => do_li_instruction(iter),
        "la" => do_la_instruction(iter),
        "move" => do_move_instruction(iter),
        "b" => do_b_instruction(iter),
        "subi" => do_subi_instruction(iter),
        "subiu" => do_subiu_instruction(iter),
        _ => return Ok(None)
    }?))
}

fn dispatch_instruction(
    instruction: &str, iter: &mut LexerCursor, map: &HashMap<&str, &Instruction>
) -> Result<EmitInstruction, AssemblerError> {
    let Some(instruction) = map.get(&instruction) else {
        return dispatch_pseudo(instruction, iter)?
            .ok_or_else(|| AssemblerError {
                start: None,
                reason: UnknownInstruction(instruction.to_string())
            });
    };

    let op = &instruction.opcode;

    let emit = match instruction.encoding {
        Encoding::Register => do_register_instruction(op, iter),
        Encoding::RegisterShift => do_register_shift_instruction(op, iter),
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

    Ok(emit)
}

pub fn do_instruction(
    instruction: &str, start: usize, iter: &mut LexerCursor,
    builder: &mut BinaryBuilder, map: &HashMap<&str, &Instruction>
) -> Result<(), AssemblerError> {
    let lowercase = instruction.to_lowercase();

    let emit = dispatch_instruction(&lowercase, iter, map)
        .map_err(default_start(start))?;

    let mut breakpoints = HashMap::new();

    let region = builder.region()
        .ok_or(AssemblerError { start: Some(start), reason: MissingRegion })?;

    for (word, branch) in emit.instructions {
        let pc = region.raw.address + region.raw.data.len() as u32;
        breakpoints.insert(pc, start);

        let offset = region.raw.data.len();

        if let Some(label) = branch {
            region.labels.push(BinaryBuilderLabel { offset, start, label });
        }

        region.raw.data.write_u32::<LittleEndian>(word).unwrap();
    }

    builder.breakpoints.extend(breakpoints);

    Ok(())
}
