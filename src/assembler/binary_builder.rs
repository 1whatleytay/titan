use std::collections::HashMap;
use std::io::{Cursor};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crate::assembler::assembler_util::AssemblerReason;
use crate::assembler::assembler_util::AssemblerReason::{JumpOutOfRange, MissingInstruction, UnknownLabel};
use crate::assembler::binary::{AddressLabel, Binary, BinarySection, RawRegion};
use crate::assembler::binary::AddressLabel::{Constant, Label};
use crate::assembler::binary_builder::InstructionLabel::{BranchLabel, JumpLabel, LowerLabel, UpperLabel};
use crate::assembler::binary_builder::BinarySection::Text;


fn get_address(label: AddressLabel, map: &HashMap<String, u32>) -> Result<u32, AssemblerReason> {
    match label {
        Constant(value) => Ok(value as u32),
        Label(name) => map.get(&name).copied().ok_or_else(|| UnknownLabel(name))
    }
}

fn add_label(instruction: u32, pc: u32, label: InstructionLabel, map: &HashMap<String, u32>)
             -> Result<u32, AssemblerReason> {
    Ok(match label {
        BranchLabel(label) => {
            let destination = get_address(label, map)?;
            let immediate = (destination >> 2) as i32 - ((pc + 4) >> 2) as i32;

            if immediate > 0xFFFF || immediate < -0x10000 {
                return Err(JumpOutOfRange(destination, pc))
            }

            instruction & 0xFFFF0000 | (immediate as u32 & 0xFFFF)
        }
        JumpLabel(label) => {
            let destination = get_address(label, map)?;
            let lossy_mask = 0xF0000000u32;

            if destination & lossy_mask != (pc + 4) & lossy_mask {
                return Err(JumpOutOfRange(destination, pc))
            }

            let mask = !0u32 << 26;
            let constant = (destination >> 2) & (!0u32 >> 6);

            instruction & mask | constant
        }
        LowerLabel(label) => {
            let destination = get_address(label, map)?;
            let bottom = destination & 0x0000FFFF;

            instruction & 0xFFFF0000 | bottom
        }
        UpperLabel(label) => {
            let destination = get_address(label, map)?;
            let top = (destination & 0xFFFF0000) >> 16;

            instruction & 0xFFFF0000 | top
        }
    })
}

pub struct BinaryBuilderRegion {
    pub raw: RawRegion,
    pub labels: HashMap<usize, InstructionLabel>
}

#[derive(Debug)]
pub enum InstructionLabel {
    BranchLabel(AddressLabel),
    JumpLabel(AddressLabel),
    LowerLabel(AddressLabel),
    UpperLabel(AddressLabel)
}

pub struct BinaryBuilderState {
    pub mode: BinarySection,
    pub indices: HashMap<BinarySection, usize>
}

pub struct BinaryBuilder {
    pub state: BinaryBuilderState,
    pub regions: Vec<BinaryBuilderRegion>,
    pub labels: HashMap<String, u32>,
    pub breakpoints: HashMap<usize, u32> // offset -> pc
}

impl BinaryBuilderState {
    fn index(&self) -> Option<usize> {
        self.indices.get(&self.mode).cloned()
    }

    fn new() -> BinaryBuilderState {
        BinaryBuilderState {
            mode: Text,
            indices: HashMap::new()
        }
    }
}

impl BinaryBuilder {
    pub fn new() -> BinaryBuilder {
        BinaryBuilder {
            state: BinaryBuilderState::new(),
            regions: vec![],
            labels: HashMap::new(),
            breakpoints: HashMap::new()
        }
    }

    fn seek(&mut self, address: u32) -> usize {
        let index = self.regions.len();

        self.regions.push(BinaryBuilderRegion {
            raw: RawRegion { address, data: vec![] }, labels: HashMap::new()
        });

        index
    }

    pub fn seek_mode(&mut self, mode: BinarySection) {
        self.state.mode = mode;

        let index = self.state.index()
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

    pub fn build(self) -> Result<Binary, AssemblerReason> {
        let mut binary = Binary::new();

        for region in self.regions {
            let mut raw = region.raw;

            for (offset, label) in region.labels {
                let pc = raw.address + offset as u32;
                let size = raw.data.len();

                let bytes = &raw.data[offset .. offset + 4];

                let instruction = Cursor::new(bytes).read_u32::<LittleEndian>();
                let Ok(instruction) = instruction else {
                    return Err(MissingInstruction)
                };

                let result = add_label(instruction, pc, label, &self.labels)?;

                let mut_bytes = &mut raw.data[offset .. offset + 4];

                if let Err(_) = Cursor::new(mut_bytes).write_u32::<LittleEndian>(result) {
                    return Err(MissingInstruction)
                }

                assert_eq!(size, raw.data.len());
            }

            binary.regions.push(raw)
        }

        binary.breakpoints = self.breakpoints;

        Ok(binary)
    }
}
