use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::cpu::error::Error::{MemoryAlign, MemoryBoundary, MemoryUnmapped};
use crate::cpu::error::Result;

pub struct Region {
    pub start: u32,
    pub data: Vec<u8>
}

pub struct Memory {
    regions: Vec<Region>
}

impl Region {
    pub fn contains(&self, address: u32) -> bool {
        self.start <= address && address < self.start + self.data.len() as u32
    }
}

type Endian = LittleEndian;

impl Memory {
    pub fn get(&self, address: u32) -> Result<u8> {
        for region in &self.regions {
            if region.contains(address) {
                return Ok(region.data[(address - region.start) as usize])
            }
        }

        Err(MemoryUnmapped(address))
    }

    pub fn get_u16(&self, address: u32) -> Result<u16> {
        if address % 2 != 0 {
            return Err(MemoryAlign(address))
        }

        for region in &self.regions {
            if region.contains(address) {
                let start = (address - region.start) as usize;
                let data = (&region.data[start .. start + 2]).read_u16::<Endian>();

                return data.map_err(|_| MemoryBoundary(address))
            }
        }

        Err(MemoryUnmapped(address))
    }

    pub fn get_u32(&self, address: u32) -> Result<u32> {
        if address % 4 != 0 {
            return Err(MemoryAlign(address))
        }

        for region in &self.regions {
            if region.contains(address) {
                let start = (address - region.start) as usize;
                let data = (&region.data[start .. start + 4]).read_u32::<Endian>();

                return data.map_err(|_| MemoryBoundary(address))
            }
        }

        Err(MemoryBoundary(address))
    }

    pub fn set(&mut self, address: u32, value: u8) -> Result<()> {
        for region in &mut self.regions {
            if region.contains(address) {
                region.data[(address - region.start) as usize] = value;

                return Ok(())
            }
        }

        Err(MemoryUnmapped(address))
    }

    pub fn set_u16(&mut self, address: u32, value: u16) -> Result<()> {
        if address % 2 != 0 {
            panic!("Address 0x{:08x} is not aligned for u16 read.", address);
        }

        for region in &mut self.regions {
            if region.contains(address) {
                let start = (address - region.start) as usize;

                (&mut region.data[start .. start + 2])
                    .write_u16::<Endian>(value)
                    .unwrap();

                return Ok(())
            }
        }

        Err(MemoryUnmapped(address))
    }

    pub fn set_u32(&mut self, address: u32, value: u32) -> Result<()> {
        if address % 4 != 0 {
            panic!("Address 0x{:08x} is not aligned for u32 read.", address);
        }

        for region in &mut self.regions {
            if region.contains(address) {
                let start = (address - region.start) as usize;

                (&mut region.data[start .. start + 4])
                    .write_u32::<Endian>(value)
                    .unwrap();

                return Ok(())
            }
        }

        Err(MemoryUnmapped(address))
    }

    pub fn mount(&mut self, region: Region) {
        self.regions.push(region)
    }

    pub fn new() -> Memory {
        return Memory {
            regions: vec![]
        }
    }
}
