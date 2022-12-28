use std::vec::IntoIter;
use crate::assembler::lexer::Item;
use crate::assembler::lexer::ItemKind::{Comment, NewLine, Comma};

pub trait LexerSeek<'a> {
    fn next_any(&mut self) -> Option<Item<'a>>;
    fn next_adjacent(&mut self) -> Option<Item<'a>>;
}

impl<'a> LexerSeek<'a> for IntoIter<Item<'a>> {
    fn next_any(&mut self) -> Option<Item<'a>> {
        while let Some(value) = self.next() {
            match value.kind {
                Comment(_) => { },
                NewLine => { },
                Comma => { }, // Completely ignored by MARS.
                _ => return Some(value)
            }
        }

        None
    }

    fn next_adjacent(&mut self) -> Option<Item<'a>> {
        while let Some(value) = self.next() {
            match value.kind {
                Comment(_) => { },
                Comma => { }, // Completely ignored by MARS.
                _ => return Some(value)
            }
        }

        None
    }
}