use crate::assembler::source::ExtendError::{
    FailedToRead, LexerFailed, NotSupported, RecursiveInclude,
};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use typed_arena::Arena;

pub enum ExtendError<LexerError> {
    NotSupported,
    FailedToRead(String),
    LexerFailed(LexerError),
    RecursiveInclude,
}

pub trait TokenProvider<Token, LexerError>: Sized {
    fn id(&self) -> usize;
    fn get(&self) -> &[Token];

    fn get_path(&self) -> Option<String>;
    fn extend(&self, path: &str) -> Result<Self, ExtendError<LexerError>>;
}

pub struct HoldingProvider<Token> {
    tokens: Vec<Token>,
}

impl<Token> HoldingProvider<Token> {
    pub fn new(tokens: Vec<Token>) -> HoldingProvider<Token> {
        HoldingProvider { tokens }
    }
}

impl<Token, LexerError> TokenProvider<Token, LexerError> for HoldingProvider<Token> {
    fn id(&self) -> usize {
        0
    }
    fn get(&self) -> &[Token] {
        &self.tokens
    }

    fn get_path(&self) -> Option<String> {
        None
    }

    fn extend(&self, _: &str) -> Result<Self, ExtendError<LexerError>> {
        Err(NotSupported)
    }
}

pub struct FileProviderSource {
    pub id: usize,
    pub path: Rc<PathBuf>,
    pub source: Rc<String>,
}

pub trait LexerProvider<'a, Token, LexerError> {
    fn lex(&self, source: &'a str, id: usize) -> Result<Vec<Token>, LexerError>;
}

pub struct FileProviderPool<Lexer> {
    lexer: Lexer,
    arena: Arena<Rc<String>>,
    sources: RefCell<Vec<FileProviderSource>>,
}

impl<Lexer> FileProviderPool<Lexer> {
    pub fn new<'a, Token, LexerError>(lexer: Lexer) -> Self where Lexer: LexerProvider<'a, Token, LexerError> {
        Self {
            lexer,
            arena: Arena::default(),
            sources: RefCell::default(),
        }
    }

    pub fn provider_sourced<'a, Token, LexerError>(
        &'a self,
        source: String,
        path: Rc<PathBuf>,
    ) -> Result<FileInfo<'a, Token, Lexer>, LexerError> where Lexer: LexerProvider<'a, Token, LexerError> {
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

            (id, self.lexer.lex(item, id)?)
        };

        Ok(FileInfo {
            pool: self,
            source: id,
            tokens,
            path,
        })
    }

    pub fn provider<'a, Token, LexerError>(
        &'a self, path: Rc<PathBuf>
    ) -> Result<FileInfo<'a, Token, Lexer>, ExtendError<LexerError>> where Lexer: LexerProvider<'a, Token, LexerError> {
        let source = fs::read_to_string(&*path)
            .map_err(|_| FailedToRead(path.to_string_lossy().to_string()))?;

        self.provider_sourced(source, path).map_err(LexerFailed)
    }
}

pub struct FileInfo<'a, Token, Lexer> {
    pool: &'a FileProviderPool<Lexer>,
    source: usize,
    tokens: Vec<Token>,
    path: Rc<PathBuf>,
}

impl<'a, Token, Lexer> FileInfo<'a, Token, Lexer> {
    pub fn to_provider(self) -> FileProvider<'a, Token, Lexer> {
        // Don't canonicalize.
        let path = self.path.clone();

        FileProvider {
            info: self,
            history: HashSet::from([path]),
        }
    }
}

pub struct FileProvider<'a, Token, Lexer> {
    info: FileInfo<'a, Token, Lexer>,
    history: HashSet<Rc<PathBuf>>,
}

impl<'a, Token, LexerError, Lexer: LexerProvider<'a, Token, LexerError>> TokenProvider<Token, LexerError> for FileProvider<'a, Token, Lexer> {
    fn id(&self) -> usize {
        self.info.source
    }
    fn get(&self) -> &[Token] {
        &self.info.tokens
    }

    fn get_path(&self) -> Option<String> {
        Some(self.info.path.to_string_lossy().to_string())
    }

    fn extend(&self, path: &str) -> Result<Self, ExtendError<LexerError>> {
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
