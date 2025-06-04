use std::collections::HashMap;
use byteorder::{LittleEndian, WriteBytesExt};
use titan_shared::assembler::binary::BinaryBreakpoint;
use titan_shared::assembler::lexer::Location;
use crate::assembler::binary_builder::{BinaryBuilder, BinaryBuilderLabel, InstructionLabel};
use crate::assembler::binary_builder::InstructionLabelKind::{Branch, JumpAndLink};
use crate::assembler::emit::InstructionValue::{Base, Compressed};
use crate::assembler::instruction_builder::InstructionBuilder;
use crate::assembler::instructions::{Encoding, Instruction, Opcode};
use crate::assembler::utilities::{default_start, get_constant_in_range, get_label, get_offset, get_register, pc_for_region, AssemblerError, TokenCursor};
use crate::assembler::utilities::AssemblerReason::{MissingRegion, UnknownInstruction};

enum InstructionValue {
    Base(u32),
    Compressed(u16),
}

type InstructionPair = (InstructionValue, Option<InstructionLabel>);

struct EmitInstruction {
    instructions: Vec<InstructionPair>,
}

impl EmitInstruction {
    fn with(instruction: u32) -> EmitInstruction {
        EmitInstruction {
            instructions: vec![(Base(instruction), None)],
        }
    }

    fn with_compressed(instruction: u16) -> EmitInstruction {
        EmitInstruction {
            instructions: vec![(Compressed(instruction), None)]
        }
    }
}

// fn load_immediate(constant: u64, into: RegisterSlot) -> Vec<u32> {
//     let constant = constant as u32; // redefine
//     let signed = constant as i32;
//
//     if (-0x8000..0x8000).contains(&signed) {
//         let add = InstructionBuilder::from_op(&Op(9)) // addiu
//             .with_temp(into)
//             .with_source(Zero)
//             .with_immediate(constant as u16)
//             .0;
//
//         vec![add]
//     } else {
//         // This branch does NOT handle zero.
//         let top = (constant & 0xFFFF0000) >> 16;
//         let bottom = constant & 0x0000FFFF;
//
//         let mut layer = Zero;
//         let mut instructions = vec![];
//
//         if top != 0 {
//             let lui = InstructionBuilder::from_op(&Op(15))
//                 .with_temp(into)
//                 .with_immediate(top as u16)
//                 .0;
//
//             layer = into;
//
//             instructions.push(lui);
//         }
//
//         if bottom != 0 {
//             let xori = InstructionBuilder::from_op(&Op(13))
//                 .with_temp(into)
//                 .with_source(layer)
//                 .with_immediate(bottom as u16)
//                 .0;
//
//             instructions.push(xori);
//         }
//
//         instructions
//     }
// }
//
// fn make_label(label: AddressLabel, dest: RegisterSlot) -> Vec<InstructionPair> {
//     // Load Address may not know the label location yet.
//     // So we will never optimize away the size of this instruction,
//     // as this might change the label location.
//
//     match label {
//         AddressLabel::Constant(constant) => load_immediate(constant, dest)
//             .into_iter()
//             .map(|instruction| (instruction, None))
//             .collect(),
//         AddressLabel::Label(_) => {
//             let label_upper = label.clone();
//             let label_lower = label;
//
//             let lui = InstructionBuilder::from_op(&Op(15)).with_temp(dest).0;
//
//             let ori = InstructionBuilder::from_op(&Op(13))
//                 .with_temp(dest)
//                 .with_source(dest)
//                 .0;
//
//             vec![
//                 (
//                     lui,
//                     Some(InstructionLabel {
//                         label: label_upper,
//                         kind: Upper,
//                     }),
//                 ),
//                 (
//                     ori,
//                     Some(InstructionLabel {
//                         label: label_lower,
//                         kind: Lower,
//                     }),
//                 ),
//             ]
//         }
//     }
// }
//
// fn make_offset_or_label(offset: OffsetOrLabel) -> (u16, RegisterSlot, Vec<InstructionPair>) {
//     match offset {
//         OffsetOrLabel::Offset(label, register) => match label {
//             AddressLabel::Constant(constant)
//             if (constant as i64) <= 0x7fff && (constant as i64) >= -0x8000 =>
//                 {
//                     (constant as u16, register, vec![])
//                 }
//             _ => {
//                 let mut instructions = make_label(label, AssemblerTemporary);
//
//                 let add = InstructionBuilder::from_op(&Func(32))
//                     .with_dest(AssemblerTemporary)
//                     .with_source(AssemblerTemporary)
//                     .with_temp(register)
//                     .0;
//
//                 instructions.push((add, None));
//
//                 (0, AssemblerTemporary, instructions)
//             }
//         },
//         OffsetOrLabel::Label(label) => {
//             let instructions = make_label(label, AssemblerTemporary);
//
//             (0, AssemblerTemporary, instructions)
//         }
//     }
// }
//
// fn unpack_value(value: InstructionValue) -> (RegisterSlot, Vec<u32>) {
//     match value {
//         InstructionValue::Slot(slot) => (slot, vec![]),
//         InstructionValue::Literal(constant) => (
//             AssemblerTemporary,
//             load_immediate(constant, AssemblerTemporary),
//         ),
//     }
// }
//
// fn emit_unpack_value(
//     value: InstructionValue,
// ) -> (RegisterSlot, Vec<(u32, Option<InstructionLabel>)>) {
//     let (slot, instructions) = unpack_value(value);
//
//     (
//         slot,
//         instructions
//             .into_iter()
//             .map(|value| (value, None))
//             .collect(),
//     )
// }

fn do_upper_instruction(
    op: &Opcode,
    iter: &mut TokenCursor,
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let immediate = get_constant_in_range(iter, 0 ..= 0xfffff)?;

    // Even though these are sign extended on RV64
    // It doesn't really make any sense for these to be "signed numbers."
    let inst = InstructionBuilder::from_op(op)
        .with_rd(dest)
        .with_upper_imm(immediate as u32)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_jump_immediate_instruction(
    op: &Opcode,
    iter: &mut TokenCursor,
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let label = get_label(iter)?;

    // Even though these are sign extended on RV64
    // It doesn't really make any sense for these to be "signed numbers."
    let inst = InstructionBuilder::from_op(op)
        .with_rd(dest)
        .0;

    Ok(EmitInstruction {
        instructions: vec![
            (Base(inst), Some(InstructionLabel { kind: JumpAndLink, label }))
        ]
    })
}

fn do_branch_instruction(
    op: &Opcode,
    iter: &mut TokenCursor,
) -> Result<EmitInstruction, AssemblerError> {
    let src1 = get_register(iter)?;
    let src2 = get_register(iter)?;
    let label = get_label(iter)?;

    // Even though these are sign extended on RV64
    // It doesn't really make any sense for these to be "signed numbers."
    let inst = InstructionBuilder::from_op(op)
        .with_rs1(src1)
        .with_rs2(src2)
        .0;

    Ok(EmitInstruction {
        instructions: vec![
            (Base(inst), Some(InstructionLabel { kind: Branch, label }))
        ]
    })
}

fn do_offset_load_instruction(
    op: &Opcode,
    iter: &mut TokenCursor,
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;

    let offset = get_offset(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_rd(dest)
        .with_rs1(offset.slot)
        .with_normal_imm(offset.immediate)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_offset_store_instruction(
    op: &Opcode,
    iter: &mut TokenCursor,
) -> Result<EmitInstruction, AssemblerError> {
    let src = get_register(iter)?;

    let offset = get_offset(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_rs1(offset.slot)
        .with_rs2(src)
        .with_store_imm(offset.immediate)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_arithmetic_immediate_instruction(
    op: &Opcode,
    iter: &mut TokenCursor,
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let src = get_register(iter)?;
    // All arithmetic immediate instructions have signed immediate (including ORI/ANDI/XORI).
    // SRC: Sail https://riscv-software-src.github.io/riscv-unified-db/manual/html/isa/isa_20240411/insts/andi.html
    let immediate = get_constant_in_range(iter, -0x800 ..= 0x7ff)? as i16;

    let inst = InstructionBuilder::from_op(op)
        .with_rd(dest)
        .with_rs1(src)
        .with_normal_imm(immediate)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_sham_instruction(
    op: &Opcode,
    iter: &mut TokenCursor,
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let src = get_register(iter)?;
    // All arithmetic immediate instructions have signed immediate (including ORI/ANDI/XORI).
    // SRC: Sail https://riscv-software-src.github.io/riscv-unified-db/manual/html/isa/isa_20240411/insts/andi.html
    let sham = get_constant_in_range(iter, 0 ..= 31)? as u8;

    let inst = InstructionBuilder::from_op(op)
        .with_rd(dest)
        .with_rs1(src)
        .with_sham(sham)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_registers_instruction(
    op: &Opcode,
    iter: &mut TokenCursor,
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let src1 = get_register(iter)?;
    let src2 = get_register(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_rd(dest)
        .with_rs1(src1)
        .with_rs2(src2)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_single_instruction(
    op: &Opcode,
    _iter: &mut TokenCursor,
) -> Result<EmitInstruction, AssemblerError> {
    // No params.

    let inst = InstructionBuilder::from_op(op)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn dispatch_pseudo(
    instruction: &str,
    iter: &mut TokenCursor,
) -> Result<Option<EmitInstruction>, AssemblerError> {
    panic!()
    // Ok(Some(match instruction {
        // "nop" => do_nop_instruction(iter),
        // "li" => do_li_instruction(iter),
        // "la" => do_la_instruction(iter),
        // // no support for "load/store local/global" yet
        // // "abs" => do_abs_instruction(iter),
        // "blt" => do_branch_custom_instruction(iter, false, true, false),
        // "bgt" => do_branch_custom_instruction(iter, true, true, false),
        // "ble" => do_branch_custom_instruction(iter, true, false, false),
        // "bge" => do_branch_custom_instruction(iter, false, false, false),
        // "bltu" => do_branch_custom_instruction(iter, false, true, true),
        // "bgtu" => do_branch_custom_instruction(iter, true, true, true),
        // "bleu" => do_branch_custom_instruction(iter, true, false, true),
        // "bgeu" => do_branch_custom_instruction(iter, false, false, true),
        // "sge" => do_set_custom_instruction(iter, false, false, false),
        // "sgt" => do_set_custom_instruction(iter, true, true, false),
        // "sle" => do_set_custom_instruction(iter, true, false, false),
        // "sgeu" => do_set_custom_instruction(iter, false, false, true),
        // "sgtu" => do_set_custom_instruction(iter, true, true, true),
        // "sleu" => do_set_custom_instruction(iter, true, false, true),
        // "seq" => do_seq_instruction(iter),
        // "sne" => do_sne_instruction(iter),
        // "neg" => do_neg_instruction(iter),
        // "negu" => do_negu_instruction(iter),
        // "not" => do_not_instruction(iter),
        // "move" => do_move_instruction(iter),
        // "b" => do_b_instruction(iter),
        // "subi" => do_subi_instruction(iter),
        // "subiu" => do_subiu_instruction(iter),
        // _ => return Ok(None),
    // }?))
}

fn dispatch_instruction(
    instruction: &str,
    iter: &mut TokenCursor,
    map: &HashMap<&str, &Instruction>,
) -> Result<EmitInstruction, AssemblerError> {
    let Some(instruction) = map.get(&instruction) else {
        return dispatch_pseudo(instruction, iter)?.ok_or_else(|| AssemblerError {
            location: None,
            reason: UnknownInstruction(instruction.to_string()),
        });
    };

    let op = &instruction.opcode;

    let emit = match &instruction.encoding {
        Encoding::UpperImmediate => do_upper_instruction(op, iter),
        Encoding::JumpImmediate => do_jump_immediate_instruction(op, iter),
        Encoding::Branch => do_branch_instruction(op, iter),
        Encoding::OffsetLoad => do_offset_load_instruction(op, iter),
        Encoding::OffsetStore => do_offset_store_instruction(op, iter),
        Encoding::ArithmeticImmediate => do_arithmetic_immediate_instruction(op, iter),
        Encoding::Sham => do_sham_instruction(op, iter),
        Encoding::Registers => do_registers_instruction(op, iter),
        Encoding::Single => do_single_instruction(op, iter),
        _ => panic!(),
    }?;

    Ok(emit)
}

pub fn do_instruction(
    instruction: &str,
    location: Location,
    iter: &mut TokenCursor,
    builder: &mut BinaryBuilder,
    map: &HashMap<&str, &Instruction>,
) -> Result<(), AssemblerError> {
    let lowercase = instruction.to_lowercase();

    let emit = dispatch_instruction(&lowercase, iter, map).map_err(default_start(location))?;

    let region = builder.region().ok_or(AssemblerError {
        location: Some(location),
        reason: MissingRegion,
    })?;

    let mut breakpoint = BinaryBreakpoint {
        location,
        pcs: vec![],
    };

    for (instruction, branch) in emit.instructions {
        let pc = pc_for_region(&region.raw, Some(location))?;

        breakpoint.pcs.push(pc);

        let offset = region.raw.data.len();

        if let Some(label) = branch {
            region.labels.push(BinaryBuilderLabel {
                offset,
                location,
                label,
            });
        }

        match instruction {
            Base(word) => region.raw.data.write_u32::<LittleEndian>(word).unwrap(),
            Compressed(compressed) => region.raw.data.write_u16::<LittleEndian>(compressed).unwrap(),
        }
    }

    // Just in case.
    if !breakpoint.pcs.is_empty() {
        builder.breakpoints.push(breakpoint)
    }

    Ok(())
}
