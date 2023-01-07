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
    pub breakpoints: HashMap<u32, usize> // pc -> offset
}

pub fn flip_breakpoints<Key: Copy + Hash + Eq, Value: Copy + Hash + Eq>(
    map: &HashMap<Key, Value>
) -> HashMap<Value, Vec<Key>> {
    let mut result: HashMap<Value, Vec<Key>> = HashMap::new();

    for (key, value) in map {
        if let Some(list) = result.get_mut(value) {
            list.push(*key);
        } else {
            result.insert(*value, vec![*key]);
        }
    }

    result
}

pub fn source_breakpoints(map: &HashMap<u32, usize>, source: &str) -> HashMap<u32, usize> {
    let mut result: HashMap<u32, usize> = HashMap::new();
    let flipped = flip_breakpoints(&map);

    let mut line_number = 0;
    let mut input = source;

    while let Some(c) = input.chars().next() {
        let next = &input[1..];

        let start = input.as_ptr() as usize - source.as_ptr() as usize;
        if let Some(pcs) = flipped.get(&start) {
            for pc in pcs {
                result.insert(*pc, line_number);
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
    pub fn source_breakpoints(&self, source: &str) -> HashMap<u32, usize> {
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
