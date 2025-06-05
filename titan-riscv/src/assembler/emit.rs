use crate::assembler::binary_builder::InstructionLabelKind::{
    Branch, JumpAndLink, Lower12, Upper20,
};
use crate::assembler::binary_builder::{BinaryBuilder, BinaryBuilderLabel, InstructionLabel};
use crate::assembler::emit::InstructionKind::{Base, Compressed};
use crate::assembler::instruction_builder::{InstructionBuilder, SplitImmediate};
use crate::assembler::instructions::{
    ADDI_OP, BEQ_OP, BGE_OP, BGEU_OP, BLT_OP, BLTU_OP, BNE_OP, BaseOpcode, Encoding, Instruction,
    JAL_OP, JALR_OP, LUI_OP, SLLI_OP, SLT_OP, SLTIU_OP, SLTU_OP, SRAI_OP, SRLI_OP, SUB_OP, XORI_OP,
};
use crate::assembler::lexer::TokenKind;
use crate::assembler::registers::RegisterSlot;
use crate::assembler::utilities::AssemblerReason::{MissingRegion, UnknownInstruction};
use crate::assembler::utilities::{
    AssemblerError, TokenCursor, default_start, get_constant_in_range, get_label, get_offset,
    get_register, pc_for_region,
};
use byteorder::{LittleEndian, WriteBytesExt};
use std::collections::HashMap;
use titan_shared::assembler::binary::BinaryBreakpoint;
use titan_shared::assembler::lexer::Location;

enum InstructionKind {
    Base(u32),
    Compressed(u16),
}

type InstructionPair = (InstructionKind, Option<InstructionLabel>);

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
            instructions: vec![(Compressed(instruction), None)],
        }
    }
}

fn do_upper_instruction(
    op: &BaseOpcode,
    iter: &mut TokenCursor,
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let immediate = get_constant_in_range(iter, 0..=0xfffff)?;

    // Even though these are sign extended on RV64
    // It doesn't really make any sense for these to be "signed numbers."
    let inst = InstructionBuilder::from_op(op)
        .with_rd(dest)
        .with_upper_imm(immediate as u32)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_jump_immediate_instruction(
    op: &BaseOpcode,
    iter: &mut TokenCursor,
) -> Result<EmitInstruction, AssemblerError> {
    let is_register = iter
        .peek_adjacent()
        .1
        .map(|x| matches!(x.kind, TokenKind::Register(_)))
        .unwrap_or(false);

    if is_register {
        let dest = get_register(iter)?;
        let label = get_label(iter)?;

        // Even though these are sign extended on RV64
        // It doesn't really make any sense for these to be "signed numbers."
        let inst = InstructionBuilder::from_op(op).with_rd(dest).0;

        Ok(EmitInstruction {
            instructions: vec![(
                Base(inst),
                Some(InstructionLabel {
                    kind: JumpAndLink,
                    label,
                }),
            )],
        })
    } else {
        let label = get_label(iter)?;

        // Even though these are sign extended on RV64
        // It doesn't really make any sense for these to be "signed numbers."
        let inst = InstructionBuilder::from_op(op)
            .with_rd(RegisterSlot::ReturnAddress)
            .0;

        Ok(EmitInstruction {
            instructions: vec![(
                Base(inst),
                Some(InstructionLabel {
                    kind: JumpAndLink,
                    label,
                }),
            )],
        })
    }
}

fn do_branch_instruction(
    op: &BaseOpcode,
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
        instructions: vec![(
            Base(inst),
            Some(InstructionLabel {
                kind: Branch,
                label,
            }),
        )],
    })
}

fn do_jump_offset_instruction(
    op: &BaseOpcode,
    iter: &mut TokenCursor,
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;

    let is_left_brace = iter
        .peek_adjacent()
        .1
        .map(|x| x.kind == TokenKind::LeftBrace)
        .unwrap_or(false);

    if is_left_brace {
        // If we are followed by a left brace, try to get the offset.
        let offset = get_offset(iter)?;

        let inst = InstructionBuilder::from_op(op)
            .with_rd(dest)
            .with_rs1(offset.slot)
            .with_normal_imm(offset.immediate)
            .0;

        Ok(EmitInstruction::with(inst))
    } else {
        // Otherwise, we will just use the "one register" version of this instruction.

        let inst = InstructionBuilder::from_op(op)
            .with_rd(RegisterSlot::ReturnAddress)
            .with_rs1(dest) // dest is used as source instead
            .with_normal_imm(0) // zero immediate
            .0;

        Ok(EmitInstruction::with(inst))
    }
}

fn do_offset_load_instruction(
    op: &BaseOpcode,
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
    op: &BaseOpcode,
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
    op: &BaseOpcode,
    iter: &mut TokenCursor,
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let src = get_register(iter)?;
    // All arithmetic immediate instructions have signed immediate (including ORI/ANDI/XORI).
    // SRC: Sail https://riscv-software-src.github.io/riscv-unified-db/manual/html/isa/isa_20240411/insts/andi.html
    let immediate = get_constant_in_range(iter, -0x800..=0x7ff)? as i16;

    let inst = InstructionBuilder::from_op(op)
        .with_rd(dest)
        .with_rs1(src)
        .with_normal_imm(immediate)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_sham_instruction(
    op: &BaseOpcode,
    iter: &mut TokenCursor,
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let src = get_register(iter)?;
    // All arithmetic immediate instructions have signed immediate (including ORI/ANDI/XORI).
    // SRC: Sail https://riscv-software-src.github.io/riscv-unified-db/manual/html/isa/isa_20240411/insts/andi.html
    let sham = get_constant_in_range(iter, 0..=31)? as u8;

    let inst = InstructionBuilder::from_op(op)
        .with_rd(dest)
        .with_rs1(src)
        .with_sham(sham)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_registers_instruction(
    op: &BaseOpcode,
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
    op: &BaseOpcode,
    _iter: &mut TokenCursor,
) -> Result<EmitInstruction, AssemblerError> {
    // No params.

    let inst = InstructionBuilder::from_op(op).0;

    Ok(EmitInstruction::with(inst))
}

fn do_nop_instruction(_iter: &mut TokenCursor) -> Result<EmitInstruction, AssemblerError> {
    // No params.

    let inst = InstructionBuilder::from_op(&ADDI_OP)
        .with_rd(RegisterSlot::Zero)
        .with_rs1(RegisterSlot::Zero)
        .with_normal_imm(0)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_j_instruction(iter: &mut TokenCursor) -> Result<EmitInstruction, AssemblerError> {
    let label = get_label(iter)?;

    let inst = InstructionBuilder::from_op(&JAL_OP)
        .with_rd(RegisterSlot::Zero)
        .0;

    Ok(EmitInstruction {
        instructions: vec![(
            Base(inst),
            Some(InstructionLabel {
                label,
                kind: JumpAndLink,
            }),
        )],
    })
}

fn do_jr_instruction(iter: &mut TokenCursor) -> Result<EmitInstruction, AssemblerError> {
    let slot = get_register(iter)?;

    let inst = InstructionBuilder::from_op(&JALR_OP)
        .with_rd(RegisterSlot::Zero)
        .with_rs1(slot)
        .with_normal_imm(0)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_b_instruction(iter: &mut TokenCursor) -> Result<EmitInstruction, AssemblerError> {
    let label = get_label(iter)?;

    let inst = InstructionBuilder::from_op(&BEQ_OP)
        .with_rs1(RegisterSlot::Zero)
        .with_rs2(RegisterSlot::Zero)
        .0;

    Ok(EmitInstruction {
        instructions: vec![(
            Base(inst),
            Some(InstructionLabel {
                label,
                kind: Branch,
            }),
        )],
    })
}

fn do_ret_instruction(_iter: &mut TokenCursor) -> Result<EmitInstruction, AssemblerError> {
    // No params.

    let inst = InstructionBuilder::from_op(&JALR_OP)
        .with_rd(RegisterSlot::Zero)
        .with_rs1(RegisterSlot::ReturnAddress)
        .with_normal_imm(0)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_mv_instruction(iter: &mut TokenCursor) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let src = get_register(iter)?;

    let inst = InstructionBuilder::from_op(&ADDI_OP)
        .with_rd(dest)
        .with_rs1(src)
        .with_normal_imm(0)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_not_instruction(iter: &mut TokenCursor) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let src = get_register(iter)?;

    let inst = InstructionBuilder::from_op(&XORI_OP)
        .with_rd(dest)
        .with_rs1(src)
        .with_normal_imm(-1)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_neg_instruction(iter: &mut TokenCursor) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let src = get_register(iter)?;

    let inst = InstructionBuilder::from_op(&SUB_OP)
        .with_rd(dest)
        .with_rs1(RegisterSlot::Zero)
        .with_rs2(src)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_ext_instruction(
    iter: &mut TokenCursor,
    bits_kept: u8,
    signed: bool,
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let src = get_register(iter)?;

    let shift = 32 - bits_kept;

    let left = InstructionBuilder::from_op(&SLLI_OP)
        .with_rd(dest)
        .with_rs1(src)
        .with_sham(shift)
        .0;

    let right_op = if signed {
        &SRAI_OP // extend with signed bit
    } else {
        &SRLI_OP // extend with zeros
    };

    let right = InstructionBuilder::from_op(right_op)
        .with_rd(dest)
        .with_rs1(dest)
        .with_sham(shift)
        .0;

    Ok(EmitInstruction {
        instructions: vec![(Base(left), None), (Base(right), None)],
    })
}

// less_than = true -> BLT
// less_than = false -> BGE
fn do_branch_custom_instruction(
    iter: &mut TokenCursor,
    unsigned: bool,
    less_than: bool,
    flip_operands: bool,
    zero: bool,
) -> Result<EmitInstruction, AssemblerError> {
    let src1 = get_register(iter)?;
    let src2 = if zero {
        RegisterSlot::Zero
    } else {
        get_register(iter)?
    };

    let label = get_label(iter)?;

    let (src1, src2) = if flip_operands {
        (src2, src1)
    } else {
        (src1, src2)
    };

    let op = if unsigned {
        if less_than { &BLTU_OP } else { &BGEU_OP }
    } else {
        if less_than { &BLT_OP } else { &BGE_OP }
    };

    let inst = InstructionBuilder::from_op(op)
        .with_rs1(src1)
        .with_rs2(src2)
        .0;

    Ok(EmitInstruction {
        instructions: vec![(
            Base(inst),
            Some(InstructionLabel {
                label,
                kind: Branch,
            }),
        )],
    })
}

fn do_branch_eq_zero_instruction(
    iter: &mut TokenCursor,
    not_equal: bool,
) -> Result<EmitInstruction, AssemblerError> {
    let src = get_register(iter)?;
    let label = get_label(iter)?;

    let op = if not_equal { &BNE_OP } else { &BEQ_OP };

    let inst = InstructionBuilder::from_op(op)
        .with_rs1(src)
        .with_rs2(RegisterSlot::Zero)
        .0;

    Ok(EmitInstruction {
        instructions: vec![(
            Base(inst),
            Some(InstructionLabel {
                label,
                kind: Branch,
            }),
        )],
    })
}

fn do_set_custom_instruction(
    iter: &mut TokenCursor,
    unsigned: bool,
    flip_operands: bool,
    negate: bool,
    zero: bool,
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;

    let src1 = get_register(iter)?;
    let src2 = if zero {
        RegisterSlot::Zero
    } else {
        get_register(iter)?
    };

    let (src1, src2) = if flip_operands {
        (src2, src1)
    } else {
        (src1, src2)
    };

    let op = if unsigned { &SLTU_OP } else { &SLT_OP };

    let inst = InstructionBuilder::from_op(op)
        .with_rd(dest)
        .with_rs1(src1)
        .with_rs2(src2)
        .0;

    let mut instructions = vec![(Base(inst), None)];

    if negate {
        let negate_inst = InstructionBuilder::from_op(&XORI_OP)
            .with_rd(dest)
            .with_rs1(dest)
            .with_normal_imm(1) // flip 1 bit
            .0;

        instructions.push((Base(negate_inst), None));
    }

    Ok(EmitInstruction { instructions })
}

fn do_seq_instruction(
    iter: &mut TokenCursor,
    zero: bool,
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;

    let src1 = get_register(iter)?;

    let mut instructions = vec![];

    let src = if zero {
        src1
    } else {
        // we need to compute src1 - src2
        // hopefully this is a correct implementation

        let src2 = get_register(iter)?;

        let sub_inst = InstructionBuilder::from_op(&SUB_OP)
            .with_rd(dest)
            .with_rs1(src1)
            .with_rs2(src2)
            .0;

        instructions.push((Base(sub_inst), None));

        dest
    };

    let inst = InstructionBuilder::from_op(&SLTIU_OP)
        .with_rd(dest)
        .with_rs1(src)
        .with_normal_imm(1)
        .0;

    instructions.push((Base(inst), None));

    Ok(EmitInstruction { instructions })
}

fn do_sne_instruction(
    iter: &mut TokenCursor,
    zero: bool,
) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;

    let src1 = get_register(iter)?;

    let mut instructions = vec![];

    let src = if zero {
        src1
    } else {
        // we need to compute src1 - src2
        // hopefully this is a correct implementation

        let src2 = get_register(iter)?;

        let sub_inst = InstructionBuilder::from_op(&SUB_OP)
            .with_rd(dest)
            .with_rs1(src1)
            .with_rs2(src2)
            .0;

        instructions.push((Base(sub_inst), None));

        dest
    };

    let inst = InstructionBuilder::from_op(&SLTU_OP)
        .with_rd(dest)
        .with_rs1(RegisterSlot::Zero)
        .with_rs2(src)
        .0;

    instructions.push((Base(inst), None));

    Ok(EmitInstruction { instructions })
}

fn do_li_instruction(iter: &mut TokenCursor) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let constant = get_constant_in_range(iter, -0x80000000..=0x7fffffff)?;

    let split = SplitImmediate::from_immediate(constant as u32);

    let mut instructions = vec![];

    let base = if split.upper != 0 {
        let lui_inst = InstructionBuilder::from_op(&LUI_OP)
            .with_rd(dest)
            .with_upper_imm(split.upper)
            .0;

        instructions.push((Base(lui_inst), None));

        dest
    } else {
        RegisterSlot::Zero
    };

    if split.lower != 0 || split.upper == 0 {
        let addi_inst = InstructionBuilder::from_op(&ADDI_OP)
            .with_rd(dest)
            .with_rs1(base)
            .with_normal_imm(split.lower)
            .0;

        instructions.push((Base(addi_inst), None));
    }

    Ok(EmitInstruction { instructions })
}

fn do_la_instruction(iter: &mut TokenCursor) -> Result<EmitInstruction, AssemblerError> {
    let dest = get_register(iter)?;
    let label = get_label(iter)?;

    let lui_inst = InstructionBuilder::from_op(&LUI_OP).with_rd(dest).0;

    let addi_inst = InstructionBuilder::from_op(&ADDI_OP)
        .with_rd(dest)
        .with_rs1(dest)
        .0;

    let instructions = vec![
        (
            Base(lui_inst),
            Some(InstructionLabel {
                label: label.clone(),
                kind: Upper20,
            }),
        ),
        (
            Base(addi_inst),
            Some(InstructionLabel {
                label,
                kind: Lower12,
            }),
        ),
    ];

    Ok(EmitInstruction { instructions })
}

fn dispatch_pseudo(
    instruction: &str,
    iter: &mut TokenCursor,
) -> Result<Option<EmitInstruction>, AssemblerError> {
    Ok(Some(match instruction {
        // Nop
        "nop" => do_nop_instruction(iter),

        // Loads
        "li" => do_li_instruction(iter),
        "la" => do_la_instruction(iter),
        // no support for "load/store local/global" yet

        // Branches
        "bgt" => do_branch_custom_instruction(iter, false, true, true, false),
        "ble" => do_branch_custom_instruction(iter, false, false, true, false),
        "bgtu" => do_branch_custom_instruction(iter, true, true, true, false),
        "bleu" => do_branch_custom_instruction(iter, true, false, true, false),

        // Branch Zero
        "beqz" => do_branch_eq_zero_instruction(iter, false),
        "bnez" => do_branch_eq_zero_instruction(iter, true),
        "blez" => do_branch_custom_instruction(iter, false, false, true, true),
        "bgez" => do_branch_custom_instruction(iter, false, false, false, true),
        "bltz" => do_branch_custom_instruction(iter, false, true, false, true),
        "bgtz" => do_branch_custom_instruction(iter, false, true, true, true),

        // Sets
        "sge" => do_set_custom_instruction(iter, false, false, true, false),
        "sgt" => do_set_custom_instruction(iter, false, true, false, false),
        "sle" => do_set_custom_instruction(iter, false, true, true, false),
        "sgeu" => do_set_custom_instruction(iter, true, false, true, false),
        "sgtu" => do_set_custom_instruction(iter, true, true, false, false),
        "sleu" => do_set_custom_instruction(iter, true, true, true, false),
        "sltz" => do_set_custom_instruction(iter, false, false, false, true),
        "sgez" => do_set_custom_instruction(iter, false, false, true, true),
        "sgtz" => do_set_custom_instruction(iter, false, true, false, true),
        "slez" => do_set_custom_instruction(iter, false, true, true, true),

        "seq" => do_seq_instruction(iter, false),
        "sne" => do_sne_instruction(iter, false),
        "seqz" => do_seq_instruction(iter, true),
        "snez" => do_sne_instruction(iter, true),

        // Jumps
        "j" => do_j_instruction(iter),
        "jr" => do_jr_instruction(iter),
        "b" => do_b_instruction(iter),
        "ret" => do_ret_instruction(iter),

        // Arithmetic
        "mv" => do_mv_instruction(iter),
        "not" => do_not_instruction(iter),
        "neg" => do_neg_instruction(iter),
        "sext.b" => do_ext_instruction(iter, 8, true),
        "sext.h" => do_ext_instruction(iter, 16, true),
        "zext.b" => do_ext_instruction(iter, 8, false),
        "zext.h" => do_ext_instruction(iter, 16, false),

        _ => return Ok(None),
    }?))
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

    let emit = match &instruction.encoding {
        Encoding::UpperImmediate { op } => do_upper_instruction(op, iter),
        Encoding::JumpImmediate { op } => do_jump_immediate_instruction(op, iter),
        Encoding::JumpOffset { op } => do_jump_offset_instruction(op, iter),
        Encoding::Branch { op } => do_branch_instruction(op, iter),
        Encoding::OffsetLoad { op } => do_offset_load_instruction(op, iter),
        Encoding::OffsetStore { op } => do_offset_store_instruction(op, iter),
        Encoding::ArithmeticImmediate { op } => do_arithmetic_immediate_instruction(op, iter),
        Encoding::Sham { op } => do_sham_instruction(op, iter),
        Encoding::Registers { op } => do_registers_instruction(op, iter),
        Encoding::Single { op } => do_single_instruction(op, iter),
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
            Compressed(compressed) => region
                .raw
                .data
                .write_u16::<LittleEndian>(compressed)
                .unwrap(),
        }
    }

    // Just in case.
    if !breakpoint.pcs.is_empty() {
        builder.breakpoints.push(breakpoint)
    }

    Ok(())
}
