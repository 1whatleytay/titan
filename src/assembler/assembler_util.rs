use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::assembler::binary::AddressLabel;
use crate::assembler::binary::AddressLabel::{Constant, Label};
use crate::assembler::lexer::{Token};
use crate::assembler::lexer::TokenKind::{IntegerLiteral, Register, StringLiteral, Symbol, LeftBrace, RightBrace};
use crate::assembler::lexer_seek::{is_adjacent_kind, LexerSeek, LexerSeekPeekable};
use crate::assembler::registers::RegisterSlot;
use crate::assembler::assembler_util::AssemblerReason::{EndOfFile, ExpectedConstant, ExpectedLabel, ExpectedLeftBrace, ExpectedRegister, ExpectedRightBrace, ExpectedString};
use crate::assembler::assembler_util::InstructionValue::{Literal, Slot};

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
    UnknownLabel(String),
    UnknownDirective(String),
    UnknownInstruction(String),
    JumpOutOfRange(u32, u32), // to, from
    MissingRegion,
    MissingInstruction
}

#[derive(Debug)]
pub struct AssemblerError {
    pub start: Option<usize>,
    pub reason: AssemblerReason
}

impl Display for AssemblerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.reason)
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
        _ => Err(ExpectedRegister)
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
        _ => Err(ExpectedRegister)
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
    match get_token(iter)?.kind {
        IntegerLiteral(value) => Ok(value),
        _ => Err(ExpectedConstant)
    }
}

pub fn get_string<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<String, AssemblerReason> {
    match get_token(iter)?.kind {
        StringLiteral(value) => Ok(value),
        _ => Err(ExpectedString)
    }
}

pub fn get_label<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<AddressLabel, AssemblerReason> {
    match get_token(iter)?.kind {
        IntegerLiteral(value) => Ok(Constant(value)),
        Symbol(value) => Ok(Label(value.get().to_string())),
        _ => Err(ExpectedLabel)
    }
}

pub fn expect_left_brace<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<(), AssemblerReason> {
    match get_token(iter)?.kind {
        LeftBrace => Ok(()),
        _ => Err(ExpectedLeftBrace)
    }
}

pub fn expect_right_brace<'a, T: LexerSeek<'a>>(iter: &mut T) -> Result<(), AssemblerReason> {
    match get_token(iter)?.kind {
        RightBrace => Ok(()),
        _ => Err(ExpectedRightBrace)
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
