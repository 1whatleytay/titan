use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use byteorder::{LittleEndian, WriteBytesExt};
use num_traits::ToPrimitive;
use crate::assembler::assembler::AddressLabel::{Constant, Label};
use crate::assembler::lexer::{Token};
use crate::assembler::lexer::TokenKind::{Directive, IntegerLiteral, NewLine, Register, Symbol};
use crate::assembler::lexer_seek::{LexerSeek, LexerSeekPeekable};
use crate::assembler::assembler::AssemblerReason::{
    UnexpectedToken,
    EndOfFile,
    ExpectedRegister,
    ExpectedConstant,
    ExpectedLabel,
    ExpectedNewline,
    UnknownInstruction
};
use crate::assembler::instructions::{Encoding, Instruction, instructions_map, Opcode};
use crate::assembler::registers::RegisterSlot;

#[derive(Debug)]
pub enum AssemblerReason<'a> {
    UnexpectedToken,
    EndOfFile,
    ExpectedRegister,
    ExpectedConstant,
    ExpectedLabel,
    ExpectedNewline,
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

#[derive(Debug)]
struct BinaryRegion {
    address: u32,
    data: Vec<u8>
}

#[derive(Debug)]
pub struct Binary {
    regions: Vec<BinaryRegion>
}

const TEXT_DEFAULT: u32 = 0x40000;

impl Binary {
    fn new() -> Binary {
        Binary { regions: vec![] }
    }

    fn seek(&mut self, address: u32) {
        self.regions.push(BinaryRegion { address, data: vec![] })
    }

    fn region(&mut self) -> Option<&mut BinaryRegion> {
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

enum AddressLabel<'a> {
    Constant(u64),
    Label(&'a str)
}

fn get_label<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<AddressLabel<'a>, AssemblerReason<'a>> {
    match get_token(iter)?.kind {
        IntegerLiteral(value) => Ok(Constant(value)),
        Symbol(value) => Ok(Label(value)),
        _ => Err(ExpectedLabel)
    }
}

fn expect_newline<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<(), AssemblerReason<'a>> {
    match get_token(iter)?.kind {
        NewLine => Ok(()),
        _ => Err(ExpectedNewline)
    }
}

fn do_directive<'a, T: LexerSeek<'a>>(directive: &'a str, iter: &mut T, binary: &mut Binary) {
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

struct EmitInstruction {
    instructions: Vec<u32>
}

fn do_register_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T
) -> Result<EmitInstruction, AssemblerReason<'a>> {
    let dest = get_register(iter)?;
    let source = get_register(iter)?;
    let temp = get_register(iter)?;

    let inst = InstructionBuilder::from_op(op)
        .with_dest(dest)
        .with_source(source)
        .with_temp(temp)
        .0;

    Ok(EmitInstruction { instructions: vec![inst] })
}

fn do_source_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T, binary: &mut Binary
) -> Result<(), AssemblerReason<'a>> {
    let base = instruction_base(&op);

    Ok(())
}

fn do_destination_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T, binary: &mut Binary
) -> Result<(), AssemblerReason<'a>> {
    let base = instruction_base(op);

    Ok(())
}

fn do_inputs_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T, binary: &mut Binary
) -> Result<(), AssemblerReason<'a>> {
    let base = instruction_base(op);

    Ok(())
}

fn do_sham_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T, binary: &mut Binary
) -> Result<(), AssemblerReason<'a>> {
    let base = instruction_base(op);

    Ok(())
}

fn do_special_branch_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T, binary: &mut Binary
) -> Result<(), AssemblerReason<'a>> {
    let base = instruction_base(op);

    Ok(())
}

fn do_immediate_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T, binary: &mut Binary
) -> Result<(), AssemblerReason<'a>> {
    let base = instruction_base(op);

    Ok(())
}

fn do_load_immediate_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T, binary: &mut Binary
) -> Result<(), AssemblerReason<'a>> {
    let base = instruction_base(op);

    Ok(())
}

fn do_jump_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T, binary: &mut Binary
) -> Result<(), AssemblerReason<'a>> {
    let base = instruction_base(op);

    Ok(())
}

fn do_branch_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T, binary: &mut Binary
) -> Result<(), AssemblerReason<'a>> {
    let base = instruction_base(op);

    Ok(())
}

fn do_parameterless_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T, binary: &mut Binary
) -> Result<(), AssemblerReason<'a>> {
    let base = instruction_base(op);

    Ok(())
}

fn do_load_offset_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T, binary: &mut Binary
) -> Result<(), AssemblerReason<'a>> {
    let base = instruction_base(op);

    Ok(())
}

fn do_store_offset_instruction<'a, T: LexerSeek<'a>>(
    op: &Opcode, iter: &mut T, binary: &mut Binary
) -> Result<(), AssemblerReason<'a>> {
    let base = instruction_base(op);

    Ok(())
}

pub fn do_instruction<'a, T: LexerSeekPeekable<'a>>(
    instruction: &'a str, iter: &mut T, binary: &mut Binary, map: &HashMap<&str, &Instruction>
) -> Result<(), AssemblerReason<'a>> {
    let lowercase = instruction.to_lowercase();
    let lowercase_ref: &str = &lowercase;

    let Some(instruction) = map.get(&lowercase_ref) else {
        return Err(UnknownInstruction(instruction))
    };

    let op = &instruction.opcode;

    Ok(())

    // match instruction.encoding {
    //     Encoding::Register => do_register_instruction(op, iter, binary),
    //     Encoding::Source => do_source_instruction(op, iter, binary),
    //     Encoding::Destination => do_destination_instruction(op, iter, binary),
    //     Encoding::Inputs => do_inputs_instruction(op, iter, binary),
    //     Encoding::Sham => do_sham_instruction(op, iter, binary),
    //     Encoding::SpecialBranch => do_special_branch_instruction(op, iter, binary),
    //     Encoding::Immediate => do_immediate_instruction(op, iter, binary),
    //     Encoding::LoadImmediate => do_load_immediate_instruction(op, iter, binary),
    //     Encoding::Jump => do_jump_instruction(op, iter, binary),
    //     Encoding::Branch => do_branch_instruction(op, iter, binary),
    //     Encoding::Parameterless => do_parameterless_instruction(op, iter, binary),
    //     Encoding::LoadOffset => do_load_offset_instruction(op, iter, binary),
    //     Encoding::StoreOffset => do_store_offset_instruction(op, iter, binary),
    // }
}

pub fn assemble<'a>(items: Vec<Token<'a>>, instructions: &[Instruction]) -> Result<Binary, AssemblerError<'a>> {
    let mut iter = items.into_iter().peekable();

    let map = instructions_map(instructions);

    let mut binary = Binary::new();
    binary.seek(TEXT_DEFAULT);

    while let Some(token) = iter.next_any() {
        match token.kind {
            Directive(directive) => do_directive(directive, &mut iter, &mut binary),
            Symbol(instruction) => do_instruction(instruction, &mut iter, &mut binary, &map)
                .map_err(|reason| AssemblerError { start: token.start, reason })?,
            _ => return Err(AssemblerError { start: token.start, reason: UnexpectedToken })
        }
    }

    Ok(binary)
}
