use std::collections::HashMap;
use crate::assembler::binary::Binary;
use crate::assembler::binary_builder::BinaryBuilder;
use crate::assembler::binary::BinarySection::Text;
use crate::assembler::directive::do_directive;
use crate::assembler::emit::do_instruction;
use crate::assembler::lexer::{Token, TokenKind};
use crate::assembler::lexer::TokenKind::{Symbol, Directive, IntegerLiteral};
use crate::assembler::lexer_seek::{is_adjacent_kind, LexerSeekPeekable};
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

    let mut last_directive = Option::<(&str, usize)>::None;

    while let Some(token) = iter.seek_without(is_adjacent_kind) {
        match &token.kind {
            IntegerLiteral(_) => {
                let Some((directive, start)) = last_directive else {
                    return Err(AssemblerError {
                        start: Some(token.start),
                        reason: UnexpectedToken(token.kind.strip())
                    })
                };

                do_directive(directive, start, &mut iter, &mut builder)?
            }
            _ => { }
        }

        let Some(token) = iter.next() else { continue };

        match &token.kind {
            Directive(directive) => {
                last_directive = Some((directive, token.start));

                do_directive(directive, token.start, &mut iter, &mut builder)
            }
            Symbol(name) => {
                last_directive = None;

                do_symbol(name.get(), token.start, &mut iter, &mut builder, &map)
            }
            _ => return Err(AssemblerError {
                start: Some(token.start),
                reason: UnexpectedToken(token.kind.strip())
            })
        }?
    }

    builder.build()
}
