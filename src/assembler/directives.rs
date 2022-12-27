use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{alphanumeric0, char, multispace0, space0};
use nom::combinator::{fail, flat_map, map, opt, value};
use nom::IResult;
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded};
use crate::assembler::labels::label_name;
use crate::assembler::literals::{integer_literal, positive_literal, string_literal};
use crate::assembler::tokens::{token, token_lookup, TokenCache, with_cache};

#[derive(Clone)]
enum AddressMarker {
    Literal(u32),
    Label(String)
}

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

    Text(Option<AddressMarker>),
    Data(Option<AddressMarker>),
    KText(Option<AddressMarker>),
    KData(Option<AddressMarker>),

    Extern(String, u64),

    // Eqv,
    // Macro,
    // EndMacro,
    // Include,
}

fn address_marker(input: &str) -> IResult<&str, AddressMarker> {
    alt((
        map(integer_literal, |value| AddressMarker::Literal(value as u32)),
        map(label_name, |value| AddressMarker::Label(value.to_string()))
    ))(input)
}

fn ascii_directive<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Directive> {
    map(token(string_literal, cache), |text| Directive::Ascii(text))(input)
}

fn asciiz_directive<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Directive> {
    map(token(string_literal, cache), |text| Directive::AsciiZ(text))(input)
}

fn align_directive<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Directive> {
    map(token(positive_literal, cache), |value| Directive::Align(value))(input)
}

fn space_directive<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Directive> {
    map(token(positive_literal, cache), |value| Directive::Space(value))(input)
}

fn integer_list<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Vec<Option<u64>>> {
    many0(delimited(
        multispace0,
        alt((
            value(None, token(char(','), cache)),
            map(token(integer_literal, cache), |value| Some(value))
        )),
        multispace0
    ))(input)
}

fn byte_directive<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Directive> {
    map(with_cache(integer_list, cache), |elements| {
        Directive::Byte(elements.iter()
            .filter_map(|value| value.map(|v| v as u8))
            .collect::<Vec<u8>>()
        )
    })(input)
}

fn half_directive<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Directive> {
    map(with_cache(integer_list, cache), |elements| {
        Directive::Half(elements.iter()
            .filter_map(|value| value.map(|v| v as u16))
            .collect::<Vec<u16>>()
        )
    })(input)
}

fn word_directive<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Directive> {
    map(with_cache(integer_list, cache), |elements| {
        Directive::Word(elements.iter()
            .filter_map(|value| value.map(|v| v as u32))
            .collect::<Vec<u32>>()
        )
    })(input)
}

fn float_directive<'a>(input: &'a str, _: &'a TokenCache) -> IResult<&'a str, Directive> {
    fail(input) // unimplemented
}

fn double_directive<'a>(input: &'a str, _: &'a TokenCache) -> IResult<&'a str, Directive> {
    fail(input) // unimplemented
}

fn text_directive<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Directive> {
    map(
        opt(token_lookup(address_marker, cache)),
        |marker| Directive::Text(marker)
    )(input)
}

fn data_directive<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Directive> {
    map(
        opt(token_lookup(address_marker, cache)),
        |marker| Directive::Data(marker)
    )(input)
}

fn ktext_directive<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Directive> {
    map(
        opt(token_lookup(address_marker, cache)),
        |marker| Directive::KText(marker)
    )(input)
}

fn kdata_directive<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Directive> {
    map(
        opt(token_lookup(address_marker, cache)),
        |marker| Directive::KData(marker)
    )(input)
}

fn extern_directive<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Directive> {
    map(
        pair(token_lookup(label_name, cache), token(positive_literal, cache)),
        |(name, size)| Directive::Extern(name.to_string(), size)
    )(input)
}

fn directive_body<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Directive> {
    let (input, directive) = token(preceded(char('.'), alphanumeric0), cache)(input)?;
    let (input, _) = multispace0(input)?;

    match directive {
        "ascii" => ascii_directive(input, cache),
        "asciiz" => asciiz_directive(input, cache),
        "align" => align_directive(input, cache),
        "space" => space_directive(input, cache),
        "byte" => byte_directive(input, cache),
        "half" => half_directive(input, cache),
        "word" => word_directive(input, cache),
        "float" => float_directive(input, cache),
        "double" => double_directive(input, cache),
        "text" => text_directive(input, cache),
        "data" => data_directive(input, cache),
        "ktext" => ktext_directive(input, cache),
        "kdata" => kdata_directive(input, cache),

        _ => fail(input)
    }
}
