use crate::cpu::error::Error::MemoryUnmapped;
use crate::cpu::Memory;
use crate::cpu::memory::{Mountable, Region};
use crate::cpu::error::Result;

const SECTION_SELECTOR_START: u32 = 16;

const SECTION_SELECTOR_MASK: u32 = !0u32 << SECTION_SELECTOR_START;
const SECTION_INDEX_MASK: u32 = !0u32 >> (32 - SECTION_SELECTOR_START);
const SECTION_COUNT: usize = 1 << (32 - SECTION_SELECTOR_START);
const SECTION_SIZE: usize = 1 << SECTION_SELECTOR_START;

const INITIAL_BYTE: u8 = 0xCC;

type Section = Box<[u8; SECTION_SIZE]>;

pub struct SectionMemory {
    sections: Box<[Option<Section>; SECTION_COUNT]>
}

impl SectionMemory {
    pub fn new() -> SectionMemory {
        let sections = vec![None; SECTION_COUNT].try_into().unwrap();

        SectionMemory { sections }
    }

    fn create_section(&mut self, selector: usize) -> &mut [u8; SECTION_SIZE] {
        let section = Section::new([INITIAL_BYTE; SECTION_SIZE]);

        self.sections[selector] = Some(section);

        self.sections[selector].as_mut().unwrap().as_mut()
    }

    fn pick_section(&mut self, selector: usize) -> &mut [u8; SECTION_SIZE] {
        if self.sections[selector].is_some() {
            self.sections[selector].as_mut().unwrap();
        }

        self.create_section(selector)
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

        if let Some(section) = &self.sections[section] {
            Ok(section[index])
        } else {
            Err(MemoryUnmapped(address))
        }
    }

    fn set(&mut self, address: u32, value: u8) -> Result<()> {
        let (section, index) = split(address);

        if let Some(section) = &mut self.sections[section] {
            section[index] = value;

            Ok(())
        } else {
            Err(MemoryUnmapped(address))
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