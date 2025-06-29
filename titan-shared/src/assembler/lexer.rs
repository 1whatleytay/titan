use std::str::FromStr;

#[derive(Copy, Clone, Debug)]
pub struct Location {
    pub source: usize,
    pub index: usize,
}

pub enum NumericLiteral {
    Integer(u64),
    Float(f64),
}

pub fn take_count<F>(input: &str, f: F) -> usize
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

pub fn take_while<F>(input: &str, f: F) -> &str
where
    F: Fn(char) -> bool,
{
    &input[take_count(input, f)..]
}

pub fn take_split<F>(input: &str, f: F) -> (&str, &str)
where
    F: Fn(char) -> bool,
{
    let size = take_count(input, f);

    (&input[size..], &input[..size])
}

pub fn take_space(input: &str) -> &str {
    take_while(input, |c| c != '\n' && c.is_whitespace())
}

pub fn is_hard(c: char) -> bool {
    c.is_whitespace() || is_explicit_hard(c)
}

// I want the ability to precompute a hash table, so this is done via match.
pub fn is_explicit_hard(c: char) -> bool {
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

pub fn take_name(input: &str) -> (&str, &str) {
    take_split(input, |c| !is_hard(c))
}

pub fn take_numeric_name(input: &str) -> (&str, &str) {
    take_split(input, |c| !is_hard(c) || c == '+' || c == '-')
}

pub fn numeric_decimal(input: &str) -> Option<(&str, NumericLiteral)> {
    let (input, value) = take_numeric_name(input);

    if value.contains('e') || value.contains('.') {
        return Some((input, NumericLiteral::Float(f64::from_str(value).ok()?)));
    }

    Some((input, NumericLiteral::Integer(u64::from_str(value).ok()?)))
}

pub fn integer_hexadecimal(input: &str) -> Option<(&str, u64)> {
    // assert(input.starts_with("0x")
    let input = &input[2..];

    let (input, value) = take_name(input);

    Some((input, u64::from_str_radix(value, 16).ok()?))
}

pub fn integer_binary(input: &str) -> Option<(&str, u64)> {
    // assert(input.starts_with("0b")
    let input = &input[2..];

    let (input, value) = take_name(input);

    Some((input, u64::from_str_radix(value, 2).ok()?))
}

pub fn integer_character(input: &str) -> Option<(&str, u64)> {
    // assert(input.starts_with("\'")
    let input = &input[1..];

    let (input, body) = string_body(input, '\'')?;

    if body.len() != 1 {
        return None;
    }

    // Should be over a quote...
    Some((&input[1..], body.chars().next()? as u64))
}

pub fn numeric_literal(input: &str) -> Option<(&str, NumericLiteral)> {
    match input {
        _ if input.starts_with("0x") => {
            integer_hexadecimal(input).map(|(a, b)| (a, NumericLiteral::Integer(b)))
        }
        _ if input.starts_with("0b") => {
            integer_binary(input).map(|(a, b)| (a, NumericLiteral::Integer(b)))
        }
        _ if input.starts_with('\'') => {
            integer_character(input).map(|(a, b)| (a, NumericLiteral::Integer(b as u64)))
        }
        _ => numeric_decimal(input),
    }
}

// MARS does not seem to support \x, \u or \U escapes (which require variable consumption).
// We will not support it either then.
pub fn escape(c: char) -> char {
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
pub fn string_body(mut input: &str, quote: char) -> Option<(&str, String)> {
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
