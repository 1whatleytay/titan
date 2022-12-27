use nom;
use nom::branch::alt;
use nom::character::complete::{newline, not_line_ending};
use nom::character::complete::char as nom_char;
use nom::combinator::{success, value};
use nom::IResult;
use nom::multi::many0;
use nom::sequence::pair;

// fn comment(input: &str) -> IResult<&str, ()> {
//     value(
//         (),
//         pair(nom_char('#'), not_line_ending)
//     )(input)
// }
//
// #[derive(Clone)]
// enum RegInstruction {
//
// }
//
// #[derive(Clone)]
// enum ImmInstruction {
//
// }
//
// #[derive(Clone)]
// enum JumpInstruction {
//
// }
//
// #[derive(Clone)]
// enum Instruction {
//     Register(RegInstruction),
//     Immediate(ImmInstruction),
//     Jump(JumpInstruction)
// }
//
// #[derive(Clone)]
// enum Entry {
//     DirectiveEntry(Directive),
//     InstructionEntry(Instruction)
// }
//
// fn entry(input: &str) -> IResult<&str, Entry> {
//     success(DirectiveEntry(Include))(input)
// }
//
// pub fn root(input: &str) -> IResult<&str, ()> {
//     value((), many0(alt((comment, value((), newline)))))(input)
// }

#[cfg(test)]
mod tests {
    use std::fs;
    use crate::assembler::literals::string_literal;

    #[test]
    fn test_me() {
        // let text = fs::read_to_string("/Users/desgroup/Projects/breakout/breakout.asm").unwrap();

        let (text, result) = string_literal("\"hel\\\"lo\n\n\"\"\"\"").unwrap();

        println!("{:?}, text: {}", result, text);
    }
}
