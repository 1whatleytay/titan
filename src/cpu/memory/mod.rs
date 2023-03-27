use crate::cpu::error::Result;
use byteorder;
use byteorder::{ByteOrder, LittleEndian};

pub mod region;
pub mod section;

pub trait Memory {
    fn get(&self, address: u32) -> Result<u8>;
    fn set(&mut self, address: u32, value: u8) -> Result<()>;

    fn get_u16(&self, address: u32) -> Result<u16> {
        Ok(LittleEndian::read_u16(
            [self.get(address)?, self.get(address + 1)?].as_slice(),
        ))
    }

    fn get_u32(&self, address: u32) -> Result<u32> {
        Ok(LittleEndian::read_u32(
            [
                self.get(address)?,
                self.get(address + 1)?,
                self.get(address + 2)?,
                self.get(address + 3)?,
            ]
            .as_slice(),
        ))
    }

    fn set_u16(&mut self, address: u32, value: u16) -> Result<()> {
        let bytes = value.to_le_bytes();

        self.set(address, bytes[0])?;
        self.set(address + 1, bytes[1])
    }

    fn set_u32(&mut self, address: u32, value: u32) -> Result<()> {
        let bytes = value.to_le_bytes();

        self.set(address, bytes[0])?;
        self.set(address + 1, bytes[1])?;
        self.set(address + 2, bytes[2])?;
        self.set(address + 3, bytes[3])
    }
}

pub struct Region {
    pub start: u32,
    pub data: Vec<u8>,
}

pub trait Mountable {
    fn mount(&mut self, region: Region);
}
