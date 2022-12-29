use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::assembler::lexer::{Token, TokenKind};
use crate::assembler::lexer::TokenKind::{LeftBrace, Parameter, RightBrace, NewLine, Symbol, Directive};
use crate::assembler::lexer_seek::{is_adjacent_kind, LexerSeek};
use crate::assembler::preprocessor::PreprocessorReason::{EndOfFile, ExpectedLeftBrace, ExpectedParameter, ExpectedRightBrace, ExpectedSymbol, LoneParameter, MacroUnknown, MacroParameterCount, MacroUnknownParameter};

#[derive(Debug)]
pub enum PreprocessorReason {
    LoneParameter,
    EndOfFile,
    ExpectedSymbol,
    ExpectedParameter,
    ExpectedLeftBrace,
    ExpectedRightBrace,
    MacroUnknown(String),
    MacroParameterCount(usize, usize), // expected, actual
    MacroUnknownParameter(String),
}

#[derive(Debug)]
pub struct PreprocessorError<'a> {
    start: &'a str,
    reason: PreprocessorReason
}

struct Macro<'a> {
    parameters: Vec<&'a str>,
    items: Vec<TokenKind<'a>>
}

impl<'a> Macro<'a> {
    fn new() -> Macro<'a> {
        Macro { parameters: vec![], items: vec![] }
    }
}

struct Cache<'a> {
    tokens: HashMap<&'a str, TokenKind<'a>>,
    macros: HashMap<&'a str, Macro<'a>>
}

impl<'a> Cache<'a> {
    fn new() -> Cache<'a> {
        Cache { tokens: HashMap::new(), macros: HashMap::new() }
    }
}

impl<'a> Display for PreprocessorError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.reason)
    }
}

impl<'a> Error for PreprocessorError<'a> { }

fn consume_eqv<'a, 'b, T>(iter: &'b mut T) -> Result<(&'a str, TokenKind<'a>), PreprocessorReason>
    where T: LexerSeek<'a> {
    let Some(symbol) = iter.next_adjacent() else { return Err(EndOfFile) };
    let Some(value) = iter.next_adjacent() else { return Err(EndOfFile) };

    let Symbol(key) = symbol.kind else { return Err(ExpectedSymbol) };

    Ok((key, value.kind))
}

fn consume_macro<'a, 'b, T>(iter: &'b mut T) -> Result<(&'a str, Macro<'a>), PreprocessorReason>
    where T: LexerSeek<'a> {
    let mut result = Macro::new();

    let Some(symbol) = iter.next_adjacent() else { return Err(EndOfFile) };
    let Symbol(name) = symbol.kind else { return Err(ExpectedSymbol) };

    let Some(left_brace) = iter.next_adjacent() else { return Err(EndOfFile) };
    if left_brace.kind != LeftBrace { return Err(ExpectedLeftBrace) };

    loop {
        let Some(next) = iter.next_adjacent() else { return Err(EndOfFile) };

        match next.kind {
            RightBrace => break,
            Parameter(name) => { result.parameters.push(name) },
            NewLine => return Err(ExpectedRightBrace),
            _ => return Err(ExpectedParameter)
        }
    }

    result.items = iter.collect_until(|kind| match kind {
        Directive(directive) if *directive == "end_macro" => true,
        _ => false
    }).into_iter().map(|item| item.kind).collect();

    match result.items.last().ok_or_else(|| EndOfFile)? {
        Directive(directive) if *directive != "end_macro" => return Err(EndOfFile),
        _ => { }
    }

    result.items.pop();

    Ok((name, result))
}

fn handle_symbol<'a, 'b, 'c, T>(name: &'a str, element: Token<'a>, iter: &'b mut T, cache: &'c Cache<'a>)
                                -> Result<Vec<Token<'a>>, PreprocessorReason> where T: LexerSeek<'a> {
    if let Some(token) = cache.tokens.get(name) {
        return Ok(vec![Token { start: element.start, kind: token.clone() }])
    }

    let elements = iter.collect_until(|kind| is_adjacent_kind(kind));

    fn concat<'a>(element: Token<'a>, mut elements: Vec<Token<'a>>) -> Vec<Token<'a>> {
        let mut first = vec![element];
        first.append(&mut elements);

        first
    }

    let Some(last) = elements.last() else {
        return Ok(concat(element, elements))
    };

    if last.kind != LeftBrace {
        return Ok(concat(element, elements))
    };

    // Treat as a macro!
    let Some(macro_info) = cache.macros.get(name) else {
        return Err(MacroUnknown(name.to_string()))
    };

    let mut parameters = vec![];

    loop {
        let Some(next) = iter.next_adjacent() else { return Err(EndOfFile) };

        match next.kind {
            RightBrace => break,
            NewLine => return Err(ExpectedRightBrace),
            _ => parameters.push(next)
        }
    };

    if macro_info.parameters.len() != parameters.len() {
        return Err(MacroParameterCount(macro_info.parameters.len(), parameters.len()))
    }

    let mut parameter_map: HashMap<&'a str, TokenKind> = HashMap::new();

    for (index, value) in parameters.into_iter().enumerate() {
        let name = macro_info.parameters[index];

        parameter_map.insert(name, value.kind);
    }

    let mut result = vec![];

    for kind in &macro_info.items {
        let mapped_kind = match kind {
            Parameter(name) => parameter_map.get(name)
                .ok_or_else(|| MacroUnknownParameter(name.to_string()))?,
            _ => kind
        };

        result.push(Token { start: element.start, kind: mapped_kind.clone() });
    }

    Ok(result)
}

pub fn preprocess(items: Vec<Token>) -> Result<Vec<Token>, PreprocessorError> {
    let mut iter = items.into_iter();
    let mut result: Vec<Token> = vec![];

    let mut cache = Cache::new();

    let watched_directives = HashSet::from(["eqv", "macro"]);

    while let Some(element) = iter.next() {
        let fail = |reason: PreprocessorReason| PreprocessorError {
            start: element.start, reason
        };

        match element.kind {
            Directive(directive) if watched_directives.contains(directive) => {
                match directive {
                    "eqv" => {
                        let (key, value) = consume_eqv(&mut iter)
                            .map_err(|err| fail(err))?;

                        cache.tokens.insert(key, value);
                    },
                    "macro" => {
                        let (key, value) = consume_macro(&mut iter)
                            .map_err(|err| fail(err))?;

                        cache.macros.insert(key, value);
                    },
                    _ => panic!()
                }
            }
            Symbol(name) => {
                let mut elements = handle_symbol(name, element, &mut iter, &cache)
                    .map_err(|err| fail(err))?;

                result.append(&mut elements)
            },
            Parameter(_) => return Err(PreprocessorError {
                start: element.start,
                reason: LoneParameter
            }),

            _ => result.push(element)
        }
    }

    Ok(result)
}
