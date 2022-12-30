use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num_traits::ToPrimitive;
use TokenKind::{LeftBrace, RightBrace};
use crate::assembler::assembler::AddressLabel::{Constant, Label};
use crate::assembler::lexer::{Token, TokenKind};
use crate::assembler::lexer::TokenKind::{Directive, IntegerLiteral, NewLine, Register, StringLiteral, Symbol};
use crate::assembler::lexer_seek::{is_adjacent_kind, LexerSeek, LexerSeekPeekable};
use crate::assembler::assembler::InstructionLabel::{BranchLabel, JumpLabel};
use crate::assembler::instructions::{Encoding, Instruction, instructions_map, Opcode};
use crate::assembler::registers::RegisterSlot;
use crate::assembler::assembler::AssemblerReason::{
    UnexpectedToken,
    EndOfFile,
    ExpectedRegister,
    ExpectedConstant,
    ExpectedString,
    ExpectedLabel,
    ExpectedNewline,
    ExpectedLeftBrace,
    ExpectedRightBrace,
    IncorrectSegment,
    UnknownLabel,
    UnknownDirective,
    UnknownInstruction,
    JumpOutOfRange,
    MissingRegion,
    MissingInstruction,
};
use crate::assembler::assembler::BinaryBuilderMode::{Data, KernelData, KernelText, Text};

#[derive(Debug)]
pub enum AssemblerReason {
    UnexpectedToken,
    EndOfFile,
    ExpectedRegister,
    ExpectedConstant,
    ExpectedString,
    ExpectedLabel,
    ExpectedNewline,
    ExpectedLeftBrace,
    ExpectedRightBrace,
    IncorrectSegment,
    UnknownLabel(String),
    UnknownDirective(String),
    UnknownInstruction(String),
    JumpOutOfRange(u32, u32), // to, from
    MissingRegion,
    MissingInstruction
}

#[derive(Debug)]
pub struct AssemblerError<'a> {
    start: Option<&'a str>,
    reason: AssemblerReason
}

impl<'a> Display for AssemblerError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.reason)
    }
}

impl<'a> Error for AssemblerError<'a> { }

#[derive(Debug)]
pub struct RawRegion {
    address: u32,
    data: Vec<u8>,
}

pub struct Binary {
    entry: u32,
    regions: Vec<RawRegion>
}

impl Binary {
    fn new() -> Binary {
        Binary {
            entry: TEXT_DEFAULT,
            regions: vec![]
        }
    }
}

struct BinaryBuilderRegion {
    raw: RawRegion,
    labels: HashMap<usize, InstructionLabel>
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BinaryBuilderMode {
    Text, Data, KernelText, KernelData
}

impl BinaryBuilderMode {
    fn is_text(&self) -> bool {
        match self {
            Text => true,
            KernelText => true,
            _ => false
        }
    }

    fn is_data(&self) -> bool {
        match self {
            Data => true,
            KernelData => true,
            _ => false
        }
    }

    fn default_address(&self) -> u32 {
        match self {
            Text => 0x00400000,
            Data => 0x10010000,
            KernelText => 0x80000000,
            KernelData => 0x90000000
        }
    }
}

struct BinaryBuilderState {
    mode: BinaryBuilderMode,
    indices: HashMap<BinaryBuilderMode, usize>
}

struct BinaryBuilder {
    state: BinaryBuilderState,
    regions: Vec<BinaryBuilderRegion>,
    labels: HashMap<String, u32>
}

const TEXT_DEFAULT: u32 = 0x40000;

impl BinaryBuilderState {
    fn index(&self) -> Option<usize> {
        self.indices.get(&self.mode).cloned()
    }

    fn new() -> BinaryBuilderState {
        BinaryBuilderState {
            mode: Text,
            indices: HashMap::new()
        }
    }
}

impl BinaryBuilder {
    fn new() -> BinaryBuilder {
        BinaryBuilder {
            state: BinaryBuilderState::new(),
            regions: vec![],
            labels: HashMap::new()
        }
    }

    fn seek(&mut self, address: u32) -> usize {
        let index = self.regions.len();

        self.regions.push(BinaryBuilderRegion {
            raw: RawRegion { address, data: vec![] }, labels: HashMap::new()
        });

        index
    }

    fn seek_mode(&mut self, mode: BinaryBuilderMode) {
        self.state.mode = mode;

        let index = self.state.index()
            .unwrap_or_else(|| self.seek(mode.default_address()));

        self.state.indices.insert(mode, index);
    }


    fn seek_mode_address(&mut self, mode: BinaryBuilderMode, address: u32) {
        self.state.mode = mode;

        let index = self.seek(address);
        self.state.indices.insert(mode, index);
    }

    fn region(&mut self) -> Option<&mut BinaryBuilderRegion> {
        let Some(index) = self.state.index() else { return None };

        Some(&mut self.regions[index])
    }

    fn build(self) -> Result<Binary, AssemblerReason> {
        let mut binary = Binary::new();

        for region in self.regions {
            let mut raw = region.raw;

            for (offset, label) in region.labels {
                let pc = raw.address + offset as u32;
                let size = raw.data.len();

                let bytes = &raw.data[offset .. offset + 4];

                let instruction = Cursor::new(bytes).read_u32::<LittleEndian>();
                let Ok(instruction) = instruction else {
                    return Err(MissingInstruction)
                };

                let result = add_label(instruction, pc, label, &self.labels)?;

                let mut_bytes = &mut raw.data[offset .. offset + 4];

                if let Err(_) = Cursor::new(mut_bytes).write_u32::<LittleEndian>(result) {
                    return Err(MissingInstruction)
                }

                assert_eq!(size, raw.data.len());
            }

            binary.regions.push(raw)
        }

        Ok(binary)
    }
}

fn get_token<'a, T: LexerSeek<'a>>(iter: &mut T)
    -> Result<Token<'a>, AssemblerReason> {
    iter.next_adjacent().ok_or(EndOfFile)
}

fn get_register<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<RegisterSlot, AssemblerReason> {
    match get_token(iter)?.kind {
        Register(slot) => Ok(slot),
        _ => Err(ExpectedRegister)
    }
}

fn get_constant<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<u64, AssemblerReason> {
    match get_token(iter)?.kind {
        IntegerLiteral(value) => Ok(value),
        _ => Err(ExpectedConstant)
    }
}

fn get_string<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<String, AssemblerReason> {
    match get_token(iter)?.kind {
        StringLiteral(value) => Ok(value),
        _ => Err(ExpectedString)
    }
}

#[derive(Debug)]
enum AddressLabel {
    Constant(u64),
    Label(String)
}

fn get_label<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<AddressLabel, AssemblerReason> {
    match get_token(iter)?.kind {
        IntegerLiteral(value) => Ok(Constant(value)),
        Symbol(value) => Ok(Label(value.to_string())),
        _ => Err(ExpectedLabel)
    }
}

fn expect_newline<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<(), AssemblerReason> {
    match get_token(iter)?.kind {
        NewLine => Ok(()),
        _ => Err(ExpectedNewline)
    }
}

fn expect_left_brace<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<(), AssemblerReason> {
    match get_token(iter)?.kind {
        LeftBrace => Ok(()),
        _ => Err(ExpectedLeftBrace)
    }
}

fn expect_right_brace<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<(), AssemblerReason> {
    match get_token(iter)?.kind {
        RightBrace => Ok(()),
        _ => Err(ExpectedRightBrace)
    }
}

fn get_optional_constant<'a, T: LexerSeekPeekable<'a>>(iter: &mut T) -> Option<u64> {
    let next = iter.seek_without(is_adjacent_kind);

    if let Some(next) = next {
        match next.kind {
            IntegerLiteral(literal) => {
                iter.next();

                Some(literal)
            },
            _ => None
        }
    } else {
        None
    }
}

fn do_seek_directive<'a, T: LexerSeekPeekable<'a>>(
    mode: BinaryBuilderMode, iter: &mut T, builder: &mut BinaryBuilder
) -> Result<(), AssemblerReason> {
    let address = get_optional_constant(iter);

    match address {
        Some(address) => builder.seek_mode_address(mode, address as u32),
        None => builder.seek_mode(mode)
    };

    Ok(())
}

fn do_ascii_directive<'a, T: LexerSeekPeekable<'a>>(iter: &mut T, builder: &mut BinaryBuilder)
    -> Result<(), AssemblerReason> {
    if !builder.state.mode.is_data() {
        return Err(IncorrectSegment)
    }

    let mut bytes = get_string(iter)?.into_bytes();
    let region = builder.region().ok_or(MissingRegion)?;

    region.raw.data.append(&mut bytes);

    Ok(())
}

fn do_asciiz_directive<'a, T: LexerSeekPeekable<'a>>(iter: &mut T, builder: &mut BinaryBuilder)
    -> Result<(), AssemblerReason> {
    if !builder.state.mode.is_data() {
        return Err(IncorrectSegment)
    }

    let mut bytes = get_string(iter)?.into_bytes();
    bytes.push(0);

    let region = builder.region().ok_or(MissingRegion)?;

    region.raw.data.append(&mut bytes);

    Ok(())
}

fn do_align_directive<'a, T: LexerSeekPeekable<'a>>(iter: &mut T, builder: &mut BinaryBuilder)
    -> Result<(), AssemblerReason> {
    let shift = get_constant(iter)?;
    let align = 1 << shift;

    let region = builder.region().ok_or(MissingRegion)?;
    let pc = region.raw.address + region.raw.data.len() as u32;

    let (select, remainder) = (pc / align, pc % align);
    let correction = if remainder > 0 { 1 } else { 0 };

    let target = (select + correction) * align;
    let align_count = pc as usize - target as usize;

    if shift > 16 {
        builder.seek_mode_address(builder.state.mode, target)
    } else {
        let mut align_bytes = vec![0; align_count];

        region.raw.data.append(&mut align_bytes);
    }

    Ok(())
}

fn do_space_directive<'a, T: LexerSeekPeekable<'a>>(iter: &mut T, builder: &mut BinaryBuilder)
    -> Result<(), AssemblerReason> {
    Ok(())
}

fn do_byte_directive<'a, T: LexerSeekPeekable<'a>>(iter: &mut T, builder: &mut BinaryBuilder)
    -> Result<(), AssemblerReason> {
    Ok(())
}

fn do_half_directive<'a, T: LexerSeekPeekable<'a>>(iter: &mut T, builder: &mut BinaryBuilder)
    -> Result<(), AssemblerReason> {
    Ok(())
}

fn do_word_directive<'a, T: LexerSeekPeekable<'a>>(iter: &mut T, builder: &mut BinaryBuilder)
    -> Result<(), AssemblerReason> {
    Ok(())
}

fn do_float_directive<'a, T: LexerSeekPeekable<'a>>(iter: &mut T, builder: &mut BinaryBuilder)
    -> Result<(), AssemblerReason> {
    Ok(())
}

fn do_double_directive<'a, T: LexerSeekPeekable<'a>>(iter: &mut T, builder: &mut BinaryBuilder)
    -> Result<(), AssemblerReason> {
    Ok(())
}

fn do_extern_directive<'a, T: LexerSeekPeekable<'a>>(iter: &mut T, builder: &mut BinaryBuilder)
    -> Result<(), AssemblerReason> {
    Ok(())
}

fn do_directive<'a, T: LexerSeekPeekable<'a>>(
    directive: &'a str, iter: &mut T, builder: &mut BinaryBuilder
) -> Result<(), AssemblerReason> {
    let lowercase = directive.to_lowercase();

    match &lowercase as &str {
        "ascii" => do_ascii_directive(iter, builder),
        "asciiz" => do_asciiz_directive(iter, builder),
        "align" => do_align_directive(iter, builder),
        "space" => do_space_directive(iter, builder),
        "byte" => do_byte_directive(iter, builder),
        "half" => do_half_directive(iter, builder),
        "word" => do_word_directive(iter, builder),
        "float" => do_float_directive(iter, builder),
        "double" => do_double_directive(iter, builder),

        "text" => do_seek_directive(Text, iter, builder),
        "data" => do_seek_directive(Data, iter, builder),
        "ktext" => do_seek_directive(KernelText, iter, builder),
        "kdata" => do_seek_directive(KernelData, iter, builder),

        "extern" => do_extern_directive(iter, builder),
        _ => Err(UnknownDirective(directive.to_string()))
    }
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

#[derive(Debug)]
enum InstructionLabel {
    BranchLabel(AddressLabel),
    JumpLabel(AddressLabel)
}

fn get_address(label: AddressLabel, map: &HashMap<String, u32>) -> Result<u32, AssemblerReason> {
    match label {
        Constant(value) => Ok(value as u32),
        Label(name) => map.get(&name).copied().ok_or_else(|| UnknownLabel(name))
    }
}

fn add_label(instruction: u32, pc: u32, label: InstructionLabel, map: &HashMap<String, u32>)
    -> Result<u32, AssemblerReason> {
    Ok(match label {
        BranchLabel(label) => {
            let destination = get_address(label, map)?;
            let immediate = (destination >> 2) as i32 - ((pc + 4) >> 2) as i32;

            if immediate > 0xFFFF || immediate < -0x10000 {
                return Err(JumpOutOfRange(destination, pc))
            }

            instruction & 0xFFFF | (immediate as u32 & 0xFFFF)
        }
        JumpLabel(label) => {
            let destination = get_address(label, map)?;
            let lossy_mask = 0xF0000000u32;

            if destination & lossy_mask != (pc + 4) & lossy_mask {
                return Err(JumpOutOfRange(destination, pc))
            }

            let mask = !0u32 << 26;
            let constant = (destination >> 2) & (!0u32 >> 6);

            instruction & mask | constant
        }
    })
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

fn do_source_instruction<'a, T: LexerSeek<'a>>(op: &Opcode, iter: &mut T)
    -> Result<EmitInstruction, AssemblerReason> {
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

fn do_instruction<'a, T: LexerSeekPeekable<'a>>(
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

fn do_symbol<'a, T: LexerSeekPeekable<'a>>(
    name: &'a str, iter: &mut T, builder: &mut BinaryBuilder, map: &HashMap<&str, &Instruction>
) -> Result<(), AssemblerReason> {
    // We need this region!

    let region = builder.region().ok_or(MissingRegion)?;

    match iter.seek_without(is_adjacent_kind) {
        Some(token) if token.kind == TokenKind::Colon => {
            iter.next(); // consume

            let pc = region.raw.address + region.raw.data.len() as u32;
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
        let fail = |reason: AssemblerReason| AssemblerError {
            start: Some(token.start), reason
        };

        match token.kind {
            Directive(directive) => do_directive(directive, &mut iter, &mut builder),
            Symbol(name) => do_symbol(name, &mut iter, &mut builder, &map),
            _ => return Err(fail(UnexpectedToken))
        }.map_err(|reason| fail(reason))?
    }

    builder.build().map_err(|reason| AssemblerError { start: None, reason })
}
