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
    pub regions: Vec<RawRegion>
}

impl Binary {
    pub fn new() -> Binary {
        Binary {
            entry: Text.default_address(),
            regions: vec![]
        }
    }
}
