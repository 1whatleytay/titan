use nom;
use nom::branch::alt;
use nom::character::complete::{multispace0, newline, not_line_ending};
use nom::character::complete::char as nom_char;
use nom::combinator::{fail, map, success, value};
use nom::IResult;
use nom::multi::many0;
use nom::sequence::{delimited, pair};
use crate::assembler::assembler::Entry::DirectiveEntry;
use crate::assembler::directives::{Directive, directive};
use crate::assembler::directives::Directive::Eqv;
use crate::assembler::tokens::{TokenCache, with_cache};

fn comment(input: &str) -> IResult<&str, ()> {
    value(
        (),
        pair(nom_char('#'), not_line_ending)
    )(input)
}

#[derive(Debug, Clone)]
pub enum RegInstruction {

}

#[derive(Debug, Clone)]
pub enum ImmInstruction {
    A
}

#[derive(Debug, Clone)]
pub enum JumpInstruction {

}

#[derive(Debug, Clone)]
pub enum Instruction {
    Register(RegInstruction),
    Immediate(ImmInstruction),
    Jump(JumpInstruction)
}

#[derive(Debug, Clone)]
pub enum Entry {
    DirectiveEntry(Directive),
    InstructionEntry(Instruction)
}

fn entry<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Entry> {
    delimited(
        multispace0,
        alt((
            map(with_cache(directive, cache), |dir| {
                DirectiveEntry(dir)
            }),
            fail
        )),
        multispace0
    )(input)
}

pub fn root<'a>(input: &'a str, cache: &'a TokenCache) -> IResult<&'a str, Vec<Entry>> {
    map(many0(
        alt((
            value(None, delimited(multispace0, comment, multispace0)),
            map(with_cache(entry, cache), |entry| Some(entry))
        ))
    ), |entries| entries.iter()
        .filter_map(|value| value.clone())
        .collect::<Vec<Entry>>()
    )(input)
}
