use crate::assembler::assembler_util::AssemblerError;
use crate::assembler::binary::Binary;
use crate::assembler::core::assemble;
use crate::assembler::instructions::INSTRUCTIONS;
use crate::assembler::lexer::{lex, LexerError};
use crate::assembler::preprocessor::{preprocess, PreprocessorError};
use crate::assembler::string::SourceError::{Assembler, Lexer, Preprocessor};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum SourceError {
    Lexer(LexerError),
    Preprocessor(PreprocessorError),
    Assembler(AssemblerError),
}

impl SourceError {
    pub fn start(&self) -> Option<usize> {
        match self {
            Lexer(error) => Some(error.start),
            Preprocessor(error) => Some(error.start),
            Assembler(error) => error.start,
        }
    }
}

impl From<LexerError> for SourceError {
    fn from(value: LexerError) -> Self {
        Lexer(value)
    }
}

impl From<PreprocessorError> for SourceError {
    fn from(value: PreprocessorError) -> Self {
        Preprocessor(value)
    }
}

impl From<AssemblerError> for SourceError {
    fn from(value: AssemblerError) -> Self {
        Assembler(value)
    }
}

impl Display for SourceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Lexer(error) => Display::fmt(error, f),
            Preprocessor(error) => Display::fmt(error, f),
            Assembler(error) => Display::fmt(error, f),
        }
    }
}

impl Error for SourceError {}

pub fn assemble_from(source: &str) -> Result<Binary, SourceError> {
    let items = lex(source)?;
    let items = preprocess(&items)?;
    let binary = assemble(&items, &INSTRUCTIONS)?;

    Ok(binary)
}
