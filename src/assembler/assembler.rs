use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::assembler::lexer::{Token};
use crate::assembler::lexer_seek::LexerSeek;

#[derive(Debug)]
pub enum AssemblerError {

}

impl Display for AssemblerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Assembler Error")
    }
}

impl Error for AssemblerError { }

struct BinaryRegion {
    start: u32,
    data: Vec<u8>
}

pub struct Binary {
    regions: Vec<BinaryRegion>
}

impl Binary {
    fn new() -> Binary {
        Binary { regions: vec![] }
    }
}

pub fn assemble(items: Vec<Token>) -> Result<Binary, AssemblerError> {
    let mut binary = Binary::new();

    let mut iter = items.into_iter();

    while let Some(token) = iter.next_any() {

    }

    Ok(binary)
}
