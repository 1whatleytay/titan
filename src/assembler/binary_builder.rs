use crate::assembler::assembler_util::AssemblerError;
use crate::assembler::assembler_util::AssemblerReason::{
    JumpOutOfRange, MissingInstruction, UnknownLabel,
};
use crate::assembler::binary::AddressLabel::{Constant, Label};
use crate::assembler::binary::{AddressLabel, Binary, BinaryBreakpoint, BinarySection, RawRegion};
use crate::assembler::binary_builder::BinarySection::Text;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

fn get_address(label: AddressLabel, map: &HashMap<String, u32>) -> Result<u32, AssemblerError> {
    match label {
        Constant(value) => Ok(value as u32),
        Label(name) => map
            .get(&name.name)
            .copied()
            .map(|value| value.wrapping_add(name.offset as u32))
            .ok_or(AssemblerError {
                start: Some(name.start),
                reason: UnknownLabel(name.name),
            }),
    }
}

fn add_label(
    instruction: u32,
    pc: u32,
    start: usize,
    label: InstructionLabel,
    map: &HashMap<String, u32>,
) -> Result<u32, AssemblerError> {
    let make_out_of_range = |destination: u32| AssemblerError {
        start: Some(start),
        reason: JumpOutOfRange(destination, pc),
    };

    let destination = get_address(label.label, map)?;

    Ok(match label.kind {
        InstructionLabelKind::Branch => {
            let immediate = (destination >> 2) as i32 - ((pc + 4) >> 2) as i32;

            if !(-0x10000..=0xFFFF).contains(&immediate) {
                return Err(make_out_of_range(destination));
            }

            instruction & 0xFFFF0000 | (immediate as u32 & 0xFFFF)
        }
        InstructionLabelKind::Jump => {
            let lossy_mask = 0xF0000000u32;

            if destination & lossy_mask != (pc + 4) & lossy_mask {
                return Err(make_out_of_range(destination));
            }

            let mask = !0u32 << 26;
            let constant = (destination >> 2) & (!0u32 >> 6);

            instruction & mask | constant
        }
        InstructionLabelKind::Lower => {
            let bottom = destination & 0x0000FFFF;

            instruction & 0xFFFF0000 | bottom
        }
        InstructionLabelKind::Upper => {
            let top = (destination & 0xFFFF0000) >> 16;

            instruction & 0xFFFF0000 | top
        }
        InstructionLabelKind::Full => destination,
    })
}

pub struct BinaryBuilderLabel {
    pub offset: usize,
    pub start: usize,
    pub label: InstructionLabel,
}

pub struct BinaryBuilderRegion {
    pub raw: RawRegion,
    pub labels: Vec<BinaryBuilderLabel>, // start
}

#[derive(Debug)]
pub enum InstructionLabelKind {
    Branch,
    Jump,
    Lower,
    Upper,
    Full,
}

#[derive(Debug)]
pub struct InstructionLabel {
    pub kind: InstructionLabelKind,
    pub label: AddressLabel,
}

pub struct BinaryBuilderState {
    pub mode: BinarySection,
    pub indices: HashMap<BinarySection, usize>,
}

pub struct BinaryBuilder {
    pub state: BinaryBuilderState,
    pub regions: Vec<BinaryBuilderRegion>,
    pub labels: HashMap<String, u32>,
    pub breakpoints: Vec<BinaryBreakpoint>,
}

impl BinaryBuilderState {
    fn index(&self) -> Option<usize> {
        self.indices.get(&self.mode).cloned()
    }

    fn new() -> BinaryBuilderState {
        BinaryBuilderState {
            mode: Text,
            indices: HashMap::new(),
        }
    }
}

impl BinaryBuilder {
    pub fn new() -> BinaryBuilder {
        BinaryBuilder {
            state: BinaryBuilderState::new(),
            regions: vec![],
            labels: HashMap::new(),
            breakpoints: vec![],
        }
    }

    fn seek(&mut self, address: u32) -> usize {
        let index = self.regions.len();

        self.regions.push(BinaryBuilderRegion {
            raw: RawRegion {
                address,
                data: vec![],
            },
            labels: vec![],
        });

        index
    }

    pub fn seek_mode(&mut self, mode: BinarySection) {
        self.state.mode = mode;

        let index = self
            .state
            .index()
            .unwrap_or_else(|| self.seek(mode.default_address()));

        self.state.indices.insert(mode, index);
    }

    pub fn seek_mode_address(&mut self, mode: BinarySection, address: u32) {
        self.state.mode = mode;

        let index = self.seek(address);
        self.state.indices.insert(mode, index);
    }

    pub fn region(&mut self) -> Option<&mut BinaryBuilderRegion> {
        let Some(index) = self.state.index() else { return None };

        Some(&mut self.regions[index])
    }

    pub fn build(self) -> Result<Binary, AssemblerError> {
        let mut binary = Binary::new();

        const MISSING: AssemblerError = AssemblerError {
            start: None,
            reason: MissingInstruction,
        };

        for region in self.regions {
            let mut raw = region.raw;

            for label in region.labels {
                let pc = raw.address + label.offset as u32;
                let size = raw.data.len();

                let bytes = &raw.data[label.offset..label.offset + 4];

                let instruction = Cursor::new(bytes).read_u32::<LittleEndian>();
                let Ok(instruction) = instruction else {
                    return Err(MISSING)
                };

                let result = add_label(instruction, pc, label.start, label.label, &self.labels)?;

                let mut_bytes = &mut raw.data[label.offset..label.offset + 4];

                if Cursor::new(mut_bytes)
                    .write_u32::<LittleEndian>(result)
                    .is_err()
                {
                    return Err(MISSING);
                }

                assert_eq!(size, raw.data.len());
            }

            binary.regions.push(raw)
        }

        binary.breakpoints = self.breakpoints;

        Ok(binary)
    }
}
