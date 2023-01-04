use byteorder::{ByteOrder, LittleEndian};
use crate::assembler::binary_builder::{BinaryBuilder};
use crate::assembler::binary::BinarySection;
use crate::assembler::binary::BinarySection::{Data, KernelData, KernelText, Text};
use crate::assembler::lexer::TokenKind::IntegerLiteral;
use crate::assembler::lexer_seek::{is_solid_kind, LexerSeekPeekable};
use crate::assembler::assembler_util::{AssemblerReason, get_constant, get_optional_constant, get_string};
use crate::assembler::assembler_util::AssemblerReason::{MissingRegion, UnknownDirective};

fn do_seek_directive<'a, T: LexerSeekPeekable<'a>>(
    mode: BinarySection, iter: &mut T, builder: &mut BinaryBuilder
) -> Result<(), AssemblerReason> {
    let address = get_optional_constant(iter);

    match address {
        Some(address) => builder.seek_mode_address(mode, address as u32),
        None => builder.seek_mode(mode)
    };

    Ok(())
}

fn do_ascii_directive<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T, builder: &mut BinaryBuilder
) -> Result<(), AssemblerReason> {
    let mut bytes = get_string(iter)?.into_bytes();
    let region = builder.region().ok_or(MissingRegion)?;

    region.raw.data.append(&mut bytes);

    Ok(())
}

fn do_asciiz_directive<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T, builder: &mut BinaryBuilder
) -> Result<(), AssemblerReason> {
    let mut bytes = get_string(iter)?.into_bytes();
    bytes.push(0);

    let region = builder.region().ok_or(MissingRegion)?;

    region.raw.data.append(&mut bytes);

    Ok(())
}

const MAX_ZERO: usize = 0x1000000;

fn do_align_directive<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T, builder: &mut BinaryBuilder
) -> Result<(), AssemblerReason> {
    let shift = get_constant(iter)?;
    let align = 1 << shift;

    let region = builder.region().ok_or(MissingRegion)?;
    let pc = region.raw.address + region.raw.data.len() as u32;

    let (select, remainder) = (pc / align, pc % align);
    let correction = if remainder > 0 { 1 } else { 0 };

    let target = (select + correction) * align;
    let align_count = pc as usize - target as usize;

    if align_count > MAX_ZERO {
        builder.seek_mode_address(builder.state.mode, target)
    } else {
        let mut align_bytes = vec![0; align_count];

        region.raw.data.append(&mut align_bytes);
    }

    Ok(())
}

fn do_space_directive<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T, builder: &mut BinaryBuilder
) -> Result<(), AssemblerReason> {
    let region = builder.region().ok_or(MissingRegion)?;
    let pc = region.raw.address + region.raw.data.len() as u32;

    let byte_count = get_constant(iter)? as usize;

    if byte_count > MAX_ZERO {
        let target = pc + byte_count as u32;

        builder.seek_mode_address(builder.state.mode, target)
    } else {
        let mut space_bytes = vec![0; byte_count];

        region.raw.data.append(&mut space_bytes);
    }

    Ok(())
}

fn get_constants<'a, T: LexerSeekPeekable<'a>>(iter: &mut T) -> Vec<u64> {
    let mut result = vec![];

    while let Some(value) = iter.seek_without(is_solid_kind) {
        match value.kind {
            IntegerLiteral(value) => result.push(value),
            _ => break
        }

        iter.next();
    }

    result
}

fn do_byte_directive<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T, builder: &mut BinaryBuilder
) -> Result<(), AssemblerReason> {
    let mut values: Vec<u8> = get_constants(iter).into_iter()
        .map(|value| value as u8).collect();

    let region = builder.region().ok_or(MissingRegion)?;

    region.raw.data.append(&mut values);

    Ok(())
}

fn do_half_directive<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T, builder: &mut BinaryBuilder
) -> Result<(), AssemblerReason> {
    let mut values: Vec<u8> = get_constants(iter).into_iter()
        .flat_map(|value| {
            let mut array = [0u8; 2];
            LittleEndian::write_u16(&mut array, value as u16);

            array
        }).collect();

    let region = builder.region().ok_or(MissingRegion)?;

    region.raw.data.append(&mut values);

    Ok(())
}

fn do_word_directive<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T, builder: &mut BinaryBuilder
) -> Result<(), AssemblerReason> {
    let mut values: Vec<u8> = get_constants(iter).into_iter()
        .flat_map(|value| {
            let mut array = [0u8; 4];
            LittleEndian::write_u32(&mut array, value as u32);

            array
        }).collect();

    let region = builder.region().ok_or(MissingRegion)?;

    region.raw.data.append(&mut values);

    Ok(())
}

// Don't want to deal with this until coprocessor
fn do_float_directive<'a, T: LexerSeekPeekable<'a>>(
    _: &mut T, _: &mut BinaryBuilder
) -> Result<(), AssemblerReason> {
    Err(UnknownDirective("float".to_string()))
}

fn do_double_directive<'a, T: LexerSeekPeekable<'a>>(
    _: &mut T, _: &mut BinaryBuilder
) -> Result<(), AssemblerReason> {
    Err(UnknownDirective("double".to_string()))
}

fn do_extern_directive<'a, T: LexerSeekPeekable<'a>>(
    iter: &mut T, _: &mut BinaryBuilder
) -> Result<(), AssemblerReason> {
    get_string(iter)?;
    get_constant(iter)?;

    Ok(())
}

pub fn do_directive<'a, T: LexerSeekPeekable<'a>>(
    directive: &'a str, iter: &mut T, builder: &mut BinaryBuilder
) -> Result<(), AssemblerReason> {
    let lowercase = directive.to_lowercase();

    match &lowercase as &str {
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
        _ => Err(UnknownDirective(directive.to_string()))
    }
}