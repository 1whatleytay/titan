use std::str::FromStr;
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, tag, take};
use nom::character::complete::{anychar, char, digit1, hex_digit1};
use nom::combinator::{consumed, cut, map, map_opt, peek, success, value};
use nom::character::complete::char as nom_char;
use nom::IResult;
use nom::multi::{many1, many_till};
use nom::sequence::{delimited, pair, preceded};

fn digit_literal(input: &str) -> IResult<&str, u64> {
    map(
        digit1,
        |result: &str| u64::from_str(result).unwrap_or(0)
    )(input)
}

fn binary_literal(input: &str) -> IResult<&str, u64> {
    let digit = consumed(many1(is_a("01")));

    map(
        preceded(tag("0b"), cut(digit)),
        |(consumed, _)| u64::from_str_radix(consumed, 2).unwrap_or(0)
    )(input)
}

fn hex_literal(input: &str) -> IResult<&str, u64> {
    map(
        preceded(tag("0x"), cut(hex_digit1)),
        |result: &str| u64::from_str_radix(result, 16).unwrap_or(0)
    )(input)
}

#[derive(Copy, Clone)]
enum Sign {
    Positive,
    Negative
}

fn literal_sign(input: &str) -> IResult<&str, Sign> {
    alt((
        value(Sign::Positive, nom_char('+')),
        value(Sign::Negative, nom_char('-')),
        success(Sign::Positive)
    ))(input)
}

pub fn positive_literal(input: &str) -> IResult<&str, u64> {
    alt((
        hex_literal,
        binary_literal,
        digit_literal,
    ))(input)
}

pub fn integer_literal(input: &str) -> IResult<&str, u64> {
    map(pair(literal_sign, positive_literal), |(sign, value)| {
        match sign {
            Sign::Positive => value,
            Sign::Negative => (-(value as i64)) as u64
        }
    })(input)
}

#[derive(Clone)]
enum StringPart {
    Text(String),
    Tab,
    Carriage,
    Newline,
    Backslash,
    SingleQuote,
    DoubleQuote,
    Byte(u8),
    Unicode([u8; 2]),
    UnicodeLong([u8; 4]),
}

impl StringPart {
    fn merge(parts: &Vec<StringPart>) -> String {
        let mut result = "".to_string();

        for part in parts {
            result += &match part {
                StringPart::Text(text) => text.clone(),
                StringPart::Tab => "\t".to_string(),
                StringPart::Carriage => "\r".to_string(),
                StringPart::Newline => "\n".to_string(),
                StringPart::Backslash => "\\".to_string(),
                StringPart::SingleQuote => "\'".to_string(),
                StringPart::DoubleQuote => "\"".to_string(),
                StringPart::Byte(value) => char::from(*value).to_string(),
                StringPart::Unicode(value) =>
                    String::from_utf8_lossy(value).to_string(),
                StringPart::UnicodeLong(value) =>
                    String::from_utf8_lossy(value).to_string(),
            }
        }

        return result
    }
}

fn escape(input: &str) -> IResult<&str, StringPart> {
    alt((
        value(StringPart::Tab, char('t')),
        value(StringPart::Carriage, char('r')),
        value(StringPart::Newline, char('n')),
        value(StringPart::Backslash, char('\\')),
        value(StringPart::SingleQuote, char('\'')),
        value(StringPart::DoubleQuote, char('\"')),
        preceded(char('x'), map_opt(take(2usize), |text: &str| {
            Some(StringPart::Byte(u8::from_str_radix(text, 16).ok()?))
        })),
        preceded(char('u'), map_opt(take(4usize), |text: &str| {
            let result = [
                u8::from_str_radix(&text[0 .. 2], 16).ok()?,
                u8::from_str_radix(&text[2 .. 4], 16).ok()?,
            ];

            Some(StringPart::Unicode(result))
        })),
        preceded(char('U'), map_opt(take(8usize), |text: &str| {
            let result = [
                u8::from_str_radix(&text[0 .. 2], 16).ok()?,
                u8::from_str_radix(&text[2 .. 4], 16).ok()?,
                u8::from_str_radix(&text[4 .. 6], 16).ok()?,
                u8::from_str_radix(&text[6 .. 8], 16).ok()?,
            ];

            Some(StringPart::UnicodeLong(result))
        }))
    ))(input)
}

fn string_body(input: &str) -> IResult<&str, Vec<StringPart>> {
    let parser = alt((
        preceded(char('\\'), escape),
        map(is_not("\\\""), |text: &str| StringPart::Text(text.to_string()))
    ));

    map(many_till(
        parser,
        peek(char('\"'))
    ), |(a, _)| a)(input)
}

pub fn string_literal(input: &str) -> IResult<&str, String> {
    map(delimited(
        char('\"'),
        string_body,
        char('\"')
    ), |parts| StringPart::merge(&parts))(input)
}
