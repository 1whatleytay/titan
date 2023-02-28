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

pub struct LexerCursor<'a, 'b> {
    index: usize,
    tokens: &'b [Token<'a>]
}

impl<'a, 'b> LexerCursor<'a, 'b> {
    pub fn new(tokens: &'b [Token<'a>]) -> LexerCursor<'a, 'b> {
        LexerCursor { index: 0, tokens }
    }

    pub fn get_position(&self) -> usize {
        self.index
    }

    pub fn set_position(&mut self, index: usize) {
        self.index = index
    }

    pub fn peek(&self) -> Option<&'b Token<'a>> {
        self.tokens.get(self.index)
    }

    pub fn next(&mut self) -> Option<&'b Token<'a>> {
        let value = self.peek();

        self.index += 1;

        value
    }

    pub fn collect_until<F>(&mut self, mut f: F) -> Vec<&'b Token<'a>>
        where F: FnMut(&'b TokenKind<'a>) -> bool {
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

    pub fn seek_until<F>(&mut self, mut f: F) -> Option<&'b Token<'a>>
        where F: FnMut(&'b TokenKind<'a>) -> bool {
        while let Some(value) = self.next() {
            if f(&value.kind) {
                return Some(value)
            }
        }

        None
    }

    pub fn next_adjacent(&mut self) -> Option<&'b Token<'a>> {
        self.seek_until(is_adjacent_kind)
    }

    pub fn collect_without<F>(&mut self, mut f: F) -> Vec<&'b Token<'a>>
        where F: FnMut(&'b TokenKind<'a>) -> bool {
        let mut result = vec![];

        while let Some(value) = self.peek() {
            if !f(&value.kind) {
                self.index += 1;

                result.push(value)
            } else {
                break
            }
        }

        result
    }

    pub fn seek_without<F>(&mut self, mut f: F) -> Option<&'b Token<'a>>
        where F: FnMut(&'b TokenKind<'a>) -> bool {
        while let Some(value) = self.peek() {
            if !f(&value.kind) {
                self.index += 1
            } else {
                break
            }
        }

        self.peek()
    }

    pub fn peek_adjacent(&mut self) -> (usize, Option<&'b Token<'a>>) {
        let position = self.get_position();

        let result = self.seek_without(is_adjacent_kind);
        let end = self.get_position();

        self.set_position(position);

        (end, result)
    }

    pub fn consume_until(&mut self, index: usize) -> Vec<&'b Token<'a>> {
        if index < self.index {
            vec![]
        } else {
            let result = self.tokens[self.index .. index].into_iter().collect();

            self.index = index;

            result
        }
    }
}
