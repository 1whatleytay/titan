use crate::assembler::assembler_util::AssemblerReason::{
    ConstantOutOfRange, EndOfFile, ExpectedConstant, MissingRegion, OverwriteEdge, UnknownDirective,
};
use crate::assembler::assembler_util::{default_start, get_constant, get_integer, get_integer_adjacent, get_string, pc_for_region, AssemblerError, get_label};
use crate::assembler::binary::AddressLabel::Label;
use crate::assembler::binary::BinarySection::{Data, KernelData, KernelText, Text};
use crate::assembler::binary::{BinarySection, NamedLabel};
use crate::assembler::binary_builder::{BinaryBuilder, BinaryBuilderLabel, BinaryBuilderRegion, InstructionLabel, InstructionLabelKind};
use crate::assembler::cursor::{is_adjacent_kind, is_solid_kind, LexerCursor};
use crate::assembler::lexer::TokenKind::{Colon, NewLine};
use crate::assembler::lexer::{Location, Token, TokenKind};
use byteorder::{ByteOrder, LittleEndian};
use TokenKind::LeftBrace;

const MISSING_REGION: AssemblerError = AssemblerError {
    location: None,
    reason: MissingRegion,
};

fn do_seek_directive(
    mode: BinarySection,
    iter: &mut LexerCursor,
    builder: &mut BinaryBuilder,
) -> Result<(), AssemblerError> {
    let address = get_integer_adjacent(iter);

    match address {
        Some(address) => builder.seek_mode_address(mode, address as u32),
        None => builder.seek_mode(mode),
    };

    Ok(())
}

fn do_globl_directive(iter: &mut LexerCursor, _: &mut BinaryBuilder) -> Result<(), AssemblerError> {
    iter.collect_without(|kind| kind == &NewLine);

    // Ignore, dummy directive since no multi-file support at the moment.

    Ok(())
}

fn do_ascii_directive(
    iter: &mut LexerCursor,
    builder: &mut BinaryBuilder,
) -> Result<(), AssemblerError> {
    let mut bytes = get_string(iter)?.into_bytes();
    let region = builder.region().ok_or(MISSING_REGION)?;

    region.raw.data.append(&mut bytes);

    Ok(())
}

fn do_asciiz_directive(
    iter: &mut LexerCursor,
    builder: &mut BinaryBuilder,
) -> Result<(), AssemblerError> {
    let mut bytes = get_string(iter)?.into_bytes();
    bytes.push(0);

    let region = builder.region().ok_or(MISSING_REGION)?;

    region.raw.data.append(&mut bytes);

    Ok(())
}

const MAX_ZERO: usize = 0x100000;

fn align_with_zeros(region: &mut BinaryBuilderRegion, align: u32) -> Result<(), AssemblerError> {
    let pc = pc_for_region(&region.raw, None)?;

    let (select, remainder) = (pc / align, pc % align);
    let correction = if remainder > 0 { 1 } else { 0 };

    let target = (select + correction) * align;
    let align_count = target as usize - pc as usize;
    
    let mut align_bytes = vec![0; align_count];

    region.raw.data.append(&mut align_bytes);
    
    Ok(())
}

fn do_align_directive(
    iter: &mut LexerCursor,
    builder: &mut BinaryBuilder,
) -> Result<(), AssemblerError> {
    let shift = get_constant(iter)?;

    if !(0..=16).contains(&shift) {
        return Err(AssemblerError {
            location: None,
            reason: ConstantOutOfRange(0, 16),
        });
    }

    let align = 1 << shift;

    let region = builder.region().ok_or(MISSING_REGION)?;
    let pc = pc_for_region(&region.raw, None)?;

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

fn do_space_directive(
    iter: &mut LexerCursor,
    builder: &mut BinaryBuilder,
) -> Result<(), AssemblerError> {
    let region = builder.region().ok_or(MISSING_REGION)?;
    let pc = pc_for_region(&region.raw, None)?;

    let byte_count = get_constant(iter)? as usize;

    if byte_count > MAX_ZERO {
        let Some(target) = pc.checked_add(byte_count as u32) else {
            return Err(AssemblerError {
                location: None,
                reason: OverwriteEdge(pc, Some(byte_count as u64))
            })
        };

        builder.seek_mode_address(builder.state.mode, target)
    } else {
        let mut space_bytes = vec![0; byte_count];

        region.raw.data.append(&mut space_bytes);
    }

    Ok(())
}

const REPEAT_LIMIT: u64 = 0x100000;

struct ConstantInfo {
    value: u64,
    count: u64,
}

// Specifically for .word
enum ConstantOrLabel {
    Constant(ConstantInfo),
    Label(NamedLabel),
}

fn grab_value(
    value: &Token,
    iter: &mut LexerCursor,
) -> Result<Option<ConstantInfo>, AssemblerError> {
    let Some(value) = get_integer(value, iter, true) else {
        return Ok(None)
    };

    let next_up = iter.seek_without(is_adjacent_kind);

    let count = if next_up.map(|x| x.kind == Colon).unwrap_or(false) {
        iter.next();

        let Some(token) = iter.next_adjacent() else {
            return Err(AssemblerError { location: None, reason: EndOfFile });
        };

        let Some(value) = get_integer(token, iter, false) else {
            return Err(AssemblerError {
                location: Some(token.location),
                reason: ExpectedConstant(token.kind.strip())
            })
        };

        if value > REPEAT_LIMIT {
            return Err(AssemblerError {
                location: Some(token.location),
                reason: ConstantOutOfRange(0, REPEAT_LIMIT as i64),
            });
        }

        value
    } else {
        1u64
    };

    Ok(Some(ConstantInfo { value, count }))
}

fn get_constant_or_labels(iter: &mut LexerCursor) -> Result<Vec<ConstantOrLabel>, AssemblerError> {
    let mut result: Vec<ConstantOrLabel> = vec![];

    while let Some(value) = iter.seek_without(is_solid_kind) {
        let start = iter.get_position();

        let item = if let TokenKind::Symbol(name) = &value.kind {
            // This is workaroundy, but a symbol can also be a label

            iter.next();

            let (_, token) = iter.peek_adjacent();

            let do_skip = match token.map(|x| &x.kind) {
                Some(Colon) => true,     // label
                Some(LeftBrace) => true, // Macro
                _ => false,
            };

            // This is obviously a sign that the directive section has to be reworked.
            if do_skip {
                iter.set_position(start);

                break;
            }

            let address = NamedLabel {
                name: name.get().to_string(),
                location: value.location,
                offset: 0,
            };

            ConstantOrLabel::Label(address)
        } else {
            let Some(constant) = grab_value(value, iter)? else { break };

            ConstantOrLabel::Constant(constant)
        };

        result.push(item);
    }

    Ok(result)
}

fn get_constants(iter: &mut LexerCursor) -> Result<Vec<ConstantInfo>, AssemblerError> {
    let mut result = vec![];

    while let Some(value) = iter.seek_without(is_solid_kind) {
        let Some(constant) = grab_value(value, iter)? else { break };

        result.push(constant)
    }

    Ok(result)
}

fn do_byte_directive(
    iter: &mut LexerCursor,
    builder: &mut BinaryBuilder,
) -> Result<(), AssemblerError> {
    let values = get_constants(iter)?;

    let region = builder.region().ok_or(MISSING_REGION)?;

    for value in values {
        if value.count > REPEAT_LIMIT {
            continue;
        }

        if value.count == 1 {
            region.raw.data.push(value.value as u8)
        } else {
            region
                .raw
                .data
                .append(&mut vec![value.value as u8; value.count as usize])
        }
    }

    Ok(())
}

fn do_half_directive(
    iter: &mut LexerCursor,
    builder: &mut BinaryBuilder,
) -> Result<(), AssemblerError> {
    let values = get_constants(iter)?;

    let region = builder.region().ok_or(MISSING_REGION)?;

    align_with_zeros(region, 2)?;
    
    for value in values {
        if value.count > REPEAT_LIMIT {
            continue;
        }

        let mut array = [0u8; 2];
        LittleEndian::write_u16(&mut array, value.value as u16);

        region.raw.data.reserve(2 * value.count as usize);

        for _ in 0..value.count {
            region.raw.data.extend_from_slice(&array);
        }
    }

    Ok(())
}

fn do_word_directive(
    iter: &mut LexerCursor,
    builder: &mut BinaryBuilder,
) -> Result<(), AssemblerError> {
    // Being extra cautious for when these features are enabled.
    // Don't want it to consume "symbols" of instructions.
    let values = if builder.state.mode.is_data() {
        get_constant_or_labels(iter)?
    } else {
        get_constants(iter)?
            .into_iter()
            .map(ConstantOrLabel::Constant)
            .collect()
    };

    let region = builder.region().ok_or(MISSING_REGION)?;

    // First, align to 4 bytes

    align_with_zeros(region, 4)?;

    for value in values {
        match value {
            ConstantOrLabel::Label(label) => {
                let offset = region.raw.data.len();

                region.raw.data.extend_from_slice(&[0u8; 4]);
                region.labels.push(BinaryBuilderLabel {
                    offset,
                    location: label.location,
                    label: InstructionLabel {
                        kind: InstructionLabelKind::Full,
                        label: Label(label),
                    },
                })
            }
            ConstantOrLabel::Constant(value) => {
                if value.count > REPEAT_LIMIT {
                    continue;
                }

                let mut array = [0u8; 4];
                LittleEndian::write_u32(&mut array, value.value as u32);

                region.raw.data.reserve(4 * value.count as usize);

                for _ in 0..value.count {
                    region.raw.data.extend_from_slice(&array);
                }
            }
        }
    }

    Ok(())
}

// Don't want to deal with this until coprocessor
fn do_float_directive(_: &mut LexerCursor, _: &mut BinaryBuilder) -> Result<(), AssemblerError> {
    Err(AssemblerError {
        location: None,
        reason: UnknownDirective("float".to_string()),
    })
}

fn do_double_directive(_: &mut LexerCursor, _: &mut BinaryBuilder) -> Result<(), AssemblerError> {
    Err(AssemblerError {
        location: None,
        reason: UnknownDirective("double".to_string()),
    })
}

fn do_entry_directive(iter: &mut LexerCursor, builder: &mut BinaryBuilder) -> Result<(), AssemblerError> {
    let label = get_label(iter)?;

    builder.entry = Some(label);

    Ok(())
}

fn do_extern_directive(
    iter: &mut LexerCursor,
    _: &mut BinaryBuilder,
) -> Result<(), AssemblerError> {
    get_string(iter)?;
    get_constant(iter)?;

    Ok(())
}

pub fn do_directive(
    directive: &str,
    location: Location,
    iter: &mut LexerCursor,
    builder: &mut BinaryBuilder,
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
        "entry" => do_entry_directive(iter, builder),

        "text" => do_seek_directive(Text, iter, builder),
        "data" => do_seek_directive(Data, iter, builder),
        "ktext" => do_seek_directive(KernelText, iter, builder),
        "kdata" => do_seek_directive(KernelData, iter, builder),

        "extern" => do_extern_directive(iter, builder),
        _ => Err(AssemblerError {
            location: Some(location),
            reason: UnknownDirective(directive.to_string()),
        }),
    }
    .map_err(default_start(location))
}
