use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::assembler::lexer::{Token};
use crate::assembler::lexer::TokenKind::{Directive, Symbol};
use crate::assembler::lexer_seek::{LexerSeek, LexerSeekPeekable};
use crate::assembler::assembler::AssemblerReason::UnexpectedToken;

#[derive(Debug)]
pub enum AssemblerReason {
    UnexpectedToken
}

#[derive(Debug)]
pub struct AssemblerError<'a> {
    start: &'a str,
    reason: AssemblerReason
}

impl<'a> Display for AssemblerError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Assembler Error")
    }
}

impl<'a> Error for AssemblerError<'a> { }

#[derive(Debug)]
struct BinaryRegion {
    address: u32,
    data: Vec<u8>
}

#[derive(Debug)]
pub struct Binary {
    regions: Vec<BinaryRegion>
}

const TEXT_DEFAULT: u32 = 0x40000;

impl Binary {
    fn new() -> Binary {
        Binary { regions: vec![] }
    }

    fn seek(&mut self, address: u32) {
        self.regions.push(BinaryRegion { address, data: vec![] })
    }

    fn region(&mut self) -> Option<&mut BinaryRegion> {
        self.regions.last_mut()
    }
}

pub fn do_directive<'a, T>(directive: &'a str, iter: &mut T, binary: &mut Binary) where T: LexerSeekPeekable<'a> {
    panic!();
}

pub fn do_instruction<'a, T>(instruction: &'a str, iter: &mut T, binary: &mut Binary) where T: LexerSeekPeekable<'a> {
    panic!();
}

pub fn assemble(items: Vec<Token>) -> Result<Binary, AssemblerError> {
    let mut iter = items.into_iter().peekable();

    let mut binary = Binary::new();
    binary.seek(TEXT_DEFAULT);

    while let Some(token) = iter.next_any() {
        match token.kind {
            Directive(directive) => do_directive(directive, &mut iter, &mut binary),
            Symbol(instruction) => do_instruction(instruction, &mut iter, &mut binary),
            _ => return Err(AssemblerError { start: token.start, reason: UnexpectedToken })
        }
    }

    Ok(binary)
}
