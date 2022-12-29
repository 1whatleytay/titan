use std::vec::IntoIter;
use crate::assembler::lexer::{Token, TokenKind};
use crate::assembler::lexer::TokenKind::{Comment, NewLine, Comma};

pub fn is_solid_kind(kind: &TokenKind) -> bool {
    match kind {
        Comment(_) => false,
        NewLine => false,
        Comma => false, // Completely ignored by MARS.
        _ => true
    }
}

pub fn is_adjacent_kind(kind: &TokenKind) -> bool {
    match kind {
        Comment(_) => false,
        Comma => false, // Completely ignored by MARS.
        _ => true
    }
}

pub trait LexerSeek<'a> {
    fn collect_until<F>(&mut self, f: F) -> Vec<Token<'a>>
        where for<'b> F: FnMut(&'b TokenKind<'a>) -> bool;

    fn seek_until<F>(&mut self, f: F) -> Option<Token<'a>>
        where for<'b> F: FnMut(&'b TokenKind<'a>) -> bool {
        self.collect_until(f).into_iter().last()
    }

    fn next_any(&mut self) -> Option<Token<'a>> {
        self.seek_until(is_solid_kind)
    }

    fn next_adjacent(&mut self) -> Option<Token<'a>> {
        self.seek_until(is_adjacent_kind)
    }
}

impl<'a> LexerSeek<'a> for IntoIter<Token<'a>> {
    fn collect_until<F>(&mut self, mut f: F) -> Vec<Token<'a>>
        where for<'b> F: FnMut(&'b TokenKind<'a>) -> bool {
        let mut result = vec![];

        while let Some(value) = self.next() {
            let do_break = f(&value.kind);

            result.push(value);

            if do_break {
                break
            }
        }

        result
    }

    fn seek_until<F>(&mut self, mut f: F) -> Option<Token<'a>>
        where for<'b> F: FnMut(&'b TokenKind<'a>) -> bool {
        while let Some(value) = self.next() {
            if f(&value.kind) {
                return Some(value)
            }
        }

        None
    }
}
