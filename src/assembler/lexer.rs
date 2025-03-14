use num::FromPrimitive;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ptr;
use std::str::FromStr;
use SymbolName::Owned;
use TokenKind::{Minus, Plus};

use crate::assembler::lexer::LexerReason::{
    ImproperLiteral, InvalidString, Stuck, UnexpectedCharacter, UnknownRegister,
};
use crate::assembler::lexer::SymbolName::Slice;
use crate::assembler::lexer::TokenKind::{
    Colon, Comma, Comment, Directive, IntegerLiteral, LeftBrace, NewLine, Parameter, Register,
    RightBrace, StringLiteral, Symbol,
};
use crate::assembler::registers::RegisterSlot;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolName<'a> {
    Slice(&'a str),
    Owned(String),
}

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
    IntegerLiteral,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind<'a> {
    Comment(&'a str),       // #*\n
    Directive(&'a str),     // .*
    Parameter(&'a str),     // %*
    Register(RegisterSlot), // $*
    IntegerLiteral(u64),    // 123 -> also characters
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
                StrippedKind::IntegerLiteral => "Integer Literal",
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
            IntegerLiteral(_) => StrippedKind::IntegerLiteral,
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

#[derive(Copy, Clone, Debug)]
pub struct Location {
    pub source: usize,
    pub index: usize,
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

fn take_count<F>(input: &str, f: F) -> usize
where
    F: Fn(char) -> bool,
{
    let mut size = 0;

    for item in input.chars() {
        if !f(item) {
            break;
        }

        size += item.len_utf8()
    }

    size
}

fn take_while<F>(input: &str, f: F) -> &str
where
    F: Fn(char) -> bool,
{
    &input[take_count(input, f)..]
}

fn take_split<F>(input: &str, f: F) -> (&str, &str)
where
    F: Fn(char) -> bool,
{
    let size = take_count(input, f);

    (&input[size..], &input[..size])
}

// I want the ability to precompute a hash table, so this is done via match.
fn is_explicit_hard(c: char) -> bool {
    matches!(
        c,
        ':' | ';'
            | ','
            | '{'
            | '}'
            | '+'
            | '-'
            | '='
            | '/'
            | '@'
            | '#'
            | '$'
            | '%'
            | '^'
            | '&'
            | '|'
            | '*'
            | '('
            | ')'
            | '!'
            | '?'
            | '<'
            | '>'
            | '~'
            | '['
            | ']'
            | '\\'
            | '\"'
            | '\''
    )
}

fn is_hard(c: char) -> bool {
    c.is_whitespace() || is_explicit_hard(c)
}

fn take_space(input: &str) -> &str {
    take_while(input, |c| c != '\n' && c.is_whitespace())
}

fn take_name(input: &str) -> (&str, &str) {
    take_split(input, |c| !is_hard(c))
}

// MARS does not seem to support \x, \u or \U escapes (which require variable consumption).
// We will not support it either then.
fn escape(c: char) -> char {
    // backslash and quotes are handled under the regular case
    match c {
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        '0' => '\0',
        _ => c,
    }
}

// If Some is returned, then the first char of .0 should be quote.
fn string_body(mut input: &str, quote: char) -> Option<(&str, String)> {
    let mut result = "".to_string();

    loop {
        let start = input.chars().next()?;

        match start {
            '\\' => {
                result += &escape(input.chars().nth(1)?).to_string();

                input = &input[2..];
            }
            _ if start == quote => {
                break; // don't consume
            }
            _ => {
                let (rest, body) = take_split(input, |c| c != quote && c != '\\');

                input = rest;
                result += body;
            }
        }
    }

    Some((input, result))
}

fn integer_decimal(input: &str) -> Option<(&str, u64)> {
    let (input, value) = take_name(input);

    Some((input, u64::from_str(value).ok()?))
}

fn integer_hexadecimal(input: &str) -> Option<(&str, u64)> {
    // assert(input.starts_with("0x")
    let input = &input[2..];

    let (input, value) = take_name(input);

    Some((input, u64::from_str_radix(value, 16).ok()?))
}

fn integer_binary(input: &str) -> Option<(&str, u64)> {
    // assert(input.starts_with("0b")
    let input = &input[2..];

    let (input, value) = take_name(input);

    Some((input, u64::from_str_radix(value, 2).ok()?))
}

fn integer_character(input: &str) -> Option<(&str, u64)> {
    // assert(input.starts_with("\'")
    let input = &input[1..];

    let (input, body) = string_body(input, '\'')?;

    if body.len() != 1 {
        return None;
    }

    // Should be over a quote...
    Some((&input[1..], body.chars().next()? as u64))
}

fn integer_literal(input: &str) -> Option<(&str, u64)> {
    match input {
        _ if input.starts_with("0x") => integer_hexadecimal(input),
        _ if input.starts_with("0b") => integer_binary(input),
        _ if input.starts_with('\'') => integer_character(input),
        _ => integer_decimal(input),
    }
}

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

            RegisterSlot::from_string(value)
                .or_else(|| RegisterSlot::from_u64(u64::from_str(value).ok()?))
                .map(|slot| Some((rest, Register(slot))))
                .ok_or_else(|| UnknownRegister(value.to_string()))
        }
        '+' => Ok(Some((&input[1..], Plus))),
        '-' => Ok(Some((&input[1..], Minus))),
        ',' => Ok(Some((&input[1..], Comma))),
        '(' => Ok(Some((&input[1..], LeftBrace))),
        ')' => Ok(Some((&input[1..], RightBrace))),
        ':' => Ok(Some((&input[1..], Colon))),
        '\n' => Ok(Some((&input[1..], NewLine))),
        '0'..='9' | '\'' => integer_literal(input)
            .map(|(out, value)| Some((out, IntegerLiteral(value))))
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
