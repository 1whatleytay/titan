use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ptr;
use num::FromPrimitive;
use std::str::FromStr;

use crate::assembler::lexer::TokenKind::{
    Comment,
    Directive,
    Parameter,
    Register,
    IntegerLiteral,
    StringLiteral,
    Symbol,
    Comma,
    Colon,
    NewLine,
    LeftBrace,
    RightBrace,
};
use crate::assembler::lexer::LexerReason::{EndOfFile, ImproperLiteral, InvalidString, Stuck, UnknownRegister};
use crate::assembler::registers::RegisterSlot;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind<'a> {
    Comment(&'a str), // #*\n
    Directive(&'a str), // .*
    Parameter(&'a str), // %*
    Register(RegisterSlot), // $*
    IntegerLiteral(u64), // 123 -> also characters
    StringLiteral(String),
    Symbol(&'a str),
    Comma,
    Colon,
    NewLine,
    LeftBrace,
    RightBrace,
}

#[derive(Clone, Debug)]
pub struct Token<'a> {
    pub start: &'a str,
    pub kind: TokenKind<'a>
}

#[derive(Debug)]
pub enum LexerReason {
    Stuck,
    EndOfFile,
    UnknownRegister,
    InvalidString,
    ImproperLiteral,
}

#[derive(Debug)]
pub struct LexerError<'a> {
    pub start: &'a str,
    pub reason: LexerReason
}

impl<'a> Display for LexerError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.reason)
    }
}

impl<'a> Error for LexerError<'a> { }

fn take_count<F>(input: &str, f: F) -> usize where F: Fn(char) -> bool {
    let mut size = 0;

    for item in input.chars() {
        if !f(item) {
            break
        }

        size += 1
    }

    size
}

fn take_while<F>(input: &str, f: F) -> &str where F: Fn(char) -> bool {
    &input[take_count(input, f)..]
}

fn take_split<F>(input: &str, f: F) -> (&str, &str) where F: Fn(char) -> bool {
    let size = take_count(input, f);

    (&input[size..], &input[..size])
}

// I want the ability to precompute a hash table, so this is done via match.
fn is_explicit_hard(c: char) -> bool {
    match c {
        ':' | ';' | ',' | '.' | '{' | '}' | '+' | '-' |
        '=' | '/' | '@' | '#' | '$' | '%' | '^' | '&' |
        '|' | '*' | '(' | ')' | '!' | '?' | '<' | '>' |
        '~' | '[' | ']' | '\\' | '\"' | '\'' => true,
        _ => false
    }
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

fn wrap_item<'a, F>(pair: (&'a str, &'a str), start: &'a str, f: F) -> (&'a str, Token<'a>)
    where F: Fn(&'a str) -> TokenKind<'a> {
    let (input, taken) = pair;

    (input, Token { start, kind: f(taken) })
}

fn maybe_wrap_item<'a, F>(pair: (&'a str, &'a str), start: &'a str, f: F)
    -> Result<(&'a str, Token<'a>), LexerError<'a>>
    where F: Fn(&'a str) -> Result<TokenKind<'a>, LexerReason> {
    let (input, taken) = pair;

    match f(taken) {
        Ok(kind) => Ok((input, Token { start, kind })),
        Err(reason) => Err(LexerError { start, reason })
    }
}

// MARS does not seem to support \x, \u or \U escapes (which require variable consumption).
// We will not support it either then.
fn escape(c: char) -> char {
    // backslash and quotes are handled under the regular case
    match c {
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        _ => c
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
            },
            _ if start == quote => {
                break // don't consume
            },
            _ => {
                let (rest, body) = take_split(
                    input, |c| c != quote && c != '\\'
                );

                input = rest;
                result += body;
            }
        }
    }

    Some((input, result))
}

fn integer_decimal(input: &str) -> Option<(&str, u64)> {
    let (input, value) = take_name(input);

    return Some((input, u64::from_str(value).ok()?));
}

fn integer_hexadecimal(input: &str) -> Option<(&str, u64)> {
    // assert(input.starts_with("0x")
    let input = &input[2..];

    let (input, value) = take_name(input);

    return Some((input, u64::from_str_radix(value, 16).ok()?));
}

fn integer_binary(input: &str) -> Option<(&str, u64)> {
    // assert(input.starts_with("0b")
    let input = &input[2..];

    let (input, value) = take_name(input);

    return Some((input, u64::from_str_radix(value, 2).ok()?));
}

fn integer_character(input: &str) -> Option<(&str, u64)> {
    // assert(input.starts_with("\'")
    let input = &input[1..];

    let (input, body) = string_body(input, '\'')?;

    if body.len() != 1 {
        return None
    }

    Some((&input[1..], body.chars().next()? as u64))
}

fn integer_literal(input: &str) -> Option<(&str, u64)> {
    let (input, positive) = match input.chars().next()? {
        '+' => (&input[1..], true),
        '-' => (&input[1..], false),
        _ => (input, true)
    };

    match input {
        _ if input.starts_with("0x") => integer_hexadecimal(input),
        _ if input.starts_with("0b") => integer_binary(input),
        _ if input.starts_with("\'") => integer_character(input),
        _ => integer_decimal(input)
    }.map(|(input, value)| (
        input, if positive { value } else { (-(value as i64)) as u64 }
    ))
}

fn lex_item(input: &str) -> Result<(&str, Token), LexerError> {
    let input = take_space(input);

    let leading = input.chars().next()
        .ok_or_else(|| LexerError { start: input, reason: EndOfFile })?;
    let after_leading = &input[1..];

    match leading {
        '#' => Ok(wrap_item(
            take_split(after_leading, |c| c != '\n'),
            after_leading, |i| Comment(i)
        )),
        '.' => Ok(wrap_item(
            take_name(after_leading),
            after_leading, |i| Directive(i)
        )),
        '%' => Ok(wrap_item(
            take_name(after_leading),
            after_leading, |i| Parameter(i)
        )),
        '$' => maybe_wrap_item(
            take_name(after_leading),
            after_leading, |i| {
                Ok(Register(
                    RegisterSlot::from_string(i)
                        .or_else(|| RegisterSlot::from_u64(u64::from_str(i).ok()?))
                        .ok_or(UnknownRegister)?
                ))
            }
        ),
        ',' => Ok((&input[1..], Token { start: input, kind: Comma })),
        '(' => Ok((&input[1..], Token { start: input, kind: LeftBrace })),
        ')' => Ok((&input[1..], Token { start: input, kind: RightBrace })),
        ':' => Ok((&input[1..], Token { start: input, kind: Colon })),
        '\n' => Ok((&input[1..], Token { start: input, kind: NewLine })),
        '0'..='9' | '-' | '+' | '\'' => return integer_literal(input)
            .map(|(out, value)| (out, Token {
                start: input, kind: IntegerLiteral(value)
            }))
            .ok_or_else(|| LexerError { start: input, reason: ImproperLiteral }),
        '\"' => string_body(after_leading, '\"')
            .map(|(out, body)| (&out[1..], Token {
                start: input, kind: StringLiteral(body)
            }))
            .ok_or_else(|| LexerError { start: input, reason: InvalidString }),

        _ => Ok(wrap_item(take_name(input), input, |i| Symbol(i)))
    }
}

pub fn lex(mut input: &str) -> Result<Vec<Token>, LexerError> {
    let mut result = vec![];

    while !input.is_empty() {
        let start = input;

        let (next, item) = lex_item(input)?;

        if ptr::eq(start.as_ptr(), next.as_ptr()) {
            return Err(LexerError { start, reason: Stuck })
        }

        result.push(item);
        input = next;
    }

    Ok(result)
}