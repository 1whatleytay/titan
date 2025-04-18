use crate::assembler::assembler_util::AssemblerReason::{
    DuplicateLabel, MissingRegion, UnexpectedToken,
};
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
use crate::assembler::lexer::{Location, Token, TokenKind};
use std::collections::HashMap;

enum SymbolType {
    Label,
    Instruction,
}

fn do_symbol(
    name: &str,
    location: Location,
    iter: &mut LexerCursor,
    builder: &mut BinaryBuilder,
    map: &HashMap<&str, &Instruction>,
) -> Result<SymbolType, AssemblerError> {
    // We need this region!

    let region = builder.region().ok_or(AssemblerError {
        location: Some(location),
        reason: MissingRegion,
    })?;

    match iter.seek_without(is_adjacent_kind) {
        Some(token) if token.kind == TokenKind::Colon => {
            iter.next(); // consume

            let pc = pc_for_region(&region.raw, Some(location))?;

            // If we already have this label, we want to panic!
            if builder.labels.contains_key(name) {
                return Err(AssemblerError {
                    location: Some(location),
                    reason: DuplicateLabel(name.to_string()),
                });
            }

            builder.labels.insert(name.to_string(), pc);

            Ok(SymbolType::Label)
        }
        _ => {
            do_instruction(name, location, iter, builder, map)?;

            Ok(SymbolType::Instruction)
        }
    }
}

pub fn assemble(items: &[Token], instructions: &[Instruction]) -> Result<Binary, AssemblerError> {
    let mut cursor = LexerCursor::new(items);

    let map = instructions_map(instructions);

    let mut builder = BinaryBuilder::new();
    builder.seek_mode(Text);

    let mut last_directive = Option::<(&str, Location)>::None;

    while let Some(token) = cursor.seek_without(is_solid_kind) {
        match &token.kind {
            Plus | Minus | IntegerLiteral(_) => {
                let Some((directive, start)) = last_directive else {
                    return Err(AssemblerError {
                        location: Some(token.location),
                        reason: UnexpectedToken(token.kind.strip()),
                    });
                };

                do_directive(directive, start, &mut cursor, &mut builder)?
            }
            _ => {}
        }

        let Some(token) = cursor.next() else { continue };

        match &token.kind {
            Directive(directive) => {
                last_directive = Some((directive, token.location));

                do_directive(directive, token.location, &mut cursor, &mut builder)?
            }
            Symbol(name) => {
                let result =
                    do_symbol(name.get(), token.location, &mut cursor, &mut builder, &map)?;

                if let SymbolType::Instruction = result {
                    last_directive = None;
                }
            }
            _ => {
                return Err(AssemblerError {
                    location: Some(token.location),
                    reason: UnexpectedToken(token.kind.strip()),
                })
            }
        }
    }

    builder.build()
}
