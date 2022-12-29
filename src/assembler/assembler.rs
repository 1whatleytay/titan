use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use byteorder::{LittleEndian, WriteBytesExt};
use num_traits::ToPrimitive;
use TokenKind::{LeftBrace, RightBrace};
use crate::assembler::assembler::AddressLabel::{Constant, Label};
use crate::assembler::lexer::{Token, TokenKind};
use crate::assembler::lexer::TokenKind::{Directive, IntegerLiteral, NewLine, Register, Symbol};
use crate::assembler::lexer_seek::{is_adjacent_kind, LexerSeek, LexerSeekPeekable};
use crate::assembler::assembler::InstructionLabel::{BranchLabel, JumpLabel};
use crate::assembler::instructions::{Encoding, Instruction, instructions_map, Opcode};
use crate::assembler::registers::RegisterSlot;
use crate::assembler::assembler::AssemblerReason::{UnexpectedToken, EndOfFile, ExpectedRegister, ExpectedConstant, ExpectedLabel, ExpectedNewline, UnknownInstruction, ExpectedLeftBrace, ExpectedRightBrace};

#[derive(Debug)]
pub enum AssemblerReason<'a> {
    UnexpectedToken,
    EndOfFile,
    ExpectedRegister,
    ExpectedConstant,
    ExpectedLabel,
    ExpectedNewline,
    ExpectedLeftBrace,
    ExpectedRightBrace,
    UnknownInstruction(&'a str)
}

#[derive(Debug)]
pub struct AssemblerError<'a> {
    start: &'a str,
    reason: AssemblerReason<'a>
}

impl<'a> Display for AssemblerError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Assembler Error")
    }
}

impl<'a> Error for AssemblerError<'a> { }

pub struct BinaryRegion {
    address: u32,
    data: Vec<u8>,
}

pub type Binary = Vec<BinaryRegion>;

#[derive(Debug)]
struct BinaryBuilderRegion {
    address: u32,
    data: Vec<u8>,
    labels: HashMap<usize, InstructionLabel>
}

#[derive(Debug)]
pub struct BinaryBuilder {
    regions: Vec<BinaryBuilderRegion>,
    labels: HashMap<String, u32>
}

const TEXT_DEFAULT: u32 = 0x40000;

impl BinaryBuilder {
    fn new() -> BinaryBuilder {
        BinaryBuilder { regions: vec![], labels: HashMap::new() }
    }

    fn seek(&mut self, address: u32) {
        self.regions.push(BinaryBuilderRegion {
            address, data: vec![], labels: HashMap::new()
        })
    }

    fn region(&mut self) -> Option<&mut BinaryBuilderRegion> {
        self.regions.last_mut()
    }

    fn emit(&mut self, word: u32) {
        let Some(region) = self.region() else { return };

        region.data.write_u32::<LittleEndian>(word).ok();
    }
}

fn get_token<'a, T: LexerSeek<'a>>(iter: &mut T)
    -> Result<Token<'a>, AssemblerReason<'a>> {
    iter.next_adjacent().ok_or(EndOfFile)
}

fn get_register<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<RegisterSlot, AssemblerReason<'a>> {
    match get_token(iter)?.kind {
        Register(slot) => Ok(slot),
        _ => Err(ExpectedRegister)
    }
}

fn get_constant<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<u64, AssemblerReason<'a>> {
    match get_token(iter)?.kind {
        IntegerLiteral(value) => Ok(value),
        _ => Err(ExpectedConstant)
    }
}

#[derive(Debug)]
enum AddressLabel {
    Constant(u64),
    Label(String)
}

fn get_label<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<AddressLabel, AssemblerReason<'a>> {
    match get_token(iter)?.kind {
        IntegerLiteral(value) => Ok(Constant(value)),
        Symbol(value) => Ok(Label(value.to_string())),
        _ => Err(ExpectedLabel)
    }
}

fn expect_newline<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<(), AssemblerReason<'a>> {
    match get_token(iter)?.kind {
        NewLine => Ok(()),
        _ => Err(ExpectedNewline)
    }
}

fn expect_left_brace<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<(), AssemblerReason<'a>> {
    match get_token(iter)?.kind {
        LeftBrace => Ok(()),
        _ => Err(ExpectedLeftBrace)
    }
}

fn expect_right_brace<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<(), AssemblerReason<'a>> {
    match get_token(iter)?.kind {
        RightBrace => Ok(()),
        _ => Err(ExpectedRightBrace)
    }
}

fn do_directive<'a, T: LexerSeek<'a>>(directive: &'a str, iter: &mut T, builder: &mut BinaryBuilder) {
    panic!();
}

fn instruction_base(op: &Opcode) -> u32 {
    match op {
        Opcode::Op(key) => (*key as u32 & 0b111111) << 26,
        Opcode::Func(key) => *key as u32 & 0b111111, // opcode: 0
        Opcode::Special(key) => (*key as u32 & 0b111111) << 16 | (0b000001) << 26 // opcode: 1
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

    fn with_slot_offset(mut self, slot: RegisterSlot, offset: u32) -> InstructionBuilder {
        self.0 &= 0b11111 << offset;
        self.0 |= register_source(slot) << offset;

        self
    }

    fn with_dest(self, slot: RegisterSlot) -> InstructionBuilder {
        self.with_slot_offset(slot, 11)
    }

    fn with_temp(self, slot: RegisterSlot) -> InstructionBuilder {
        self.with_slot_offset(slot, 16)
    }

    fn with_source(self, slot: RegisterSlot) -> InstructionBuilder {
        self.with_slot_offset(slot, 21)
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

#[derive(Debug)]
enum InstructionLabel {
    BranchLabel(AddressLabel),
    JumpLabel(AddressLabel)
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
    -> Result<EmitInstruction, AssemblerReason<'a>> {
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

fn do_source_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
    -> Result<EmitInstruction, AssemblerReason<'a>> {
    let source = get_register(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_destination_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
    -> Result<EmitInstruction, AssemblerReason<'a>> {
    let dest = get_register(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_dest(dest)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_inputs_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
    -> Result<EmitInstruction, AssemblerReason<'a>> {
    let source = get_register(iter)?;
    let temp = get_register(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .with_temp(temp)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_sham_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
    -> Result<EmitInstruction, AssemblerReason<'a>> {
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
    -> Result<EmitInstruction, AssemblerReason<'a>> {
    let source = get_register(iter)?;
    let label = get_label(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .0;

    Ok(EmitInstruction { instructions: vec![(inst, Some(BranchLabel(label)))] })
}

fn do_immediate_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
    -> Result<EmitInstruction, AssemblerReason<'a>> {
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
    -> Result<EmitInstruction, AssemblerReason<'a>> {
    let temp = get_register(iter)?;
    let constant = get_constant(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_temp(temp)
        .with_immediate(constant as u16)
        .0;

    Ok(EmitInstruction::with(inst))
}

fn do_jump_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
    -> Result<EmitInstruction, AssemblerReason<'a>> {
    let label = get_label(iter)?;

    let inst = InstructionBuilder::from_op(op).0;

    Ok(EmitInstruction { instructions: vec![(inst, Some(JumpLabel(label)))] })
}

fn do_branch_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
    -> Result<EmitInstruction, AssemblerReason<'a>> {
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
    -> Result<EmitInstruction, AssemblerReason<'a>> {
    let source = get_register(iter)?;
    let label = get_label(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_source(source)
        .0;

    Ok(EmitInstruction { instructions: vec![(inst, Some(BranchLabel(label)))] })
}

fn do_parameterless_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, _: &mut T)
    -> Result<EmitInstruction, AssemblerReason<'a>> {
    let inst = InstructionBuilder::from_op(op).0;

    Ok(EmitInstruction::with(inst))
}

fn do_offset_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
    -> Result<EmitInstruction, AssemblerReason<'a>> {
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

fn do_instruction<'a, T: LexerSeekPeekable<'a>>(
    instruction: &'a str, iter: &mut T, builder: &mut BinaryBuilder, map: &HashMap<&str, &Instruction>
) -> Result<(), AssemblerReason<'a>> {
    let lowercase = instruction.to_lowercase();
    let lowercase_ref: &str = &lowercase;

    let Some(instruction) = map.get(&lowercase_ref) else {
        return Err(UnknownInstruction(instruction))
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

    Ok(())
}

fn do_label<'a, T: LexerSeekPeekable<'a>>(
    name: &'a str, iter: &mut T, builder: &mut BinaryBuilder, map: &HashMap<&str, &Instruction>
) -> Result<(), AssemblerReason<'a>> {
    // We need this region!
    let region = builder.region().unwrap();

    match iter.seek_without(is_adjacent_kind) {
        Some(token) if token.kind == TokenKind::Colon => {
            let pc = region.address + region.data.len() as u32;
            builder.labels.insert(name.to_string(), pc);

            Ok(())
        },
        _ => do_instruction(name, iter, builder, map)
    }
}

pub fn assemble<'a>(items: Vec<Token<'a>>, instructions: &[Instruction]) -> Result<Binary, AssemblerError<'a>> {
    let mut iter = items.into_iter().peekable();

    let map = instructions_map(instructions);

    let mut builder = BinaryBuilder::new();
    builder.seek(TEXT_DEFAULT);

    while let Some(token) = iter.next_any() {
        match token.kind {
            Directive(directive) => do_directive(directive, &mut iter, &mut builder),
            Symbol(instruction) => do_instruction(instruction, &mut iter, &mut builder, &map)
                .map_err(|reason| AssemblerError { start: token.start, reason })?,
            _ => return Err(AssemblerError { start: token.start, reason: UnexpectedToken })
        }
    }

    Ok(Binary::new())
}
