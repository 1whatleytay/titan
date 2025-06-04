use crate::assembler::utilities::AssemblerError;
use crate::assembler::utilities::AssemblerReason::{
    JumpOutOfRange, MissingInstruction, UnknownLabel,
};
use titan_shared::assembler::binary::{
    Binary, BinaryBreakpoint, BinarySection, RawRegion, RegionFlags,
};
use crate::assembler::binary_builder::BinarySection::Text;
use crate::assembler::lexer::Location;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::collections::HashMap;
use std::io::Cursor;
use crate::assembler::binary_builder::AddressLabel::{Constant, Label};
use crate::assembler::instruction_builder::{InstructionBuilder, SplitImmediate};

#[derive(Clone, Debug)]
pub struct NamedLabel {
    pub name: String,
    pub location: Location,
    pub offset: u64,
}

#[derive(Clone, Debug)]
pub enum AddressLabel {
    Constant(u64),
    Label(NamedLabel), // usize -> start, offset
}

fn get_address(label: AddressLabel, map: &HashMap<String, u32>) -> Result<u32, AssemblerError> {
    match label {
        Constant(value) => Ok(value as u32),
        Label(name) => map
            .get(&name.name)
            .copied()
            .map(|value| value.wrapping_add(name.offset as u32))
            .ok_or(AssemblerError {
                location: Some(name.location),
                reason: UnknownLabel(name.name),
            }),
    }
}

fn add_label(
    instruction: u32,
    pc: u32,
    location: Location,
    label: InstructionLabel,
    map: &HashMap<String, u32>,
) -> Result<u32, AssemblerError> {
    let make_out_of_range = |destination: u32| AssemblerError {
        location: Some(location),
        reason: JumpOutOfRange(destination, pc),
    };

    let destination = get_address(label.label, map)?;

    Ok(match label.kind {
        InstructionLabelKind::JumpAndLink => {
            let immediate = ((destination.wrapping_sub(pc)) >> 1) as i32;

            // we have 20 bits of signed immediate, hopefully this is right
            if !(-0x80000 ..= 0x7ffff).contains(&immediate) {
                return Err(make_out_of_range(destination))
            }

            InstructionBuilder(instruction)
                .with_jal_imm(immediate)
                .0
        }
        InstructionLabelKind::Branch => {
            let immediate = ((destination.wrapping_sub(pc)) >> 1) as i32;

            // we have 12 bits of signed immediate, hopefully this is right
            if !(-0x800 ..= 0x7ff).contains(&immediate) {
                return Err(make_out_of_range(destination))
            }

            InstructionBuilder(instruction)
                .with_branch_imm(immediate as i16)
                .0
        }
        InstructionLabelKind::Upper20 => {
            let split = SplitImmediate::from_immediate(destination);

            InstructionBuilder(instruction)
                .with_upper_imm(split.upper)
                .0
        }
        InstructionLabelKind::Lower12 => {
            let split = SplitImmediate::from_immediate(destination);

            InstructionBuilder(instruction)
                .with_normal_imm(split.lower)
                .0
        }
        InstructionLabelKind::Full => destination,
    })
}

pub struct BinaryBuilderLabel {
    pub offset: usize,
    pub location: Location,
    pub label: InstructionLabel,
}

pub struct BinaryBuilderRegion {
    pub raw: RawRegion,
    pub labels: Vec<BinaryBuilderLabel>, // start
}

#[derive(Debug)]
pub enum InstructionLabelKind {
    // All of these are assumed to be based (not compressed)
    JumpAndLink,
    Branch,
    Upper20,
    Lower12,
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
    pub entry: Option<AddressLabel>,
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
            entry: None,
            state: BinaryBuilderState::new(),
            regions: vec![],
            labels: HashMap::new(),
            breakpoints: vec![],
        }
    }

    fn seek(&mut self, address: u32, flags: RegionFlags) -> usize {
        let index = self.regions.len();

        self.regions.push(BinaryBuilderRegion {
            raw: RawRegion {
                flags,
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
            .unwrap_or_else(|| self.seek(mode.default_address(), mode.into()));

        self.state.indices.insert(mode, index);
    }

    pub fn seek_mode_address(&mut self, mode: BinarySection, address: u32) {
        self.state.mode = mode;

        let index = self.seek(address, mode.into());
        self.state.indices.insert(mode, index);
    }

    pub fn region(&mut self) -> Option<&mut BinaryBuilderRegion> {
        let index = self.state.index()?;

        Some(&mut self.regions[index])
    }

    pub fn build(self) -> Result<Binary, AssemblerError> {
        let mut binary = Binary::new();

        const MISSING: AssemblerError = AssemblerError {
            location: None,
            reason: MissingInstruction,
        };

        if let Some(entry) = self.entry {
            let address = get_address(entry, &self.labels)?;

            binary.entry = address;
        }

        for region in self.regions {
            let mut raw = region.raw;

            for label in region.labels {
                let pc = raw.address + label.offset as u32;
                let size = raw.data.len();

                let bytes = &raw.data[label.offset..label.offset + 4];

                let instruction = Cursor::new(bytes).read_u32::<LittleEndian>();
                let Ok(instruction) = instruction else {
                    return Err(MISSING);
                };

                let result = add_label(instruction, pc, label.location, label.label, &self.labels)?;

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
        binary.labels = self.labels;

        Ok(binary)
    }
}
