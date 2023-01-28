use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::assembler::binary::AddressLabel;
use crate::assembler::binary::AddressLabel::{Constant, Label};
use crate::assembler::lexer::{StrippedKind, Token, TokenKind};
use crate::assembler::lexer::TokenKind::{IntegerLiteral, Register, StringLiteral, Symbol, LeftBrace, RightBrace};
use crate::assembler::lexer_seek::{is_adjacent_kind, LexerSeek, LexerSeekPeekable};
use crate::assembler::registers::RegisterSlot;
use crate::assembler::assembler_util::InstructionValue::{Literal, Slot};
use crate::assembler::assembler_util::AssemblerReason::{
    UnexpectedToken,
    EndOfFile,
    ExpectedRegister,
    ExpectedConstant,
    ExpectedString,
    ExpectedLabel,
    ExpectedNewline,
    ExpectedLeftBrace,
    ExpectedRightBrace,
    UnknownLabel,
    UnknownDirective,
    UnknownInstruction,
    JumpOutOfRange,
    MissingRegion,
    MissingInstruction,
};

#[derive(Debug)]
pub enum AssemblerReason {
    UnexpectedToken,
    EndOfFile,
    ExpectedRegister(StrippedKind),
    ExpectedConstant(StrippedKind),
    ExpectedString(StrippedKind),
    ExpectedLabel(StrippedKind),
    ExpectedNewline(StrippedKind),
    ExpectedLeftBrace(StrippedKind),
    ExpectedRightBrace(StrippedKind),
    UnknownLabel(String),
    UnknownDirective(String),
    UnknownInstruction(String),
    JumpOutOfRange(u32, u32), // to, from
    MissingRegion,
    MissingInstruction
}

impl Display for AssemblerReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnexpectedToken => write!(f, "Expected instruction or directive, but encountered some unexpected token"),
            EndOfFile => write!(f, "Assembler reached the end of the file, but requires an additional token here"),
            ExpectedRegister(kind) => write!(f, "Expected a register, but found {}", kind),
            ExpectedConstant(kind) => write!(f, "Expected an integer, but found {}", kind),
            ExpectedString(kind) => write!(f, "Expected a string literal, but found {}", kind),
            ExpectedLabel(kind) => write!(f, "Expected a label, but found {}", kind),
            ExpectedNewline(kind) => write!(f, "Expected a newline, but found {}", kind),
            ExpectedLeftBrace(kind) => write!(f, "Expected a left brace, but found {}", kind),
            ExpectedRightBrace(kind) => write!(f, "Expected a right brace, but found {}", kind),
            UnknownLabel(name) => write!(f, "Could not find a label named \"{}\", check for typos", name),
            UnknownDirective(name) => write!(f, "There's no current support for any {} directive", name),
            UnknownInstruction(name) => write!(f, "Unknown instruction named \"{}\", check for typos", name),
            JumpOutOfRange(to, from) => write!(f, "Trying to jump to 0x{:08x} from 0x{:08x}, but this jump is too distant for this instruction", to, from),
            MissingRegion => write!(f, "Assembler did not mount a binary region. Please file an issue at https://github.com/1whatleytay/titan/issues"),
            MissingInstruction => write!(f, "Assembler marked an instruction that does not exist. Please file an issue at https://github.com/1whatleytay/titan/issues"),
        }
    }
}

#[derive(Debug)]
pub struct AssemblerError {
    pub start: Option<usize>,
    pub reason: AssemblerReason
}

impl Display for AssemblerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.reason.fmt(f)
    }
}

impl Error for AssemblerError { }

pub fn get_token<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<Token<'a>, AssemblerReason> {
    iter.next_adjacent().ok_or(EndOfFile)
}

pub fn get_register<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<RegisterSlot, AssemblerReason> {
    let t = get_token(iter)?;
    match t.kind {
        Register(slot) => Ok(slot),
        _ => Err(ExpectedRegister(t.kind.strip()))
    }
}

pub enum InstructionValue {
    Slot(RegisterSlot),
    Literal(u64)
}

pub fn get_value<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<InstructionValue, AssemblerReason> {
    let t = get_token(iter)?;
    match t.kind {
        Register(slot) => Ok(Slot(slot)),
        IntegerLiteral(value) => Ok(Literal(value)),
        _ => Err(ExpectedRegister(t.kind.strip()))
    }
}

pub fn maybe_get_value<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T
) -> Option<InstructionValue> {
    let Some(value) = iter.seek_without(is_adjacent_kind) else { return None };

    match value.kind {
        Register(slot) => {
            iter.next();

            Some(Slot(slot))
        },
        IntegerLiteral(value) => {
            iter.next();

            Some(Literal(value))
        },
        _ => None
    }
}

pub fn get_constant<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<u64, AssemblerReason> {
    let token_kind = get_token(iter)?.kind;

    match token_kind {
        IntegerLiteral(value) => Ok(value),
        _ => Err(ExpectedConstant(token_kind.strip()))
    }
}

pub fn get_string<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<String, AssemblerReason> {
    let token_kind = get_token(iter)?.kind;

    match token_kind {
        StringLiteral(value) => Ok(value),
        _ => Err(ExpectedString(token_kind.strip()))
    }
}

fn to_label(kind: TokenKind) -> Result<AddressLabel, AssemblerReason> {
    match kind {
        IntegerLiteral(value) => Ok(Constant(value)),
        Symbol(value) => Ok(Label(value.get().to_string())),
        _ => Err(ExpectedLabel(kind.strip()))
    }
}

pub fn get_label<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<AddressLabel, AssemblerReason> {
    to_label(get_token(iter)?.kind)
}

pub enum OffsetOrLabel {
    Offset(u64, RegisterSlot),
    Address(AddressLabel)
}

pub fn get_offset_or_label<'a, T: LexerSeekPeekable<'a>>(iter: &mut T) -> Result<OffsetOrLabel, AssemblerReason> {
    let token_kind = get_token(iter)?.kind;

    let is_offset = iter.seek_without(is_adjacent_kind)
        .map(|token| token.kind == LeftBrace)
        .unwrap_or(false);

    if is_offset {
        let IntegerLiteral(value) = token_kind else {
            return Err(ExpectedLabel(token_kind.strip()))
        };

        iter.next(); // left brace

        let register = get_register(iter)?;

        let Some(right) = iter.next_adjacent() else {
            return Err(EndOfFile)
        };

        if right.kind != RightBrace {
            return Err(ExpectedRightBrace(token_kind.strip()))
        }

        Ok(OffsetOrLabel::Offset(value, register))
    } else {
        Ok(OffsetOrLabel::Address(to_label(token_kind)?))
    }
}

pub fn get_optional_constant<'a, T: LexerSeekPeekable<'a>>(iter: &mut T) -> Option<u64> {
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
