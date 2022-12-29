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
    address: u32,
    data: Vec<u8>
}

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

pub fn assemble(items: Vec<Token>) -> Result<Binary, AssemblerError> {
    let mut iter = items.into_iter().peekable();

    let mut binary = Binary::new();
    binary.seek(TEXT_DEFAULT);

    while let Some(token) = iter.next_any() {
        match token.kind {
            _ => panic!()
        }
    }

    Ok(binary)
}
