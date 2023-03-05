use byteorder::{ByteOrder, LittleEndian};
use crate::assembler::binary_builder::{BinaryBuilder};
use crate::assembler::binary::BinarySection;
use crate::assembler::binary::BinarySection::{Data, KernelData, KernelText, Text};
use crate::assembler::lexer::TokenKind::{Colon, NewLine};
use crate::assembler::cursor::{is_adjacent_kind, is_solid_kind, LexerCursor};
use crate::assembler::assembler_util::{AssemblerError, default_start, get_constant, get_integer, get_integer_adjacent, get_string};
use crate::assembler::assembler_util::AssemblerReason::{ConstantOutOfRange, EndOfFile, ExpectedConstant, MissingRegion, OverwriteEdge, UnknownDirective};

const MISSING_REGION: AssemblerError = AssemblerError { start: None, reason: MissingRegion };

fn do_seek_directive<'a>(
    mode: BinarySection, iter: &mut LexerCursor, builder: &mut BinaryBuilder
) -> Result<(), AssemblerError> {
    let address = get_integer_adjacent(iter);

    match address {
        Some(address) => builder.seek_mode_address(mode, address as u32),
        None => builder.seek_mode(mode)
    };

    Ok(())
}

fn do_globl_directive<'a>(
    iter: &mut LexerCursor, _: &mut BinaryBuilder
) -> Result<(), AssemblerError> {
    iter.collect_without(|kind| kind == &NewLine);

    // Ignore, dummy directive since no multi-file support at the moment.

    Ok(())
}

fn do_ascii_directive<'a>(
    iter: &mut LexerCursor, builder: &mut BinaryBuilder
) -> Result<(), AssemblerError> {
    let mut bytes = get_string(iter)?.into_bytes();
    let region = builder.region().ok_or(MISSING_REGION)?;

    region.raw.data.append(&mut bytes);

    Ok(())
}

fn do_asciiz_directive<'a>(
    iter: &mut LexerCursor, builder: &mut BinaryBuilder
) -> Result<(), AssemblerError> {
    let mut bytes = get_string(iter)?.into_bytes();
    bytes.push(0);

    let region = builder.region().ok_or(MISSING_REGION)?;

    region.raw.data.append(&mut bytes);

    Ok(())
}

const MAX_ZERO: usize = 0x100000;

fn do_align_directive<'a>(
    iter: &mut LexerCursor, builder: &mut BinaryBuilder
) -> Result<(), AssemblerError> {
    let shift = get_constant(iter)?;

    if !(0 ..= 16).contains(&shift) {
        return Err(AssemblerError { start: None, reason: ConstantOutOfRange(0, 16) })
    }

    let align = 1 << shift;

    let region = builder.region().ok_or(MISSING_REGION)?;
    let pc = region.raw.address + region.raw.data.len() as u32;

    let (select, remainder) = (pc / align, pc % align);
    let correction = if remainder > 0 { 1 } else { 0 };

    let target = (select + correction) * align;
    let align_count = target as usize - pc as usize;

    if align_count > MAX_ZERO {
        builder.seek_mode_address(builder.state.mode, target)
    } else {
        let mut align_bytes = vec![0; align_count];

        region.raw.data.append(&mut align_bytes);
    }

    Ok(())
}

fn do_space_directive<'a>(
    iter: &mut LexerCursor, builder: &mut BinaryBuilder
) -> Result<(), AssemblerError> {
    let region = builder.region().ok_or(MISSING_REGION)?;
    let pc = region.raw.address + region.raw.data.len() as u32;

    let byte_count = get_constant(iter)? as usize;

    if byte_count > MAX_ZERO {
        let Some(target) = pc.checked_add(byte_count as u32) else {
            return Err(AssemblerError { start: None, reason: OverwriteEdge(pc, byte_count as u64) })
        };

        builder.seek_mode_address(builder.state.mode, target)
    } else {
        let mut space_bytes = vec![0; byte_count];

        region.raw.data.append(&mut space_bytes);
    }

    Ok(())
}

const REPEAT_LIMIT: u64 = 0x100000;

fn get_constants<'a>(iter: &mut LexerCursor) -> Result<Vec<(u64, u64)>, AssemblerError> {
    let mut result = vec![];

    while let Some(value) = iter.seek_without(is_solid_kind) {
        let Some(value) = get_integer(value, iter, true) else {
            break
        };

        let next_up = iter.seek_without(is_adjacent_kind);

        let count = if next_up.map(|x| x.kind == Colon).unwrap_or(false) {
            iter.next();

            let Some(token) = iter.next_adjacent() else {
                return Err(AssemblerError { start: None, reason: EndOfFile });
            };

            let Some(value) = get_integer(token, iter, false) else {
                return Err(AssemblerError {
                    start: Some(token.start),
                    reason: ExpectedConstant(token.kind.strip())
                })
            };

            if value > REPEAT_LIMIT {
                return Err(AssemblerError {
                    start: Some(token.start),
                    reason: ConstantOutOfRange(0, REPEAT_LIMIT)
                })
            }

            value as u64
        } else {
            1u64
        };

        result.push((value, count))
    }

    Ok(result)
}

fn do_byte_directive<'a>(
    iter: &mut LexerCursor, builder: &mut BinaryBuilder
) -> Result<(), AssemblerError> {
    let values = get_constants(iter)?;

    let region = builder.region().ok_or(MISSING_REGION)?;

    for (value, count) in values {
        if count > REPEAT_LIMIT {
            continue
        }

        if count == 1 {
            region.raw.data.push(value as u8)
        } else {
            region.raw.data.append(&mut vec![value as u8; count as usize])
        }
    }

    Ok(())
}

fn do_half_directive<'a>(
    iter: &mut LexerCursor, builder: &mut BinaryBuilder
) -> Result<(), AssemblerError> {
    let values = get_constants(iter)?;

    let region = builder.region().ok_or(MISSING_REGION)?;

    for (value, count) in values {
        if count > REPEAT_LIMIT {
            continue
        }

        let mut array = [0u8; 2];
        LittleEndian::write_u16(&mut array, value as u16);

        region.raw.data.reserve(2 * count as usize);

        for _ in 0 .. count {
            region.raw.data.extend_from_slice(&array);
        }
    }

    Ok(())
}

fn do_word_directive<'a>(
    iter: &mut LexerCursor, builder: &mut BinaryBuilder
) -> Result<(), AssemblerError> {
    let values = get_constants(iter)?;

    let region = builder.region().ok_or(MISSING_REGION)?;

    for (value, count) in values {
        if count > REPEAT_LIMIT {
            continue
        }

        let mut array = [0u8; 4];
        LittleEndian::write_u32(&mut array, value as u32);

        region.raw.data.reserve(4 * count as usize);

        for _ in 0 .. count {
            region.raw.data.extend_from_slice(&array);
        }
    }

    Ok(())
}

// Don't want to deal with this until coprocessor
fn do_float_directive<'a>(
    _: &mut LexerCursor, _: &mut BinaryBuilder
) -> Result<(), AssemblerError> {
    Err(AssemblerError { start: None, reason: UnknownDirective("float".to_string()) })
}

fn do_double_directive<'a>(
    _: &mut LexerCursor, _: &mut BinaryBuilder
) -> Result<(), AssemblerError> {
    Err(AssemblerError { start: None, reason: UnknownDirective("double".to_string()) })
}

fn do_extern_directive<'a>(
    iter: &mut LexerCursor, _: &mut BinaryBuilder
) -> Result<(), AssemblerError> {
    get_string(iter)?;
    get_constant(iter)?;

    Ok(())
}

pub fn do_directive<'a>(
    directive: &'a str, start: usize, iter: &mut LexerCursor, builder: &mut BinaryBuilder
) -> Result<(), AssemblerError> {
    let lowercase = directive.to_lowercase();

    match &lowercase as &str {
        "globl" | "global" => do_globl_directive(iter, builder),

        "ascii" => do_ascii_directive(iter, builder),
        "asciiz" => do_asciiz_directive(iter, builder),
        "align" => do_align_directive(iter, builder),
        "space" => do_space_directive(iter, builder),
        "byte" => do_byte_directive(iter, builder),
        "half" => do_half_directive(iter, builder),
        "word" => do_word_directive(iter, builder),
        "float" => do_float_directive(iter, builder),
        "double" => do_double_directive(iter, builder),

        "text" => do_seek_directive(Text, iter, builder),
        "data" => do_seek_directive(Data, iter, builder),
        "ktext" => do_seek_directive(KernelText, iter, builder),
        "kdata" => do_seek_directive(KernelData, iter, builder),

        "extern" => do_extern_directive(iter, builder),
        _ => Err(AssemblerError {
            start: Some(start),
            reason: UnknownDirective(directive.to_string())
        })
    }
        .map_err(default_start(start))
}