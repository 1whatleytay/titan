use crate::assembler::assembler_util::AssemblerReason::{MissingRegion, UnexpectedToken};
use crate::assembler::assembler_util::{pc_for_region, AssemblerError};
use crate::assembler::binary::Binary;
use crate::assembler::binary::BinarySection::Text;
use crate::assembler::binary_builder::BinaryBuilder;
use crate::assembler::cursor::{is_adjacent_kind, is_solid_kind, LexerCursor};
use crate::assembler::directive::do_directive;
use crate::assembler::emit::do_instruction;
use crate::assembler::instructions::instructions_map;
use crate::assembler::instructions::Instruction;
use crate::assembler::lexer::TokenKind::{Directive, IntegerLiteral, Minus, Plus, Symbol};
use crate::assembler::lexer::{Token, TokenKind};
use std::collections::HashMap;

enum SymbolType {
    Label,
    Instruction,
}

fn do_symbol(
    name: &str,
    start: usize,
    iter: &mut LexerCursor,
    builder: &mut BinaryBuilder,
    map: &HashMap<&str, &Instruction>,
) -> Result<SymbolType, AssemblerError> {
    // We need this region!

    let region = builder.region().ok_or(AssemblerError {
        start: Some(start),
        reason: MissingRegion,
    })?;

    match iter.seek_without(is_adjacent_kind) {
        Some(token) if token.kind == TokenKind::Colon => {
            iter.next(); // consume

            let pc = pc_for_region(&region.raw, Some(start))?;
            builder.labels.insert(name.to_string(), pc);

            Ok(SymbolType::Label)
        }
        _ => {
            do_instruction(name, start, iter, builder, map)?;

            Ok(SymbolType::Instruction)
        }
    }
}

pub fn assemble(items: &[Token], instructions: &[Instruction]) -> Result<Binary, AssemblerError> {
    let mut cursor = LexerCursor::new(items);

    let map = instructions_map(instructions);

    let mut builder = BinaryBuilder::new();
    builder.seek_mode(Text);

    let mut last_directive = Option::<(&str, usize)>::None;

    while let Some(token) = cursor.seek_without(is_solid_kind) {
        match &token.kind {
            Plus | Minus | IntegerLiteral(_) => {
                let Some((directive, start)) = last_directive else {
                    return Err(AssemblerError {
                        start: Some(token.start),
                        reason: UnexpectedToken(token.kind.strip())
                    })
                };

                do_directive(directive, start, &mut cursor, &mut builder)?
            }
            _ => {}
        }

        let Some(token) = cursor.next() else { continue };

        match &token.kind {
            Directive(directive) => {
                last_directive = Some((directive, token.start));

                do_directive(directive, token.start, &mut cursor, &mut builder)?
            }
            Symbol(name) => {
                let result = do_symbol(name.get(), token.start, &mut cursor, &mut builder, &map)?;

                if let SymbolType::Instruction = result {
                    last_directive = None;
                }
            }
            _ => {
                return Err(AssemblerError {
                    start: Some(token.start),
                    reason: UnexpectedToken(token.kind.strip()),
                })
            }
        }
    }

    builder.build()
}
