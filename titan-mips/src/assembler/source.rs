use crate::assembler::lexer::{lex, lex_with_source, LexerError, Token};
use crate::assembler::source::ExtendError::{
    FailedToRead, LexerFailed, NotSupported, RecursiveInclude,
};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use typed_arena::Arena;

pub enum ExtendError {
    NotSupported,
    FailedToRead(String),
    LexerFailed(LexerError),
    RecursiveInclude,
}

pub trait TokenProvider<'a>: Sized {
    fn id(&self) -> usize;
    fn get(&self) -> &[Token<'a>];

    fn get_path(&self) -> Option<String>;
    fn extend(&self, path: &str) -> Result<Self, ExtendError>;
}

pub struct HoldingProvider<'a> {
    tokens: Vec<Token<'a>>,
}

impl<'a> HoldingProvider<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> HoldingProvider<'a> {
        HoldingProvider { tokens }
    }

    pub fn from_source(source: &str) -> Result<HoldingProvider, LexerError> {
        Ok(HoldingProvider {
            tokens: lex(source)?,
        })
    }
}

impl<'a> TokenProvider<'a> for HoldingProvider<'a> {
    fn id(&self) -> usize {
        0
    }
    fn get(&self) -> &[Token<'a>] {
        &self.tokens
    }

    fn get_path(&self) -> Option<String> {
        None
    }

    fn extend(&self, _: &str) -> Result<Self, ExtendError> {
        Err(NotSupported)
    }
}

pub struct FileProviderSource {
    pub id: usize,
    pub path: Rc<PathBuf>,
    pub source: Rc<String>,
}

pub struct FileProviderPool {
    arena: Arena<Rc<String>>,
    sources: RefCell<Vec<FileProviderSource>>,
}

impl Default for FileProviderPool {
    fn default() -> Self {
        Self::new()
    }
}

impl FileProviderPool {
    pub fn new() -> FileProviderPool {
        FileProviderPool {
            arena: Arena::new(),
            sources: RefCell::new(Vec::new()),
        }
    }

    pub fn provider_sourced(
        &self,
        source: String,
        path: Rc<PathBuf>,
    ) -> Result<FileInfo, LexerError> {
        let (id, tokens) = {
            let source = Rc::new(source);

            let mut items = self.sources.borrow_mut();
            let id = items.len();

            items.push(FileProviderSource {
                id,
                path: path.clone(),
                source: source.clone(),
            });

            let item = self.arena.alloc(source);

            (id, lex_with_source(item, id)?)
        };

        Ok(FileInfo {
            pool: self,
            source: id,
            tokens,
            path,
        })
    }

    pub fn provider(&self, path: Rc<PathBuf>) -> Result<FileInfo, ExtendError> {
        let source = fs::read_to_string(&*path)
            .map_err(|_| FailedToRead(path.to_string_lossy().to_string()))?;

        self.provider_sourced(source, path).map_err(LexerFailed)
    }
}

pub struct FileInfo<'a> {
    pool: &'a FileProviderPool,
    source: usize,
    tokens: Vec<Token<'a>>,
    path: Rc<PathBuf>,
}

impl<'a> FileInfo<'a> {
    pub fn to_provider(self) -> FileProvider<'a> {
        // Don't canonicalize.
        let path = self.path.clone();

        FileProvider {
            info: self,
            history: HashSet::from([path]),
        }
    }
}

pub struct FileProvider<'a> {
    info: FileInfo<'a>,
    history: HashSet<Rc<PathBuf>>,
}

impl<'a> TokenProvider<'a> for FileProvider<'a> {
    fn id(&self) -> usize {
        self.info.source
    }
    fn get(&self) -> &[Token<'a>] {
        &self.info.tokens
    }

    fn get_path(&self) -> Option<String> {
        Some(self.info.path.to_string_lossy().to_string())
    }

    fn extend(&self, path: &str) -> Result<Self, ExtendError> {
        let file = self
            .info
            .path
            .parent()
            .unwrap_or(&self.info.path)
            .join(path);

        let file = fs::canonicalize(&file)
            .map_err(|_| FailedToRead(file.to_string_lossy().to_string()))?;

        let file = Rc::new(file);

        let mut history = self.history.clone();

        if !history.insert(file.clone()) {
            return Err(RecursiveInclude);
        }

        Ok(FileProvider {
            info: self.info.pool.provider(file)?,
            history,
        })
    }
}
