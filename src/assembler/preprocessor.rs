use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::assembler::lexer::{StrippedKind, SymbolName, Token, TokenKind};
use crate::assembler::lexer::SymbolName::Owned;
use crate::assembler::lexer::TokenKind::{
    Colon, LeftBrace, Parameter, RightBrace, NewLine, Symbol, Directive
};
use crate::assembler::cursor::{is_adjacent_kind, LexerCursor};
use crate::assembler::preprocessor::PreprocessorReason::{
    EndOfFile, ExpectedSymbol, ExpectedParameter, ExpectedLeftBrace, ExpectedRightBrace,
    MacroUnknown, MacroParameterCount, MacroUnknownParameter, RecursiveExpansion
};

#[derive(Debug)]
pub enum PreprocessorReason {
    EndOfFile,
    ExpectedSymbol(StrippedKind),
    ExpectedParameter(StrippedKind),
    ExpectedLeftBrace(StrippedKind),
    ExpectedRightBrace(StrippedKind),
    RecursiveExpansion,
    MacroUnknown(String),
    MacroParameterCount(usize, usize), // expected, actual
    MacroUnknownParameter(String),
}

impl Display for PreprocessorReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EndOfFile =>
                write!(f, "A required token is missing for the preprocessor, nstead got end-of-file"),
            ExpectedSymbol(kind) =>
                write!(f, "Expected a symbol (name) token, but found {}", kind),
            ExpectedParameter(kind) =>
                write!(f, "Expected a parameter (%param) token, but found {}", kind),
            ExpectedLeftBrace(kind) =>
                write!(f, "Expected a left brace, but found {}", kind),
            ExpectedRightBrace(kind) =>
                write!(f, "Expected a right brace, but found {}", kind),
            RecursiveExpansion =>
                write!(f, "Macro recursively calls itself, so preprocessor has stopped expanding"),
            MacroUnknown(name) =>
                write!(f, "Could not find a macro named \"{}\"", name),
            MacroParameterCount(expected, actual) =>
                write!(f, "Expected {} macro parameters, but passed {}", expected, actual),
            MacroUnknownParameter(name) =>
                write!(f, "Unknown macro parameter named \"{}\"", name),
        }
    }
}

#[derive(Debug)]
pub struct PreprocessorError {
    pub start: usize,
    pub reason: PreprocessorReason
}

impl Display for PreprocessorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.reason.fmt(f)
    }
}

impl Error for PreprocessorError { }


#[derive(Clone)]
struct Macro<'a> {
    name: String,
    parameters: Vec<&'a str>,
    labels: HashSet<String>,
    items: Vec<Token<'a>>
}

impl<'a> Macro<'a> {
    fn new(name: String) -> Macro<'a> {
        Macro { name, parameters: vec![], labels: HashSet::new(), items: vec![] }
    }
}

struct Cache<'a> {
    seed: usize,
    tokens: HashMap<String, Vec<TokenKind<'a>>>,
    macros: HashMap<String, Rc<Macro<'a>>>,
    expanding: HashSet<String>
}

impl<'a> Cache<'a> {
    fn new() -> Cache<'a> {
        Cache {
            seed: 0,
            tokens: HashMap::new(),
            macros: HashMap::new(),
            expanding: HashSet::new()
        }
    }
}

fn consume_eqv<'a, 'b>(
    iter: &mut LexerCursor<'a, 'b>
) -> Result<(String, Vec<TokenKind<'a>>), PreprocessorReason> {
    let Some(symbol) = iter.next_adjacent() else { return Err(EndOfFile) };

    let Symbol(key) = &symbol.kind else { return Err(ExpectedSymbol(symbol.kind.strip())) };
    let value = iter.collect_without(|kind| kind == &NewLine).into_iter()
        .map(|token| token.kind.clone())
        .collect();

    Ok((key.get().to_string(), value))
}

fn consume_macro<'a>(iter: &mut LexerCursor<'a, '_>) -> Result<Macro<'a>, PreprocessorReason> {
    let Some(symbol) = iter.next_adjacent() else { return Err(EndOfFile) };
    let Symbol(name) = &symbol.kind else { return Err(ExpectedSymbol(symbol.kind.strip())) };

    let mut result = Macro::new(name.get().to_string());

    let Some(left_brace) = iter.next_adjacent() else { return Err(EndOfFile) };
    if left_brace.kind != LeftBrace { return Err(ExpectedLeftBrace(left_brace.kind.strip())) };

    loop {
        let Some(next) = iter.next_adjacent() else { return Err(EndOfFile) };

        match next.kind {
            RightBrace => break,
            Parameter(name) => { result.parameters.push(name) },
            NewLine => return Err(ExpectedRightBrace(next.kind.strip())),
            _ => return Err(ExpectedParameter(next.kind.strip()))
        }
    }

    let mut body: Vec<Token> = vec![];

    let mut stop = false;
    while !stop {
        let mut items = iter.collect_until(is_adjacent_kind);

        let last = items.last().ok_or_else(|| EndOfFile)?;

        stop = match &last.kind {
            Symbol(name) => {
                let mut post_items = iter.collect_until(is_adjacent_kind);

                if post_items.last().map(|last| last.kind == Colon).unwrap_or(false) {
                    result.labels.insert(name.get().to_string());
                }

                items.append(&mut post_items);

                false
            }
            Directive(directive) if *directive == "end_macro" => true,
            _ => false
        };

        body.extend(items.into_iter().cloned());
    }

    body.pop();

    result.items = body;

    Ok(result)
}

fn expand_macro<'a>(
    macro_info: Rc<Macro<'a>>, parameters: Vec<Token<'a>>, cache: &mut Cache<'a>
) -> Result<Vec<Token<'a>>, PreprocessorReason> {
    if cache.expanding.contains(&macro_info.name) {
        return Err(RecursiveExpansion)
    }

    cache.expanding.insert(macro_info.name.clone());

    if macro_info.parameters.len() != parameters.len() {
        return Err(MacroParameterCount(macro_info.parameters.len(), parameters.len()))
    }

    let label_names: HashMap<&str, String> = macro_info.labels.iter()
        .map(|name| (&name[..], format!("_M{}_{}", name, { cache.seed += 1; cache.seed })))
        .collect();

    let mut parameter_map: HashMap<&'a str, TokenKind> = HashMap::new();

    for (index, value) in parameters.into_iter().enumerate() {
        let name = macro_info.parameters[index];

        parameter_map.insert(name, value.kind);
    }

    let mut result = vec![];

    for token in &macro_info.items {
        let mapped_kind = match &token.kind {
            Parameter(name) => parameter_map.get(name).cloned()
                .ok_or_else(|| MacroUnknownParameter(name.to_string()))?,
            Symbol(name) => {
                if let Some(new_name) = label_names.get(name.get()) {
                    Symbol(Owned(new_name.clone()))
                } else {
                    token.kind.clone()
                }
            }
            _ => token.kind.clone()
        };

        result.push(Token { start: token.start, kind: mapped_kind });
    }

    let result = preprocess_cached(&result, cache)
        .map_err(|err| err.reason)?;

    cache.expanding.remove(&macro_info.name);

    Ok(result)
}

fn handle_symbol<'a>(
    name: &SymbolName<'a>, start: usize, iter: &mut LexerCursor<'a, '_>, cache: &mut Cache<'a>
) -> Result<Vec<Token<'a>>, PreprocessorReason> {
    if let Some(tokens) = cache.tokens.get(name.get()) {
        return Ok(tokens.iter().map(|kind| Token { start, kind: kind.clone() }).collect())
    }

    // Consumes nothing until we call iter.consume_until(position)
    let (position, token) = iter.peek_adjacent();

    let Some(last) = token else {
        return Ok(vec![Token { start, kind: Symbol(name.clone()) }])
    };

    match last.kind {
        LeftBrace => { iter.next(); /* pop */ },
        _ => return Ok(vec![Token { start, kind: Symbol(name.clone()) }])
    }

    // Treat as a macro!
    iter.consume_until(position); // includes the item

    let Some(macro_info) = cache.macros.get(name.get()) else {
        return Err(MacroUnknown(name.get().to_string()))
    };

    let mut parameters = vec![];

    loop {
        let Some(next) = iter.next_adjacent() else { return Err(EndOfFile) };

        match next.kind {
            RightBrace => break,
            NewLine => return Err(ExpectedRightBrace(next.kind.strip())),
            _ => parameters.push(next.clone())
        }
    };

    expand_macro(macro_info.clone(), parameters, cache)
}

fn preprocess_cached<'a, 'b>(
    items: &[Token<'a>], cache: &'b mut Cache<'a>
) -> Result<Vec<Token<'a>>, PreprocessorError> {
    let mut iter = LexerCursor::new(items);
    let mut result: Vec<Token> = vec![];

    result.reserve(items.len());

    let watched_directives = HashSet::from(["eqv", "macro"]);

    while let Some(element) = iter.next() {
        let fail = |reason: PreprocessorReason| PreprocessorError {
            start: element.start, reason
        };

        match &element.kind {
            Directive(directive) if watched_directives.contains(directive) => {
                match *directive {
                    "eqv" => {
                        let (key, value) = consume_eqv(&mut iter)
                            .map_err(|err| fail(err))?;

                        cache.tokens.insert(key, value);
                    },
                    "macro" => {
                        let value = consume_macro(&mut iter)
                            .map_err(|err| fail(err))?;

                        cache.macros.insert(value.name.clone(), Rc::new(value));
                    },
                    _ => panic!()
                }
            }
            Symbol(name) => {
                let mut elements = handle_symbol(
                    name, element.start, &mut iter, cache
                ).map_err(|err| fail(err))?;

                result.append(&mut elements)
            },

            _ => result.push(element.clone())
        }
    }

    Ok(result)
}

pub fn preprocess<'a>(items: &[Token<'a>]) -> Result<Vec<Token<'a>>, PreprocessorError> {
    let mut cache = Cache::new();

    preprocess_cached(items, &mut cache)
}
