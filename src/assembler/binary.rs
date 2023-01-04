use std::collections::HashMap;
use std::hash::Hash;
use crate::assembler::binary::BinarySection::{
    Text,
    Data,
    KernelText,
    KernelData
};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BinarySection {
    Text,
    Data,
    KernelText,
    KernelData
}

impl BinarySection {
    pub fn default_address(&self) -> u32 {
        match self {
            Text => 0x00400000,
            Data => 0x10010000,
            KernelText => 0x80000000,
            KernelData => 0x90000000
        }
    }
}

#[derive(Clone, Debug)]
pub enum AddressLabel {
    Constant(u64),
    Label(String)
}

#[derive(Debug)]
pub struct RawRegion {
    pub address: u32,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct Binary {
    pub entry: u32,
    pub regions: Vec<RawRegion>,
    pub breakpoints: HashMap<usize, u32> // offset -> pc
}

pub fn source_breakpoints(map: &HashMap<usize, u32>, source: &str) -> HashMap<usize, u32> {
    let mut result = HashMap::new();

    let mut line_number = 0;
    let mut input = source;

    while let Some(c) = input.chars().next() {
        let next = &input[1..];

        if let Some(pc) = map.get(&(input.as_ptr() as usize)).copied() {
            result.insert(line_number, pc);
        }

        if c == '\n' {
            line_number += 1;
        }

        input = next;
    }

    result
}

impl Binary {
    pub fn source_breakpoints(&self, source: &str) -> HashMap<usize, u32> {
        source_breakpoints(&self.breakpoints, source)
    }

    pub fn new() -> Binary {
        Binary {
            entry: Text.default_address(),
            regions: vec![],
            breakpoints: HashMap::new()
        }
    }
}
