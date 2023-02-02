use std::collections::HashMap;
use crate::assembler::binary::Binary;
use crate::assembler::binary_builder::BinaryBuilder;
use crate::assembler::binary::BinarySection::Text;
use crate::assembler::directive::do_directive;
use crate::assembler::emit::do_instruction;
use crate::assembler::lexer::{Token, TokenKind};
use crate::assembler::lexer::TokenKind::{Symbol, Directive};
use crate::assembler::lexer_seek::{is_adjacent_kind, LexerSeek, LexerSeekPeekable};
use crate::assembler::instructions::Instruction;
use crate::assembler::instructions::instructions_map;
use crate::assembler::assembler_util::AssemblerReason::{UnexpectedToken, MissingRegion};
use crate::assembler::assembler_util::AssemblerError;

fn do_symbol<'a, T: LexerSeekPeekable<'a>>(
    name: &str, start: usize, iter: &mut T,
    builder: &mut BinaryBuilder, map: &HashMap<&str, &Instruction>
) -> Result<(), AssemblerError> {
    // We need this region!

    let region = builder.region()
        .ok_or(AssemblerError { start: Some(start), reason: MissingRegion })?;

    match iter.seek_without(is_adjacent_kind) {
        Some(token) if token.kind == TokenKind::Colon => {
            iter.next(); // consume

            let pc = region.raw.address + region.raw.data.len() as u32;
            builder.labels.insert(name.to_string(), pc);

            Ok(())
        },
        _ => do_instruction(name, start, iter, builder, map)
    }
}

pub fn assemble<'a>(
    items: Vec<Token<'a>>, instructions: &[Instruction]
) -> Result<Binary, AssemblerError> {
    let mut iter = items.into_iter().peekable();

    let map = instructions_map(instructions);

    let mut builder = BinaryBuilder::new();
    builder.seek_mode(Text);

    while let Some(token) = iter.next_any() {
        match token.kind {
            Directive(directive) =>
                do_directive(directive, token.start, &mut iter, &mut builder),
            Symbol(name) =>
                do_symbol(name.get(), token.start, &mut iter, &mut builder, &map),
            _ => return Err(AssemblerError {
                start: Some(token.start),
                reason: UnexpectedToken
            })
        }?
    }

    builder.build()
}
