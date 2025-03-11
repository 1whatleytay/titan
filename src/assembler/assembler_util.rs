use crate::assembler::assembler_util::InstructionValue::{Literal, Slot};
use crate::assembler::binary::AddressLabel::{Constant, Label};
use crate::assembler::binary::{AddressLabel, NamedLabel, RawRegion};
use crate::assembler::cursor::{is_adjacent_kind, LexerCursor};
use crate::assembler::lexer::TokenKind::{
    FloatLiteral, IntegerLiteral, LeftBrace, NewLine, Plus, Register, RightBrace, StringLiteral,
    Symbol,
};
use crate::assembler::lexer::{Location, StrippedKind, Token, TokenKind};
use crate::assembler::registers::RegisterSlot;
use std::error::Error;
use std::fmt::{Display, Formatter};
use TokenKind::Minus;

#[derive(Debug)]
pub enum AssemblerReason {
    UnexpectedToken(StrippedKind),
    EndOfFile,
    ExpectedRegister(StrippedKind),
    ExpectedConstant(StrippedKind),
    ExpectedString(StrippedKind),
    ExpectedLabel(StrippedKind),
    ExpectedNewline(StrippedKind),
    ExpectedLeftBrace(StrippedKind),
    ExpectedRightBrace(StrippedKind),
    ConstantOutOfRange(i64, i64),    // start, end
    OverwriteEdge(u32, Option<u64>), // pc, count
    UnknownLabel(String),
    UnknownDirective(String),
    UnknownInstruction(String),
    JumpOutOfRange(u32, u32), // to, from
    MissingRegion,
    MissingInstruction,
    DuplicateLabel(String),
}

impl Display for AssemblerReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblerReason::UnexpectedToken(kind) => write!(f, "Expected instruction or directive, but found {kind}"),
            AssemblerReason::EndOfFile => write!(f, "Assembler reached the end of the file, but requires an additional token here"),
            AssemblerReason::ExpectedRegister(kind) => write!(f, "Expected a register, but found {kind}"),
            AssemblerReason::ExpectedConstant(kind) => write!(f, "Expected an integer, but found {kind}"),
            AssemblerReason::ExpectedString(kind) => write!(f, "Expected a string literal, but found {kind}"),
            AssemblerReason::ExpectedLabel(kind) => write!(f, "Expected a label, but found {kind}"),
            AssemblerReason::ExpectedNewline(kind) => write!(f, "Expected a newline, but found {kind}"),
            AssemblerReason::ExpectedLeftBrace(kind) => write!(f, "Expected a left brace, but found {kind}"),
            AssemblerReason::ExpectedRightBrace(kind) => write!(f, "Expected a right brace, but found {kind}"),
            AssemblerReason::ConstantOutOfRange(min, max) => write!(f, "Constant must be between {min:#x} and {max:#x}"),
            AssemblerReason::OverwriteEdge(pc, count) => write!(
                f, "Instruction pushes cursor out of boundary (from {:#x}{})",
                pc, count.map(|v| format!(" with 0x{v:x} bytes")).unwrap_or("".into())
            ),
            AssemblerReason::UnknownLabel(name) => write!(f, "Could not find a label named \"{name}\", check for typos"),
            AssemblerReason::UnknownDirective(name) => write!(f, "There's no current support for any {name} directive"),
            AssemblerReason::UnknownInstruction(name) => write!(f, "Unknown instruction named \"{name}\", check for typos"),
            AssemblerReason::JumpOutOfRange(to, from) => write!(
                f, "Trying to jump to 0x{to:08x} from 0x{from:08x}, but this jump is too distant for this instruction"),
            AssemblerReason::MissingRegion => write!(
                f, "Assembler did not mount a binary region. Please file an issue at https://github.com/1whatleytay/titan/issues"),
            AssemblerReason::MissingInstruction => write!(
                f, "Assembler marked an instruction that does not exist. Please file an issue at https://github.com/1whatleytay/titan/issues"),
            AssemblerReason::DuplicateLabel(label) => write!(
                f, "Found duplicate label with the name \"{label}\", only one label with each name is allowed")
        }
    }
}

#[derive(Debug)]
pub struct AssemblerError {
    pub location: Option<Location>,
    pub reason: AssemblerReason,
}

impl Display for AssemblerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.reason.fmt(f)
    }
}

pub fn pc_for_region(
    region: &RawRegion,
    location: Option<Location>,
) -> Result<u32, AssemblerError> {
    region.pc().ok_or_else(|| {
        let reason = AssemblerReason::OverwriteEdge(region.address, Some(region.data.len() as u64));

        AssemblerError { location, reason }
    })
}

impl Error for AssemblerError {}

pub fn get_token<'a, 'b>(iter: &mut LexerCursor<'a, 'b>) -> Result<&'b Token<'a>, AssemblerError> {
    iter.next_adjacent().ok_or(AssemblerError {
        location: None,
        reason: AssemblerReason::EndOfFile,
    })
}

fn default_error(reason: AssemblerReason, token: &Token) -> AssemblerError {
    let location = if token.kind == NewLine {
        None
    } else {
        Some(token.location)
    };

    AssemblerError { location, reason }
}

pub fn get_register(iter: &mut LexerCursor) -> Result<RegisterSlot, AssemblerError> {
    let token = get_token(iter)?;

    match token.kind {
        Register(slot) => Ok(slot),
        _ => Err(default_error(
            AssemblerReason::ExpectedRegister(token.kind.strip()),
            token,
        )),
    }
}

pub enum InstructionValue {
    Slot(RegisterSlot),
    Literal(u64),
}

// first -> pointed to but NOT consumed yet, this method call will consume it
pub fn get_integer(first: &Token, iter: &mut LexerCursor, consume: bool) -> Option<u64> {
    let start = iter.get_position();

    match &first.kind {
        Plus | Minus => {
            if consume {
                iter.next(); // consume first
            }

            let multiplier = if first.kind == Plus { 1i64 } else { -1i64 };

            let adjacent = iter.next_adjacent();

            if let Some(IntegerLiteral(value)) = adjacent.map(|t| &t.kind) {
                Some(((*value as i64) * multiplier) as u64)
            } else {
                iter.set_position(start);

                None
            }
        }
        IntegerLiteral(value) => {
            if consume {
                iter.next(); // consume first
            }

            Some(*value)
        }
        _ => None,
    }
}

pub fn get_float(first: &Token, iter: &mut LexerCursor, consume: bool) -> Option<f32> {
    let start = iter.get_position();

    match &first.kind {
        Plus | Minus => {
            if consume {
                iter.next(); // consume first
            }
            let multiplier = if first.kind == Plus { 1f32 } else { -1f32 };
            let adjacent = iter.next_adjacent();
            if let Some(IntegerLiteral(value)) = adjacent.map(|t| &t.kind) {
                Some((*value as f32) * multiplier)
            } else if let Some(FloatLiteral(value)) = adjacent.map(|t| &t.kind) {
                Some(*value * multiplier)
            } else {
                iter.set_position(start);

                None
            }
        }
        IntegerLiteral(value) => {
            if consume {
                iter.next(); // consume first
            }

            Some(*value as f32)
        }
        FloatLiteral(value) => {
            if consume {
                iter.next(); // consume first
            }

            Some(*value)
        },
        _ => None,
    }
}

pub fn get_integer_adjacent(iter: &mut LexerCursor) -> Option<u64> {
    if let Some(token) = iter.seek_without(is_adjacent_kind) {
        get_integer(token, iter, true)
    } else {
        None
    }
}

pub fn get_value(iter: &mut LexerCursor) -> Result<InstructionValue, AssemblerError> {
    let token = get_token(iter)?;

    if let Some(value) = get_integer(token, iter, false) {
        Ok(Literal(value))
    } else {
        match token.kind {
            Register(slot) => Ok(Slot(slot)),
            _ => Err(default_error(
                AssemblerReason::ExpectedRegister(token.kind.strip()),
                token,
            )),
        }
    }
}

pub fn maybe_get_value(iter: &mut LexerCursor) -> Option<InstructionValue> {
    let value = iter.seek_without(is_adjacent_kind)?;

    if let Some(value) = get_integer(value, iter, true) {
        Some(Literal(value))
    } else {
        match value.kind {
            Register(slot) => {
                iter.next();

                Some(Slot(slot))
            }
            _ => None,
        }
    }
}

pub fn get_constant(iter: &mut LexerCursor) -> Result<u64, AssemblerError> {
    let token = get_token(iter)?;

    if let Some(value) = get_integer(token, iter, false) {
        Ok(value)
    } else {
        Err(default_error(
            AssemblerReason::ExpectedConstant(token.kind.strip()),
            token,
        ))
    }
}

pub fn get_string(iter: &mut LexerCursor) -> Result<String, AssemblerError> {
    let token = get_token(iter)?;

    match &token.kind {
        StringLiteral(value) => Ok(value.clone()),
        _ => Err(default_error(
            AssemblerReason::ExpectedString(token.kind.strip()),
            token,
        )),
    }
}

fn to_label(token: &Token, iter: &mut LexerCursor) -> Result<AddressLabel, AssemblerError> {
    if let Some(value) = get_integer(token, iter, false) {
        Ok(Constant(value))
    } else {
        match &token.kind {
            Symbol(value) => {
                let (position, plus) = iter.peek_adjacent();
                let follows_plus = plus.map(|token| token.kind == Plus).unwrap_or(false);

                let offset = if follows_plus {
                    iter.set_position(position);
                    iter.next(); // consume +

                    get_constant(iter)?
                } else {
                    0u64
                };

                Ok(Label(NamedLabel {
                    name: value.get().to_string(),
                    location: token.location,
                    offset,
                }))
            }
            _ => Err(default_error(
                AssemblerReason::ExpectedLabel(token.kind.strip()),
                token,
            )),
        }
    }
}

pub fn get_label(iter: &mut LexerCursor) -> Result<AddressLabel, AssemblerError> {
    to_label(get_token(iter)?, iter)
}

pub enum OffsetOrLabel {
    Label(AddressLabel),
    Offset(AddressLabel, RegisterSlot),
}

pub fn get_offset_or_label(iter: &mut LexerCursor) -> Result<OffsetOrLabel, AssemblerError> {
    let label = to_label(get_token(iter)?, iter);

    let is_offset = iter
        .seek_without(is_adjacent_kind)
        .map(|token| token.kind == LeftBrace)
        .unwrap_or(false);

    if is_offset {
        iter.next(); // left brace

        let register = get_register(iter)?;

        let Some(right) = iter.next_adjacent() else {
            return Err(AssemblerError {
                location: None,
                reason: AssemblerReason::EndOfFile,
            });
        };

        if right.kind != RightBrace {
            return Err(default_error(
                AssemblerReason::ExpectedRightBrace(right.kind.strip()),
                right,
            ));
        }

        Ok(OffsetOrLabel::Offset(
            label.unwrap_or(AddressLabel::Constant(0)),
            register,
        ))
    } else {
        Ok(OffsetOrLabel::Label(label?))
    }
}

pub fn default_start(location: Location) -> impl Fn(AssemblerError) -> AssemblerError {
    move |error| {
        if error.location.is_none() {
            AssemblerError {
                location: Some(location),
                reason: error.reason,
            }
        } else {
            error
        }
    }
}
