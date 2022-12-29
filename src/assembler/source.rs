use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::assembler::assembler::{assemble, Binary, AssemblerError};
use crate::assembler::lexer::{lex, LexerError};
use crate::assembler::preprocessor::{preprocess, PreprocessorError};
use crate::assembler::source::SourceError::{Assembler, Lexer, Preprocessor};

#[derive(Debug)]
pub enum SourceError<'a> {
    Lexer(LexerError<'a>),
    Preprocessor(PreprocessorError<'a>),
    Assembler(AssemblerError)
}

impl<'a> From<LexerError<'a>> for SourceError<'a> {
    fn from(value: LexerError<'a>) -> Self {
        Lexer(value)
    }
}

impl<'a> From<PreprocessorError<'a>> for SourceError<'a> {
    fn from(value: PreprocessorError<'a>) -> Self {
        Preprocessor(value)
    }
}

impl<'a> From<AssemblerError> for SourceError<'a> {
    fn from(value: AssemblerError) -> Self {
        Assembler(value)
    }
}

impl<'a> Display for SourceError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Lexer(error) => Display::fmt(error, f),
            Preprocessor(error) => Display::fmt(error, f),
            Assembler(error) => Display::fmt(error, f)
        }
    }
}

impl<'a> Error for SourceError<'a> { }

pub fn assemble_from(source: &str) -> Result<Binary, SourceError> {
    let items = lex(source)?;
    let items = preprocess(items)?;
    let binary = assemble(items)?;

    Ok(binary)
}