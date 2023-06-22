use std::fs;
use typed_arena::Arena;
use std::path::PathBuf;
use crate::assembler::lexer::{lex, LexerError, Token};
use crate::assembler::source::ExtendError::{FailedToRead, LexerFailed, NotSupported};

pub enum ExtendError {
    NotSupported,
    FailedToRead(String),
    LexerFailed(LexerError)
}

pub trait TokenProvider<'a>: Sized {
    fn get(&self) -> &[Token<'a>];

    fn extend(&self, path: &str) -> Result<Self, ExtendError>;
}

pub struct HoldingProvider<'a> {
    tokens: Vec<Token<'a>>
}

impl<'a> HoldingProvider<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> HoldingProvider {
        HoldingProvider { tokens }
    }

    pub fn from_source(source: &str) -> Result<HoldingProvider, LexerError> {
        Ok(HoldingProvider { tokens: lex(source)? })
    }
}

impl<'a> TokenProvider<'a> for HoldingProvider<'a> {
    fn get(&self) -> &[Token<'a>] {
        &self.tokens
    }

    fn extend(&self, _: &str) -> Result<Self, ExtendError> {
        Err(NotSupported)
    }
}

pub struct FileProviderPool(Arena<Box<String>>);

impl FileProviderPool {
    pub fn new() -> FileProviderPool {
        FileProviderPool(Arena::new())
    }

    pub fn provider(&self, path: PathBuf) -> Result<FileProvider, ExtendError> {
        let source = fs::read_to_string(&path)
            .map_err(|_| FailedToRead(path.to_string_lossy().to_string()))?;

        let tokens = {
            let item = self.0.alloc(Box::new(source));

            lex(&**item).map_err(|e| LexerFailed(e))?
        };

        Ok(FileProvider {
            pool: self,
            tokens,
            path
        })
    }
}

pub struct FileProvider<'a> {
    pool: &'a FileProviderPool,
    tokens: Vec<Token<'a>>,
    path: PathBuf
}


impl<'a> TokenProvider<'a> for FileProvider<'a> {
    fn get(&self) -> &[Token<'a>] {
        &self.tokens
    }

    fn extend(&self, path: &str) -> Result<Self, ExtendError> {
        let file = self.path.with_extension(path);

        self.pool.provider(file)
    }
}
