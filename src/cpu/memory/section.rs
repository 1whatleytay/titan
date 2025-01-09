use crate::cpu::error::Error::{MemoryAlign, MemoryUnmapped};
use crate::cpu::error::{MemoryAlignment, Result};
use crate::cpu::memory::section::Section::{Data, Empty, Writable};
use crate::cpu::memory::{Mountable, Region};
use crate::cpu::Memory;
use std::fmt::{Debug, Formatter};
use Section::Listen;

const SECTION_SELECTOR_START: u32 = 16;

const SECTION_SELECTOR_MASK: u32 = !0u32 << SECTION_SELECTOR_START;
const SECTION_INDEX_MASK: u32 = !0u32 >> (32 - SECTION_SELECTOR_START);
const SECTION_COUNT: usize = 1 << (32 - SECTION_SELECTOR_START);
const SECTION_SIZE: usize = 1 << SECTION_SELECTOR_START;

const INITIAL_BYTE: u8 = 0xCC;

pub trait ListenResponder {
    fn read(&self, address: u32) -> Result<u8>;
    fn write(&mut self, address: u32, value: u8) -> Result<()>;
}

#[derive(Clone)]
pub struct DefaultResponder {}

impl ListenResponder for DefaultResponder {
    fn read(&self, address: u32) -> Result<u8> {
        Err(MemoryUnmapped(address))
    }

    fn write(&mut self, address: u32, _: u8) -> Result<()> {
        Err(MemoryUnmapped(address))
    }
}

#[derive(Clone)]
enum Section<T: ListenResponder> {
    Empty,
    Data(Box<[u8; SECTION_SIZE]>),
    Listen(T),
    Writable(u8),
}

impl<T: ListenResponder> Debug for Section<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Empty => "Section [Unmounted]",
                Data(_) => "Section [Data Mounted]",
                Listen(_) => "Section [Listen Mounted]",
                Writable(_) => "Section [Writable Mounted]",
            }
        )
    }
}

pub struct SectionMemory<T: ListenResponder> {
    sections: Box<[Section<T>; SECTION_COUNT]>,
}

impl<T: ListenResponder + Clone> Clone for SectionMemory<T> {
    fn clone(&self) -> Self {
        let sections = (0..SECTION_COUNT)
            .map(|i| self.sections[i].clone())
            .collect::<Vec<Section<T>>>()
            .try_into()
            .unwrap();

        SectionMemory { sections }
    }
}

impl<T: ListenResponder> SectionMemory<T> {
    pub fn new() -> SectionMemory<T> {
        let sections = vec![(); SECTION_COUNT]
            .into_iter()
            .map(|_| Empty)
            .collect::<Vec<Section<T>>>()
            .try_into()
            .unwrap();

        SectionMemory { sections }
    }

    fn allocate_data(value: u8) -> Box<[u8; SECTION_SIZE]> {
        Box::new([value; SECTION_SIZE])
    }

    fn create_section(&mut self, selector: usize) -> &mut [u8; SECTION_SIZE] {
        self.sections[selector] = Data(Self::allocate_data(INITIAL_BYTE));

        match &mut self.sections[selector] {
            Data(data) => data.as_mut(),
            _ => panic!("Expected Data Section"),
        }
    }

    fn pick_section(&mut self, selector: usize) -> &mut [u8; SECTION_SIZE] {
        // Complicated sidestepping of capting mut.
        match &self.sections[selector] {
            Data(_) => match &mut self.sections[selector] {
                Data(data) => data,
                _ => panic!(),
            },
            _ => self.create_section(selector),
        }
    }

    // selector is NOT an address! Leading 16-bits.
    pub fn mount_listen(&mut self, selector: usize, listener: T) {
        self.sections[selector] = Listen(listener);
    }

    pub fn mount_writable(&mut self, selector: usize, value: u8) {
        // If the section isn't already writable...
        if let Empty = self.sections[selector] {
            self.sections[selector] = Writable(value)
        }
    }
}

impl<T: ListenResponder> Default for SectionMemory<T> {
    fn default() -> Self {
        Self::new()
    }
}

const fn split(address: u32) -> (usize, usize) {
    let section = ((address & SECTION_SELECTOR_MASK) >> SECTION_SELECTOR_START) as usize;
    let index = (address & SECTION_INDEX_MASK) as usize;

    (section, index)
}

impl<T: ListenResponder> Memory for SectionMemory<T> {
    fn get(&self, address: u32) -> Result<u8> {
        let (section, index) = split(address);

        match &self.sections[section] {
            Data(section) => Ok(section[index]),
            Listen(responder) => responder.read(address),
            Empty => Err(MemoryUnmapped(address)),
            Writable(value) => Ok(*value),
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
            Empty => Err(MemoryUnmapped(address)),
            Writable(default) => {
                let mut data = Self::allocate_data(*default);
                data[index] = value;

                self.sections[section] = Data(data);

                Ok(())
            }
        }
    }

    fn get_u16(&self, address: u32) -> Result<u16> {
        if address % 2 != 0 {
            return Err(MemoryAlign(MemoryAlignment::Half, address));
        }

        let (section, index) = split(address);

        fn glue(a: u8, b: u8) -> u16 {
            a as u16 | ((b as u16) << 8)
        }

        match &self.sections[section] {
            Data(section) => Ok(glue(section[index], section[index + 1])),
            Listen(responder) => Ok(glue(responder.read(address)?, responder.read(address + 1)?)),
            Empty => Err(MemoryUnmapped(address)),
            Writable(value) => Ok(glue(*value, *value)),
        }
    }

    fn get_u32(&self, address: u32) -> Result<u32> {
        if address % 4 != 0 {
            return Err(MemoryAlign(MemoryAlignment::Word, address));
        }

        let (section, index) = split(address);

        fn glue(a: u8, b: u8, c: u8, d: u8) -> u32 {
            a as u32 | ((b as u32) << 8) | ((c as u32) << 16) | ((d as u32) << 24)
        }

        match &self.sections[section] {
            Data(section) => Ok(glue(
                section[index],
                section[index + 1],
                section[index + 2],
                section[index + 3],
            )),
            Listen(responder) => Ok(glue(
                responder.read(address)?,
                responder.read(address + 1)?,
                responder.read(address + 2)?,
                responder.read(address + 3)?,
            )),
            Empty => Err(MemoryUnmapped(address)),
            Writable(value) => Ok(glue(*value, *value, *value, *value)),
        }
    }

    fn set_u16(&mut self, address: u32, value: u16) -> Result<()> {
        if address % 2 != 0 {
            return Err(MemoryAlign(MemoryAlignment::Half, address));
        }

        let (section, index) = split(address);

        let (a, b) = ((value & 0xFF) as u8, ((value >> 8) & 0xFF) as u8);

        match &mut self.sections[section] {
            Data(section) => {
                section[index] = a;
                section[index + 1] = b;

                Ok(())
            }
            Listen(responder) => {
                responder.write(address, a)?;
                responder.write(address + 1, b)
            }
            Empty => Err(MemoryUnmapped(address)),
            Writable(default) => {
                let mut data = Self::allocate_data(*default);
                data[index] = a;
                data[index + 1] = b;

                self.sections[section] = Data(data);

                Ok(())
            }
        }
    }

    fn set_u32(&mut self, address: u32, value: u32) -> Result<()> {
        if address % 4 != 0 {
            return Err(MemoryAlign(MemoryAlignment::Word, address));
        }

        let (section, index) = split(address);

        let (a, b, c, d) = (
            (value & 0xFF) as u8,
            ((value >> 8) & 0xFF) as u8,
            ((value >> 16) & 0xFF) as u8,
            ((value >> 24) & 0xFF) as u8,
        );

        match &mut self.sections[section] {
            Data(section) => {
                section[index] = a;
                section[index + 1] = b;
                section[index + 2] = c;
                section[index + 3] = d;

                Ok(())
            }
            Listen(responder) => {
                responder.write(address, a)?;
                responder.write(address + 1, b)?;
                responder.write(address + 2, c)?;
                responder.write(address + 3, d)
            }
            Empty => Err(MemoryUnmapped(address)),
            Writable(default) => {
                let mut data = Self::allocate_data(*default);
                data[index] = a;
                data[index + 1] = b;
                data[index + 2] = c;
                data[index + 3] = d;

                self.sections[section] = Data(data);

                Ok(())
            }
        }
    }
}

impl<T: ListenResponder> Mountable for SectionMemory<T> {
    fn mount(&mut self, region: Region) {
        let (start_selector, start_index) = split(region.start);
        let (end_selector, end_index) = split(region.start + region.data.len() as u32);

        let mut selector = start_selector;
        let mut data_index = 0;

        while selector <= end_selector {
            let section = self.pick_section(selector);

            let begin = if selector == start_selector {
                start_index
            } else {
                0
            };
            let end = if selector == end_selector {
                end_index
            } else {
                SECTION_SIZE
            };

            for i in section.iter_mut().take(end).skip(begin) {
                *i = region.data[data_index];
                data_index += 1;
            }

            selector += 1
        }
    }
}
