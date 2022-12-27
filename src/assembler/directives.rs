use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{char, multispace0};
use nom::combinator::map;
use nom::IResult;
use nom::sequence::preceded;
use crate::assembler::literals::string_literal;

#[derive(Clone)]
enum Directive {
    Ascii(String),
    AsciiZ(String),

    Align(u64),
    Space(u64),
    Byte(Vec<u8>),
    Half(Vec<u16>),
    Word(Vec<u32>),
    Float(Vec<f32>),
    Double(Vec<f64>),

    Text(Option<u32>),
    Data(Option<u32>),
    KData(Option<u32>),
    KText(Option<u32>),

    Extern(String, u64),

    // Eqv,
    // Macro,
    // EndMacro,
    // Include,
}

fn ascii_directive(input: &str) -> IResult<&str, Directive> {
    preceded(
        tag_no_case("ascii"),
        preceded(multispace0, map(
            string_literal, |text| Directive::Ascii(text)
        ))
    )(input)
}

fn asciiz_directive(input: &str) -> IResult<&str, Directive> {
    preceded(
        tag_no_case("asciiz"),
        preceded(multispace0, map(
            string_literal, |text| Directive::AsciiZ(text)
        ))
    )(input)
}

fn directive_body(input: &str) -> IResult<&str, Directive> {
    alt((
        ascii_directive,
        asciiz_directive,
    ))(input)
}

fn directive(input: &str) -> IResult<&str, Directive> {
    preceded(char(('.')), directive_body)(input)
}
