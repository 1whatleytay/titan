use std::cell::RefCell;
use std::fs;
use typed_arena::Arena;
use std::path::PathBuf;
use std::rc::Rc;
use crate::assembler::lexer::{lex, lex_with_source, LexerError, Token};
use crate::assembler::source::ExtendError::{FailedToRead, LexerFailed, NotSupported};

pub enum ExtendError {
    NotSupported,
    FailedToRead(String),
    LexerFailed(LexerError)
}

pub trait TokenProvider<'a>: Sized {
    fn id(&self) -> usize;
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
    fn id(&self) -> usize { 0 }
    fn get(&self) -> &[Token<'a>] {
        &self.tokens
    }

    fn extend(&self, _: &str) -> Result<Self, ExtendError> {
        Err(NotSupported)
    }
}

pub struct FileProviderSource {
    pub id: usize,
    pub path: PathBuf,
    pub source: Rc<String>
}

pub struct FileProviderPool {
    arena: Arena<Rc<String>>,
    sources: RefCell<Vec<FileProviderSource>>
}

impl FileProviderPool {
    pub fn new() -> FileProviderPool {
        FileProviderPool {
            arena: Arena::new(),
            sources: RefCell::new(Vec::new())
        }
    }

    pub fn provider_sourced(&self, source: String, path: PathBuf) -> Result<FileProvider, LexerError> {
        let (id, tokens) = {
            let source = Rc::new(source);

            let mut items = self.sources.borrow_mut();
            let id = items.len();

            items.push(FileProviderSource {
                id, path: path.clone(), source: source.clone()
            });

            let item = self.arena.alloc(source);

            (id, lex_with_source(&**item, id)?)
        };

        Ok(FileProvider {
            pool: self,
            source: id,
            tokens,
            path
        })
    }

    pub fn provider(&self, path: PathBuf) -> Result<FileProvider, ExtendError> {
        let source = fs::read_to_string(&path)
            .map_err(|_| FailedToRead(path.to_string_lossy().to_string()))?;

        self.provider_sourced(source, path).map_err(|e| LexerFailed(e))
    }
}

pub struct FileProvider<'a> {
    pool: &'a FileProviderPool,
    source: usize,
    tokens: Vec<Token<'a>>,
    path: PathBuf
}


impl<'a> TokenProvider<'a> for FileProvider<'a> {
    fn id(&self) -> usize { self.source }
    fn get(&self) -> &[Token<'a>] {
        &self.tokens
    }

    fn extend(&self, path: &str) -> Result<Self, ExtendError> {
        let file = self.path.parent()
            .unwrap_or(&self.path)
            .with_extension(path);

        self.pool.provider(file)
    }
}
