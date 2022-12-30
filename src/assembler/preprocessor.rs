use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::assembler::lexer::{SymbolName, Token, TokenKind};
use crate::assembler::lexer::SymbolName::Owned;
use crate::assembler::lexer::TokenKind::{
    Colon, LeftBrace, Parameter, RightBrace, NewLine, Symbol, Directive
};
use crate::assembler::lexer_seek::{is_adjacent_kind, LexerSeek};
use crate::assembler::preprocessor::PreprocessorReason::{
    LoneParameter,
    EndOfFile,
    ExpectedSymbol,
    ExpectedParameter,
    ExpectedLeftBrace,
    ExpectedRightBrace,
    MacroUnknown,
    MacroParameterCount,
    MacroUnknownParameter,
};

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
    labels: HashSet<String>,
    items: Vec<TokenKind<'a>>
}

impl<'a> Macro<'a> {
    fn new() -> Macro<'a> {
        Macro { parameters: vec![], labels: HashSet::new(), items: vec![] }
    }
}

struct Cache<'a> {
    tokens: HashMap<String, TokenKind<'a>>,
    macros: HashMap<String, Macro<'a>>
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

fn consume_eqv<'a, 'b, T>(iter: &'b mut T) -> Result<(String, TokenKind<'a>), PreprocessorReason>
    where T: LexerSeek<'a> {
    let Some(symbol) = iter.next_adjacent() else { return Err(EndOfFile) };
    let Some(value) = iter.next_adjacent() else { return Err(EndOfFile) };

    let Symbol(key) = symbol.kind else { return Err(ExpectedSymbol) };

    Ok((key.get().to_string(), value.kind))
}

fn consume_macro<'a, 'b, T>(iter: &'b mut T) -> Result<(String, Macro<'a>), PreprocessorReason>
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

    fn collect_tokens<'a, T: LexerSeek<'a>>(iter: &mut T) -> Vec<TokenKind<'a>> {
        iter.collect_until(is_adjacent_kind)
            .into_iter().map(|item| item.kind).collect()
    }

    let mut stop = false;
    while !stop {
        let mut items: Vec<TokenKind> = collect_tokens(iter);

        let last = items.last().ok_or_else(|| EndOfFile)?;

        stop = match last {
            Symbol(name) => {
                let mut post_items = collect_tokens(iter);

                if post_items.last().map(|last| last == &Colon).unwrap_or(false) {
                    result.labels.insert(name.get().to_string());
                }

                items.append(&mut post_items);

                false
            }
            Directive(directive) if *directive == "end_macro" => true,
            _ => false
        };

        result.items.append(&mut items);
    }

    result.items.pop();

    Ok((name.get().to_string(), result))
}

fn handle_symbol<'a, 'b, 'c, T>(
    name: SymbolName<'a>, start: &'a str, iter: &'b mut T, cache: &'c Cache<'a>, seed: &mut usize
) -> Result<Vec<Token<'a>>, PreprocessorReason> where T: LexerSeek<'a> {
    if let Some(token) = cache.tokens.get(name.get()) {
        return Ok(vec![Token { start, kind: token.clone() }])
    }

    let elements = iter.collect_until(|kind| is_adjacent_kind(kind));

    fn concat<'a>(element: Token<'a>, mut elements: Vec<Token<'a>>) -> Vec<Token<'a>> {
        let mut first = vec![element];
        first.append(&mut elements);

        first
    }

    let Some(last) = elements.last() else {
        return Ok(concat(Token { start, kind: Symbol(name) }, elements))
    };

    match last.kind {
        LeftBrace => { },
        _ => return Ok(concat(Token { start, kind: Symbol(name) }, elements))
    }

    // Treat as a macro!
    let Some(macro_info) = cache.macros.get(name.get()) else {
        return Err(MacroUnknown(name.get().to_string()))
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

    let label_names: HashMap<&str, String> = macro_info.labels.iter()
        .map(|name| (&name[..], format!("_M{}_{}", name, { *seed += 1; *seed })))
        .collect();

    let mut parameter_map: HashMap<&'a str, TokenKind> = HashMap::new();

    for (index, value) in parameters.into_iter().enumerate() {
        let name = macro_info.parameters[index];

        parameter_map.insert(name, value.kind);
    }

    let mut result = vec![];

    for kind in &macro_info.items {
        let mapped_kind = match kind {
            Parameter(name) => parameter_map.get(name).cloned()
                .ok_or_else(|| MacroUnknownParameter(name.to_string()))?,
            Symbol(name) => {
                if let Some(new_name) = label_names.get(name.get()) {
                    Symbol(Owned(new_name.clone()))
                } else {
                    kind.clone()
                }
            }
            _ => kind.clone()
        };

        result.push(Token { start, kind: mapped_kind.clone() });
    }

    Ok(result)
}

pub fn preprocess(items: Vec<Token>) -> Result<Vec<Token>, PreprocessorError> {
    let mut iter = items.into_iter().peekable();
    let mut result: Vec<Token> = vec![];

    let mut cache = Cache::new();

    let watched_directives = HashSet::from(["eqv", "macro"]);

    let mut seed = 0;
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
                let mut elements = handle_symbol(
                    name, element.start, &mut iter, &cache, &mut seed
                ).map_err(|err| fail(err))?;

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

#[cfg(test)]
mod tests {
    use std::fs;
    use crate::assembler::source::assemble_from;

    #[test]
    fn my_test() {
        let path = "/Users/desgroup/Projects/breakout/breakout.asm";
        let text = fs::read_to_string(path).unwrap();

        let binary = assemble_from(&text).unwrap();

        println!("{:?}", binary);
    }
}
