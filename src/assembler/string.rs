use crate::assembler::assembler_util::AssemblerError;
use crate::assembler::binary::Binary;
use crate::assembler::core::assemble;
use crate::assembler::instructions::INSTRUCTIONS;
use crate::assembler::lexer::{lex, LexerError, Location};
use crate::assembler::preprocessor::{preprocess, PreprocessorError};
use crate::assembler::string::SourceError::{Assembler, Lexer, Preprocessor};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use crate::assembler::source::{FileProviderPool, HoldingProvider};

#[derive(Debug)]
pub enum SourceError {
    Lexer(LexerError),
    Preprocessor(PreprocessorError),
    Assembler(AssemblerError),
}

impl SourceError {
    pub fn start(&self) -> Option<Location> {
        match self {
            Lexer(error) => Some(error.location),
            Preprocessor(error) => Some(error.location),
            Assembler(error) => error.location,
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
    let provider = HoldingProvider::new(items);

    let items = preprocess(&provider)?;
    let binary = assemble(&items, &INSTRUCTIONS)?;

    Ok(binary)
}

pub fn assemble_from_path(source: String, path: PathBuf) -> Result<Binary, SourceError> {
    let pool = FileProviderPool::new();

    let provider = pool.provider_sourced(source, path.into())?.to_provider();

    let items = preprocess(&provider)?;
    let binary = assemble(&items, &INSTRUCTIONS)?;

    Ok(binary)
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::path::PathBuf;
    use crate::assembler::string::assemble_from_path;

    #[test]
    fn test() {
        let text = fs::read_to_string("/Users/desgroup/Desktop/benchmarks/test.asm").unwrap();

        assemble_from_path(text, PathBuf::from("/Users/desgroup/Desktop/benchmarks/test.asm")).unwrap();
    }
}
