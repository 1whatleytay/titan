use crate::assembler::lexer::{Item, ItemKind};
use crate::assembler::lexer::LexerNextIterator;

fn preprocess(items: Vec<Item>) -> Vec<Item> {
    let mut iter = items.iter();
    let mut result = vec![];

    while let Some(element) = iter.next_any() {
        match element.kind {
            ItemKind::Comment(_) => {}
            ItemKind::Directive(_) => {}
            ItemKind::Parameter(_) => {}
            ItemKind::Register(_) => {}
            ItemKind::IntegerLiteral(_) => {}
            ItemKind::StringLiteral(_) => {}
            ItemKind::Symbol(_) => {}
            ItemKind::Comma => {}
            ItemKind::Colon => {}
            ItemKind::NewLine => {}
            ItemKind::LeftBrace => {}
            ItemKind::RightBrace => {}
        }
    }

    result
}