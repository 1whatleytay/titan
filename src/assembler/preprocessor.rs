use crate::assembler::cursor::{is_adjacent_kind, LexerCursor};
use crate::assembler::lexer::SymbolName::Owned;
use crate::assembler::lexer::TokenKind::{
    Colon, Directive, LeftBrace, NewLine, Parameter, RightBrace, Symbol,
};
use crate::assembler::lexer::{LexerError, Location, StrippedKind, SymbolName, Token, TokenKind};
use crate::assembler::preprocessor::PreprocessorReason::{
    EndOfFile, ExpectedLeftBrace, ExpectedParameter, ExpectedRightBrace, ExpectedString,
    ExpectedSymbol, FailedToFindFile, FailedToLexFile, IncludeUnsupported, MacroParameterCount,
    MacroUnknownParameter, RecursiveExpansion, RecursiveInclude,
};
use crate::assembler::source::{ExtendError, TokenProvider};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use PreprocessorReason::NoFilePathAssociated;

#[derive(Debug)]
pub enum PreprocessorReason {
    EndOfFile,
    ExpectedSymbol(StrippedKind),
    ExpectedParameter(StrippedKind),
    ExpectedLeftBrace(StrippedKind),
    ExpectedRightBrace(StrippedKind),
    ExpectedString(StrippedKind),
    RecursiveExpansion,
    MacroParameterCount(usize, usize), // expected, actual
    MacroUnknownParameter(String),
    IncludeUnsupported,
    NoFilePathAssociated,
    FailedToFindFile(String),
    FailedToLexFile(LexerError),
    RecursiveInclude,
}

impl Display for PreprocessorReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EndOfFile => write!(
                f,
                "A required token is missing for the preprocessor, instead got end-of-file"
            ),
            ExpectedSymbol(kind) => write!(f, "Expected a symbol (name) token, but found {kind}"),
            ExpectedParameter(kind) => {
                write!(f, "Expected a parameter (%param) token, but found {kind}")
            }
            ExpectedLeftBrace(kind) => write!(f, "Expected a left brace, but found {kind}"),
            ExpectedRightBrace(kind) => write!(f, "Expected a right brace, but found {kind}"),
            ExpectedString(kind) => write!(f, "Expected a string brace, but found {kind}"),
            RecursiveExpansion => write!(
                f,
                "Macro recursively calls itself, so preprocessor has stopped expanding"
            ),
            MacroParameterCount(expected, actual) => write!(
                f,
                "Expected {expected} macro parameters, but passed {actual}"
            ),
            MacroUnknownParameter(name) => write!(f, "Unknown macro parameter named \"{name}\""),
            IncludeUnsupported => write!(f, "Cannot include because this file is not saved to disk. Please save the file to use include."),
            NoFilePathAssociated => write!(f, "This file is not saved to disk, so there is no path for this file."),
            FailedToFindFile(name) => write!(f, "Failed to find file \"{name}\""),
            FailedToLexFile(error) => write!(f, "File has invalid format, {error}"),
            RecursiveInclude => write!(f, "Include is recursive (includes itself), this is not allowed")
        }
    }
}

#[derive(Debug)]
pub struct PreprocessorError {
    pub location: Location,
    pub reason: PreprocessorReason,
}

impl Display for PreprocessorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.reason.fmt(f)
    }
}

impl Error for PreprocessorError {}

#[derive(Clone)]
struct Macro<'a> {
    name: String,
    parameters: Vec<&'a str>,
    labels: HashSet<String>,
    items: Vec<Token<'a>>,
}

impl<'a> Macro<'a> {
    fn new(name: String) -> Macro<'a> {
        Macro {
            name,
            parameters: vec![],
            labels: HashSet::new(),
            items: vec![],
        }
    }
}

struct Cache<'a> {
    seed: usize,
    tokens: HashMap<String, Vec<TokenKind<'a>>>,
    macros: HashMap<String, Rc<Macro<'a>>>,
    expanding: HashSet<String>,
}

impl<'a> Cache<'a> {
    fn new() -> Cache<'a> {
        Cache {
            seed: 0,
            tokens: HashMap::new(),
            macros: HashMap::new(),
            expanding: HashSet::new(),
        }
    }
}

fn consume_eqv<'a>(
    iter: &mut LexerCursor<'a, '_>,
) -> Result<(String, Vec<TokenKind<'a>>), PreprocessorReason> {
    let Some(symbol) = iter.next_adjacent() else {
        return Err(EndOfFile);
    };

    let Symbol(key) = &symbol.kind else {
        return Err(ExpectedSymbol(symbol.kind.strip()));
    };
    let value = iter
        .collect_without(|kind| kind == &NewLine)
        .into_iter()
        .map(|token| token.kind.clone())
        .collect();

    Ok((key.get().to_string(), value))
}

fn consume_macro<'a>(iter: &mut LexerCursor<'a, '_>) -> Result<Macro<'a>, PreprocessorReason> {
    let Some(symbol) = iter.next_adjacent() else {
        return Err(EndOfFile);
    };
    let Symbol(name) = &symbol.kind else {
        return Err(ExpectedSymbol(symbol.kind.strip()));
    };

    let mut result = Macro::new(name.get().to_string());

    let Some(left_brace) = iter.next_adjacent() else {
        return Err(EndOfFile);
    };
    if left_brace.kind != LeftBrace {
        return Err(ExpectedLeftBrace(left_brace.kind.strip()));
    };

    loop {
        let Some(next) = iter.next_adjacent() else {
            return Err(EndOfFile);
        };

        match next.kind {
            LeftBrace => continue,
            RightBrace => continue,
            Parameter(name) => result.parameters.push(name),
            NewLine => break,
            _ => return Err(ExpectedParameter(next.kind.strip())),
        }
    }

    let mut body: Vec<Token> = vec![];

    let mut stop = false;
    while !stop {
        let mut items = iter.collect_until(is_adjacent_kind);

        let last = items.last().ok_or(EndOfFile)?;

        stop = match &last.kind {
            Symbol(name) => {
                let mut post_items = iter.collect_until(is_adjacent_kind);

                if post_items
                    .last()
                    .map(|last| last.kind == Colon)
                    .unwrap_or(false)
                {
                    result.labels.insert(name.get().to_string());
                }

                items.append(&mut post_items);

                false
            }
            Directive(directive) if *directive == "end_macro" => true,
            _ => false,
        };

        body.extend(items.into_iter().cloned());
    }

    body.pop();

    result.items = body;

    Ok(result)
}

fn consume_include<'a, P: TokenProvider<'a>>(
    iter: &mut LexerCursor<'a, '_>,
    provider: &P,
    cache: &mut Cache<'a>,
) -> Result<Vec<Token<'a>>, PreprocessorReason> {
    let next = iter.next().ok_or(EndOfFile)?;

    let TokenKind::StringLiteral(path) = &next.kind else {
        return Err(ExpectedString(next.kind.strip()));
    };

    let new_provider = provider.extend(path).map_err(|e| match e {
        ExtendError::NotSupported => IncludeUnsupported,
        ExtendError::FailedToRead(f) => FailedToFindFile(f),
        ExtendError::LexerFailed(e) => FailedToLexFile(e),
        ExtendError::RecursiveInclude => RecursiveInclude,
    })?;

    preprocess_cached(&new_provider, new_provider.get(), cache).map_err(|e| e.reason)
    // strip any location info ATM
}

fn expand_macro<'a, P: TokenProvider<'a>>(
    macro_info: Rc<Macro<'a>>,
    parameters: Vec<Vec<Token<'a>>>,
    provider: &P,
    cache: &mut Cache<'a>,
) -> Result<Vec<Token<'a>>, PreprocessorReason> {
    if cache.expanding.contains(&macro_info.name) {
        return Err(RecursiveExpansion);
    }

    cache.expanding.insert(macro_info.name.clone());

    if macro_info.parameters.len() != parameters.len() {
        return Err(MacroParameterCount(
            macro_info.parameters.len(),
            parameters.len(),
        ));
    }

    let label_names: HashMap<&str, String> = macro_info
        .labels
        .iter()
        .map(|name| {
            (
                &name[..],
                format!("_M{}_{}", name, {
                    cache.seed += 1;
                    cache.seed
                }),
            )
        })
        .collect();

    let mut parameter_map: HashMap<&'a str, Vec<TokenKind>> = HashMap::new();

    for (index, value) in parameters.into_iter().enumerate() {
        let name = macro_info.parameters[index];

        parameter_map.insert(name, value.into_iter().map(|token| token.kind).collect());
    }

    let mut result = vec![];

    for token in &macro_info.items {
        let mapped_kind = match &token.kind {
            Parameter(name) => {
                let kinds = parameter_map
                    .get(name)
                    .ok_or_else(|| MacroUnknownParameter(name.to_string()))?;

                for kind in kinds {
                    result.push(Token {
                        location: token.location,
                        kind: kind.clone(),
                    });
                }

                // isolate dealing with vectors to this branch
                continue;
            }
            Symbol(name) => {
                if let Some(new_name) = label_names.get(name.get()) {
                    Symbol(Owned(new_name.clone()))
                } else {
                    token.kind.clone()
                }
            }
            _ => token.kind.clone(),
        };

        result.push(Token {
            location: token.location,
            kind: mapped_kind,
        });
    }

    let result = preprocess_cached(provider, &result, cache).map_err(|err| err.reason)?;

    cache.expanding.remove(&macro_info.name);

    Ok(result)
}

fn handle_symbol<'a, P: TokenProvider<'a>>(
    name: &SymbolName<'a>,
    location: Location,
    iter: &mut LexerCursor<'a, '_>,
    provider: &P,
    cache: &mut Cache<'a>,
) -> Result<Vec<Token<'a>>, PreprocessorReason> {
    if let Some(tokens) = cache.tokens.get(name.get()) {
        return Ok(tokens
            .iter()
            .map(|kind| Token {
                location,
                kind: kind.clone(),
            })
            .collect());
    }

    // Consumes nothing until we call iter.consume_until(position)
    let (position, token) = iter.peek_adjacent();

    let Some(last) = token else {
        return Ok(vec![Token {
            location,
            kind: Symbol(name.clone()),
        }]);
    };

    let start = iter.get_position();

    match last.kind {
        LeftBrace => {
            iter.next(); /* pop */
        }
        _ => {
            return Ok(vec![Token {
                location,
                kind: Symbol(name.clone()),
            }])
        }
    }

    // Treat as a macro!
    iter.consume_until(position); // includes the item

    let Some(macro_info) = cache.macros.get(name.get()) else {
        iter.set_position(start);
        return Ok(vec![Token {
            location,
            kind: Symbol(name.clone()),
        }]);
    };

    let mut parameters = vec![];

    loop {
        let Some(next) = iter.next_adjacent() else {
            return Err(EndOfFile);
        };

        match next.kind {
            RightBrace => break,
            NewLine => return Err(ExpectedRightBrace(next.kind.strip())),
            TokenKind::Plus | TokenKind::Minus => {
                let mut result = vec![next.clone()];

                if let (position, Some(number)) = iter.peek_adjacent() {
                    if let TokenKind::IntegerLiteral(_) = number.kind {
                        iter.set_position(position);
                        iter.next();

                        result.push(number.clone());
                    }
                }

                parameters.push(result);
            }
            _ => parameters.push(vec![next.clone()]),
        }
    }

    expand_macro(macro_info.clone(), parameters, provider, cache)
}

fn preprocess_cached<'a, P: TokenProvider<'a>>(
    provider: &P,
    items: &[Token<'a>],
    cache: &mut Cache<'a>,
) -> Result<Vec<Token<'a>>, PreprocessorError> {
    let mut iter = LexerCursor::new(items);
    let mut result: Vec<Token> = Vec::with_capacity(items.len());

    let watched_directives = HashSet::from(["eqv", "macro", "include", "file_path"]);

    while let Some(element) = iter.next() {
        let fail = |reason: PreprocessorReason| PreprocessorError {
            location: element.location,
            reason,
        };

        match &element.kind {
            Directive(directive) if watched_directives.contains(directive) => match *directive {
                "eqv" => {
                    let (key, value) = consume_eqv(&mut iter).map_err(fail)?;

                    cache.tokens.insert(key, value);
                }
                "macro" => {
                    let value = consume_macro(&mut iter).map_err(fail)?;

                    cache.macros.insert(value.name.clone(), Rc::new(value));
                }
                "include" => {
                    let tokens = consume_include(&mut iter, provider, cache).map_err(fail)?;

                    result.extend(tokens);
                }
                "file_path" => {
                    let path = provider.get_path();

                    if let Some(path) = path {
                        result.push(Token {
                            location: element.location,
                            kind: TokenKind::StringLiteral(path),
                        })
                    } else {
                        return Err(fail(NoFilePathAssociated));
                    }
                }
                _ => panic!(), // ??
            },
            Symbol(name) => {
                let mut elements =
                    handle_symbol(name, element.location, &mut iter, provider, cache)
                        .map_err(fail)?;

                result.append(&mut elements)
            }

            _ => result.push(element.clone()),
        }
    }

    Ok(result)
}

pub fn mark_parameters_as_error(result: Vec<Token>) -> Result<Vec<Token>, PreprocessorError> {
    for token in &result {
        if let Parameter(name) = token.kind {
            return Err(PreprocessorError {
                location: token.location,
                reason: MacroUnknownParameter(name.to_string()),
            });
        }
    }

    Ok(result)
}

pub fn preprocess<'a, P: TokenProvider<'a>>(
    provider: &P,
) -> Result<Vec<Token<'a>>, PreprocessorError> {
    let mut cache = Cache::new();

    preprocess_cached(provider, provider.get(), &mut cache).and_then(mark_parameters_as_error)
}
