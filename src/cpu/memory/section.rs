use std::fmt::{Debug, Formatter};
use Section::Listen;
use crate::cpu::error::Error::MemoryUnmapped;
use crate::cpu::Memory;
use crate::cpu::memory::{Mountable, Region};
use crate::cpu::error::Result;
use crate::cpu::memory::section::Section::{Data, Empty};

const SECTION_SELECTOR_START: u32 = 16;

const SECTION_SELECTOR_MASK: u32 = !0u32 << SECTION_SELECTOR_START;
const SECTION_INDEX_MASK: u32 = !0u32 >> (32 - SECTION_SELECTOR_START);
const SECTION_COUNT: usize = 1 << (32 - SECTION_SELECTOR_START);
const SECTION_SIZE: usize = 1 << SECTION_SELECTOR_START;

const INITIAL_BYTE: u8 = 0xCC;

enum Section {
    Empty,
    Data(Box<[u8; SECTION_SIZE]>),
    Listen(Box<dyn ListenResponder>)
}

impl Debug for Section {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Empty => "Section [Unmounted]",
            Data(_) => "Section [Data Mounted]",
            Listen(_) => "Section [Listen Mounted]"
        })
    }
}

pub trait ListenResponder {
    fn read(&self, address: u32) -> Result<u8>;
    fn write(&mut self, address: u32, value: u8) -> Result<()>;
}

pub struct SectionMemory {
    sections: Box<[Section; SECTION_COUNT]>
}

impl SectionMemory {
    pub fn new() -> SectionMemory {
        let sections = vec![(); SECTION_COUNT]
            .into_iter()
            .map(|_| Empty)
            .collect::<Vec<Section>>()
            .try_into()
            .unwrap();

        SectionMemory { sections }
    }

    fn create_section(&mut self, selector: usize) -> &mut [u8; SECTION_SIZE] {
        self.sections[selector] = Data(Box::new([INITIAL_BYTE; SECTION_SIZE]));

        match &mut self.sections[selector] {
            Data(data) => data.as_mut(),
            _ => panic!("Expected Data Section")
        }
    }

    fn pick_section(&mut self, selector: usize) -> &mut [u8; SECTION_SIZE] {
        // Complicated sidestepping of capting mut.
        match &self.sections[selector] {
            Data(_) => {
                match &mut self.sections[selector] {
                    Data(data) => data,
                    _ => panic!()
                }
            }
            _ => {
                self.create_section(selector)
            }
        }
    }

    // selector is NOT an address! Leading 16-bits.
    pub fn mount_listen(&mut self, selector: usize, listener: Box<dyn ListenResponder>) {
        self.sections[selector] = Listen(listener);
    }
}

const fn split(address: u32) -> (usize, usize) {
    let section = ((address & SECTION_SELECTOR_MASK) >> SECTION_SELECTOR_START) as usize;
    let index = (address & SECTION_INDEX_MASK) as usize;

    (section, index)
}

impl Memory for SectionMemory {
    fn get(&self, address: u32) -> Result<u8> {
        let (section, index) = split(address);

        match &self.sections[section] {
            Data(section) => Ok(section[index]),
            Listen(responder) => responder.read(address),
            Empty => Err(MemoryUnmapped(address))
        }
    }

    fn set(&mut self, address: u32, value: u8) -> Result<()> {
        let (section, index) = split(address);

        match &mut self.sections[section] {
            Data(section) => {
                section[index] = value;

                Ok(())
            }
            Listen(responder) => responder.write(address, value),
            Empty => Err(MemoryUnmapped(address))
        }
    }
}

impl Mountable for SectionMemory {
    fn mount(&mut self, region: Region) {
        let (start_selector, start_index) = split(region.start);
        let (end_selector, end_index) = split(region.start + region.data.len() as u32);

        let mut selector = start_selector;
        let mut data_index = 0;

        while selector <= end_selector {
            let section = self.pick_section(selector);

            let begin = if selector == start_selector { start_index } else { 0 };
            let end = if selector == end_selector { end_index } else { SECTION_SIZE };

            for i in begin .. end {
                let byte = region.data[data_index];
                data_index += 1;

                section[i] = byte
            }

            selector += 1
        }
    }
}
