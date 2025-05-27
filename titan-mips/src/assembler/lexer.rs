use num::FromPrimitive;
pub use titan_shared::assembler::lexer::Location;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ptr;
use std::str::FromStr;
use SymbolName::Owned;
use titan_shared::assembler::lexer::{is_hard, numeric_literal, string_body, take_name, take_space, take_split, NumericLiteral};
use titan_shared::assembler::source::{LexerProvider, TokenProvider};
use TokenKind::{Minus, Plus};

use crate::assembler::lexer::LexerReason::{
    ImproperLiteral, InvalidString, Stuck, UnexpectedCharacter, UnknownRegister,
};
use crate::assembler::lexer::SymbolName::Slice;
use crate::assembler::lexer::TokenKind::{
    Colon, Comma, Comment, Directive, FPRegister, FloatLiteral, IntegerLiteral, LeftBrace, NewLine,
    Parameter, Register, RightBrace, StringLiteral, Symbol,
};
use crate::assembler::registers::RegisterSlot;

use super::registers::FPRegisterSlot;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolName<'a> {
    Slice(&'a str),
    Owned(String),
}

// Temporary Trait Alias
pub trait MipsTokenProvider<'a> : TokenProvider<Token<'a>, LexerError> { }

impl<'a, T: TokenProvider<Token<'a>, LexerError>> MipsTokenProvider<'a> for T { }

impl<'a> SymbolName<'a> {
    pub fn get<'b: 'a>(&'b self) -> &'b str {
        match self {
            Slice(text) => text,
            Owned(text) => text,
        }
    }
}

fn offset_from_start(start: &str, other: &str) -> usize {
    other.as_ptr() as usize - start.as_ptr() as usize
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StrippedKind {
    Comment,
    Directive,
    Parameter,
    Register,
    FPRegister,
    IntegerLiteral,
    FloatLiteral,
    StringLiteral,
    Symbol,
    Plus,
    Minus,
    Comma,
    Colon,
    NewLine,
    LeftBrace,
    RightBrace,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind<'a> {
    Comment(&'a str),           // #*\n
    Directive(&'a str),         // .*
    Parameter(&'a str),         // %*
    Register(RegisterSlot),     // $*
    FPRegister(FPRegisterSlot), // $f*
    IntegerLiteral(u64),        // 123 -> also characters
    FloatLiteral(f64),          // 123.0
    StringLiteral(String),
    Symbol(SymbolName<'a>),
    Plus,
    Minus,
    Comma,
    Colon,
    NewLine,
    LeftBrace,
    RightBrace,
}

impl Display for StrippedKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                StrippedKind::Comment => "Comment",
                StrippedKind::Directive => "Directive",
                StrippedKind::Parameter => "Parameter",
                StrippedKind::Register => "Register",
                StrippedKind::FPRegister => "Floating Point Register",
                StrippedKind::IntegerLiteral => "Integer Literal",
                StrippedKind::FloatLiteral => "Float Literal",
                StrippedKind::StringLiteral => "String Literal",
                StrippedKind::Symbol => "Symbol",
                StrippedKind::Plus => "Plus",
                StrippedKind::Minus => "Minus",
                StrippedKind::Comma => "Comma",
                StrippedKind::Colon => "Colon",
                StrippedKind::NewLine => "NewLine",
                StrippedKind::LeftBrace => "LeftBrace",
                StrippedKind::RightBrace => "RightBrace",
            }
        )
    }
}

impl TokenKind<'_> {
    pub fn strip(&self) -> StrippedKind {
        match self {
            Comment(_) => StrippedKind::Comment,
            Directive(_) => StrippedKind::Directive,
            Parameter(_) => StrippedKind::Parameter,
            Register(_) => StrippedKind::Register,
            FPRegister(_) => StrippedKind::FPRegister,
            IntegerLiteral(_) => StrippedKind::IntegerLiteral,
            FloatLiteral(_) => StrippedKind::FloatLiteral,
            StringLiteral(_) => StrippedKind::StringLiteral,
            Symbol(_) => StrippedKind::Symbol,
            Plus => StrippedKind::Plus,
            Minus => StrippedKind::Minus,
            Comma => StrippedKind::Comma,
            Colon => StrippedKind::Colon,
            NewLine => StrippedKind::NewLine,
            LeftBrace => StrippedKind::LeftBrace,
            RightBrace => StrippedKind::RightBrace,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token<'a> {
    pub location: Location,
    pub kind: TokenKind<'a>,
}

#[derive(Debug)]
pub enum LexerReason {
    Stuck,
    UnknownRegister(String),
    UnexpectedCharacter(char),
    InvalidString,
    ImproperLiteral,
}

impl Display for LexerReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Stuck => write!(f, "Lexer got stuck on this token. Please file an issue at https://github.com/1whatleytay/titan/issues"),
            UnknownRegister(register) => write!(f, "Unknown register \"{register}\""),
            UnexpectedCharacter(c) => write!(f, "Unexpected character \"{c}\""),
            InvalidString => write!(f, "String literal is incorrectly formatted. Check that you have closing quotes"),
            ImproperLiteral => write!(f, "Integer literal is incorrectly formatted or too big"),
        }
    }
}

#[derive(Debug)]
pub struct LexerError {
    pub location: Location,
    pub reason: LexerReason,
}

impl Display for LexerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.reason.fmt(f)
    }
}

impl Error for LexerError {}


fn lex_item(input: &str) -> Result<Option<(&str, TokenKind)>, LexerReason> {
    let input = take_space(input);

    let Some(leading) = input.chars().next() else {
        return Ok(None);
    };
    let after_leading = &input[leading.len_utf8()..];

    match leading {
        '#' => Ok({
            let (rest, value) = take_split(after_leading, |c| c != '\n');

            Some((rest, Comment(value)))
        }),
        '.' => Ok({
            let (rest, value) = take_name(after_leading);

            Some((rest, Directive(value)))
        }),
        '%' => Ok({
            let (rest, value) = take_name(after_leading);

            Some((rest, Parameter(value)))
        }),
        '$' => {
            let (rest, value) = take_name(after_leading);

            let mut chars = value.chars();
            if chars.next() == Some('f') && chars.next() != Some('p') {
                FPRegisterSlot::from_string(value)
                    .map(|reg| Some((rest, TokenKind::FPRegister(reg))))
                    .ok_or_else(|| UnknownRegister(value.to_string()))
            } else {
                RegisterSlot::from_string(value)
                    .or_else(|| RegisterSlot::from_u64(u64::from_str(value).ok()?))
                    .map(|slot| Some((rest, Register(slot))))
                    .ok_or_else(|| UnknownRegister(value.to_string()))
            }
        }
        '+' => Ok(Some((&input[1..], Plus))),
        '-' => Ok(Some((&input[1..], Minus))),
        ',' => Ok(Some((&input[1..], Comma))),
        '(' => Ok(Some((&input[1..], LeftBrace))),
        ')' => Ok(Some((&input[1..], RightBrace))),
        ':' => Ok(Some((&input[1..], Colon))),
        '\n' => Ok(Some((&input[1..], NewLine))),
        '0'..='9' | '\'' => numeric_literal(input)
            .map(|(out, value)| match value {
                NumericLiteral::Integer(value) => Some((out, IntegerLiteral(value))),
                NumericLiteral::Float(value) => Some((out, FloatLiteral(value))),
            })
            .ok_or(ImproperLiteral),
        '\"' => string_body(after_leading, '\"')
            .map(|(out, body)| Some((&out[1..], StringLiteral(body))))
            .ok_or(InvalidString),
        _ if is_hard(leading) => Err(UnexpectedCharacter(leading)),
        _ => Ok({
            let (rest, value) = take_name(input);

            Some((rest, Symbol(Slice(value))))
        }),
    }
}

pub fn lex_with_source(mut input: &str, source: usize) -> Result<Vec<Token>, LexerError> {
    let begin = input;
    let mut result = vec![];

    while !input.is_empty() {
        let trail = input;
        let start = offset_from_start(begin, trail);
        let location = Location {
            source,
            index: start,
        };

        let Some((next, kind)) =
            lex_item(input).map_err(|reason| LexerError { location, reason })?
        else {
            break;
        };

        if ptr::eq(trail.as_ptr(), next.as_ptr()) {
            return Err(LexerError {
                location,
                reason: Stuck,
            });
        }

        result.push(Token { location, kind });
        input = next;
    }

    Ok(result)
}

pub fn lex(input: &str) -> Result<Vec<Token>, LexerError> {
    lex_with_source(input, 0)
}

pub struct MipsLexerProvider;

impl<'a> LexerProvider<'a, Token<'a>, LexerError> for MipsLexerProvider {
    fn lex(&self, source: &'a str, id: usize) -> Result<Vec<Token<'a>>, LexerError> {
        lex_with_source(source, id)
    }
}
