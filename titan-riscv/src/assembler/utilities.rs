use titan_shared::assembler::binary::RawRegion;
use crate::assembler::lexer::TokenKind::{Comma, Comment, FloatLiteral, IntegerLiteral, LeftBrace, NewLine, Plus, Register, RightBrace, StringLiteral, Symbol};
use crate::assembler::lexer::{Location, StrippedKind, Token, TokenKind};
use crate::assembler::registers::RegisterSlot;
use titan_shared::assembler::cursor::{BaseTokenCursor, TokenCursorInsights};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::{RangeInclusive};
use TokenKind::Minus;
use crate::assembler::binary_builder::{AddressLabel, NamedLabel};
use crate::assembler::binary_builder::AddressLabel::{Constant, Label};

pub fn is_solid_kind(kind: &TokenKind) -> bool {
    match kind {
        Comment(_) => false,
        NewLine => false,
        Comma => false, // Completely ignored by MARS.
        _ => true,
    }
}

pub fn is_adjacent_kind(kind: &TokenKind) -> bool {
    match kind {
        Comment(_) => false,
        Comma => false, // Completely ignored by MARS.
        _ => true,
    }
}


pub fn is_solid(token: &Token) -> bool {
    is_solid_kind(&token.kind)
}

pub fn is_adjacent(token: &Token) -> bool {
    is_adjacent_kind(&token.kind)
}

pub struct TokenInsights;

impl<'a, 'b> TokenCursorInsights<'b, Token<'a>> for TokenInsights {
    fn is_adjacent(&self, token: &'b Token<'a>) -> bool {
        is_adjacent_kind(&token.kind)
    }
}

pub type TokenCursor<'a, 'b> = BaseTokenCursor<'b, Token<'a>, TokenInsights>;

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

pub fn get_token<'a, 'b>(iter: &mut TokenCursor<'a, 'b>) -> Result<&'b Token<'a>, AssemblerError> {
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

pub fn get_register(iter: &mut TokenCursor) -> Result<RegisterSlot, AssemblerError> {
    let token = get_token(iter)?;

    if let Register(slot) = token.kind {
        Ok(slot)
    } else {
        Err(default_error(
            AssemblerReason::ExpectedRegister(token.kind.strip()),
            token,
        ))
    }
}

// pub enum InstructionValue {
//     Slot(RegisterSlot),
//     Literal(u64),
// }

// first -> pointed to but NOT consumed yet, this method call will consume it
pub fn get_integer(first: &Token, iter: &mut TokenCursor, consume: bool) -> Option<u64> {
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

pub fn get_float(first: &Token, iter: &mut TokenCursor, consume: bool) -> Option<f64> {
    let start = iter.get_position();

    match &first.kind {
        Plus | Minus => {
            if consume {
                iter.next(); // consume first
            }
            let multiplier = if first.kind == Plus { 1f64 } else { -1f64 };
            let adjacent = iter.next_adjacent();
            match adjacent.map(|t| &t.kind) {
                Some(IntegerLiteral(value)) => Some((*value as f64) * multiplier),
                Some(FloatLiteral(value)) => Some(*value * multiplier),
                _ => {
                    iter.set_position(start);
                    None
                }
            }
        }
        IntegerLiteral(value) => {
            if consume {
                iter.next(); // consume first
            }

            Some(*value as f64)
        }
        FloatLiteral(value) => {
            if consume {
                iter.next(); // consume first
            }

            Some(*value)
        }
        _ => None,
    }
}

pub fn get_integer_adjacent(iter: &mut TokenCursor) -> Option<u64> {
    if let Some(token) = iter.seek_without(is_adjacent) {
        get_integer(token, iter, true)
    } else {
        None
    }
}

// pub fn get_value(iter: &mut TokenCursor) -> Result<InstructionValue, AssemblerError> {
//     let token = get_token(iter)?;
//
//     if let Some(value) = get_integer(token, iter, false) {
//         Ok(Literal(value))
//     } else {
//         match token.kind {
//             Register(slot) => Ok(Slot(slot)),
//             _ => Err(default_error(
//                 AssemblerReason::ExpectedRegister(token.kind.strip()),
//                 token,
//             )),
//         }
//     }
// }
//
// pub fn maybe_get_value(iter: &mut TokenCursor) -> Option<InstructionValue> {
//     let value = iter.seek_without(is_adjacent)?;
//
//     if let Some(value) = get_integer(value, iter, true) {
//         Some(Literal(value))
//     } else {
//         match value.kind {
//             Register(slot) => {
//                 iter.next();
//
//                 Some(Slot(slot))
//             }
//             _ => None,
//         }
//     }
// }

pub fn get_constant(iter: &mut TokenCursor) -> Result<u64, AssemblerError> {
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

pub fn get_constant_in_range(iter: &mut TokenCursor, range: RangeInclusive<i64>) -> Result<u64, AssemblerError> {
    let token = get_token(iter)?;

    if let Some(value) = get_integer(token, iter, false) {
        if range.contains(&(value as i64)) { //
            Ok(value)
        } else {
            // This could cause some kind of overflow bug in the future.
            Err(default_error(
                AssemblerReason::ConstantOutOfRange(*range.start(), *range.end()),
                token,
            ))
        }
    } else {
        Err(default_error(
            AssemblerReason::ExpectedConstant(token.kind.strip()),
            token,
        ))
    }
}

pub fn get_string(iter: &mut TokenCursor) -> Result<String, AssemblerError> {
    let token = get_token(iter)?;

    match &token.kind {
        StringLiteral(value) => Ok(value.clone()),
        _ => Err(default_error(
            AssemblerReason::ExpectedString(token.kind.strip()),
            token,
        )),
    }
}

fn to_label(token: &Token, iter: &mut TokenCursor) -> Result<AddressLabel, AssemblerError> {
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

pub fn get_label(iter: &mut TokenCursor) -> Result<AddressLabel, AssemblerError> {
    to_label(get_token(iter)?, iter)
}

pub struct Offset {
    pub immediate: i16,
    pub slot: RegisterSlot
}

pub fn get_offset(iter: &mut TokenCursor) -> Result<Offset, AssemblerError> {
    // 12-bit offset immediate for all offset instructions
    let immediate = get_constant_in_range(iter, -0x800 ..= 0x7ff)? as i16;

    let left_brace = get_token(iter)?;

    if left_brace.kind != LeftBrace {
        return Err(AssemblerError {
            location: Some(left_brace.location),
            reason: AssemblerReason::ExpectedLeftBrace(left_brace.kind.strip()),
        })
    }

    let slot = get_register(iter)?;

    let right_brace = get_token(iter)?;

    if right_brace.kind != RightBrace {
        return Err(AssemblerError {
            location: Some(right_brace.location),
            reason: AssemblerReason::ExpectedLeftBrace(right_brace.kind.strip()),
        })
    }

    Ok(Offset {
        immediate,
        slot
    })
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
