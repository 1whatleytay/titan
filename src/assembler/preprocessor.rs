use std::collections::{HashMap, HashSet};
use crate::assembler::lexer::{Item, ItemKind};
use crate::assembler::lexer::ItemKind::{LeftBrace, Parameter, RightBrace, NewLine, Symbol};
use crate::assembler::lexer_seek::LexerSeek;
use crate::assembler::preprocessor::PreprocessorReason::{
    EndOfFile,
    ExpectedLeftBrace,
    ExpectedParameter,
    ExpectedRightBrace,
    ExpectedSymbol,
    LoneParameter
};

enum PreprocessorReason {
    LoneParameter,
    EndOfFile,
    ExpectedSymbol,
    ExpectedParameter,
    ExpectedLeftBrace,
    ExpectedRightBrace,
}

struct PreprocessorError<'a> {
    start: &'a str,
    reason: PreprocessorReason
}

struct Macro<'a> {
    parameters: Vec<&'a str>,
    items: Vec<ItemKind<'a>>
}

fn preprocess(items: Vec<Item>) -> Result<Vec<Item>, PreprocessorError> {
    let mut iter = items.into_iter();
    let mut result: Vec<Item> = vec![];

    let mut eqv_cache: HashMap<&str, ItemKind> = HashMap::new();
    let mut macro_cache: HashMap<&str, Macro> = HashMap::new();

    let watched_directives = HashSet::from(["eqv", "macro"]);

    while let Some(element) = iter.next() {
        let fail = |reason: PreprocessorReason| Err(PreprocessorError {
            start: element.start, reason
        });

        let ended = || fail(EndOfFile);

        match element.kind {
            ItemKind::Directive(directive) if watched_directives.contains(directive) => {
                match directive {
                    "eqv" => {
                        let Some(symbol) = iter.next_adjacent() else { return ended() };
                        let Some(value) = iter.next_adjacent() else { return ended() };

                        let Symbol(key) = symbol.kind else { return fail(ExpectedSymbol) };

                        eqv_cache.insert(key, value.kind);
                    },
                    "macro" => {
                        let mut result = Macro { parameters: vec![], items: vec![] };

                        let Some(symbol) = iter.next_adjacent() else { return ended() };
                        let Symbol(name) = symbol.kind else { return fail(ExpectedSymbol) };

                        let Some(left_brace) = iter.next_adjacent() else { return ended() };
                        if left_brace.kind != LeftBrace { return fail(ExpectedLeftBrace) };

                        loop {
                            let Some(next) = iter.next_adjacent() else { return ended() };

                            match next.kind {
                                RightBrace => break,
                                Parameter(name) => { result.parameters.push(name) },
                                NewLine => return fail(ExpectedRightBrace),
                                _ => return fail(ExpectedParameter)
                            }
                        }

                        loop {
                            let Some(next) = iter.next() else { return ended() };

                            match next.kind {
                                RightBrace => break,
                                Parameter(name) => { result.parameters.push(name) },
                                NewLine => return fail(ExpectedRightBrace),
                                _ => return fail(ExpectedParameter)
                            }
                        }

                        macro_cache.insert(name, result);
                    },
                    _ => { }
                }
            }
            Symbol(name) => {
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