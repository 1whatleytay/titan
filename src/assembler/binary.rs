use crate::assembler::binary::BinarySection::{Data, KernelData, KernelText, Text};
use std::collections::HashMap;
use std::hash::Hash;
use crate::assembler::lexer::Location;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BinarySection {
    Text,
    Data,
    KernelText,
    KernelData,
}

impl BinarySection {
    pub fn is_data(&self) -> bool {
        matches!(self, Data | KernelData)
    }

    pub fn is_text(&self) -> bool {
        matches!(self, Text | KernelText)
    }

    pub fn default_address(&self) -> u32 {
        match self {
            Text => 0x00400000,
            Data => 0x10010000,
            KernelText => 0x80000000,
            KernelData => 0x90000000,
        }
    }
}

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

#[derive(Debug)]
pub struct RawRegion {
    pub address: u32,
    pub data: Vec<u8>,
}

impl RawRegion {
    pub fn pc(&self) -> Option<u32> {
        self.address.checked_add(self.data.len() as u32)
    }

    pub fn wrapping_pc(&self) -> u32 {
        self.address.wrapping_add(self.data.len() as u32)
    }
}

#[derive(Debug)]
pub struct BinaryBreakpoint {
    pub location: Location,
    pub pcs: Vec<u32>,
}

#[derive(Debug)]
pub struct Binary {
    pub entry: u32,
    pub regions: Vec<RawRegion>,
    pub breakpoints: Vec<BinaryBreakpoint>, // pc -> offset
}

fn build_breakpoint_map(
    breakpoints: &Vec<BinaryBreakpoint>, id: usize
) -> HashMap<usize, Vec<&BinaryBreakpoint>> {
    // offset -> breakpoints
    let mut result: HashMap<usize, Vec<&BinaryBreakpoint>> = HashMap::new();

    for breakpoint in breakpoints {
        if breakpoint.location.source != id {
            continue
        }

        let offset = breakpoint.location.index;
        let list = result.entry(offset).or_default();

        list.push(breakpoint);
    }

    result
}

// Similar definition, but offset is the line number.
pub struct SourceBreakpoint {
    pub line: usize,
    pub pcs: Vec<u32>, // anchor breakpoint is the first in the list
}

pub fn source_breakpoints(map: &Vec<BinaryBreakpoint>, source: &str, id: usize) -> Vec<SourceBreakpoint> {
    let mut result: Vec<SourceBreakpoint> = vec![];
    let map = build_breakpoint_map(map, id);

    let mut line_number = 0;
    let mut input = source;

    while let Some(c) = input.chars().next() {
        let next = &input[c.len_utf8()..];

        let start = input.as_ptr() as usize - source.as_ptr() as usize;
        if let Some(breakpoints) = map.get(&start) {
            for breakpoint in breakpoints {
                result.push(SourceBreakpoint {
                    line: line_number,
                    pcs: breakpoint.pcs.clone(),
                });
            }
        }

        if c == '\n' {
            line_number += 1;
        }

        input = next;
    }

    result
}

impl Binary {
    pub fn source_breakpoints(&self, source: &str, id: usize) -> Vec<SourceBreakpoint> {
        source_breakpoints(&self.breakpoints, source, id)
    }

    pub fn new() -> Binary {
        Binary {
            entry: Text.default_address(),
            regions: vec![],
            breakpoints: vec![],
        }
    }
}

impl Default for Binary {
    fn default() -> Self {
        Self::new()
    }
}
