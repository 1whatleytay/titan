use crate::cpu::error::Result;

pub fn dispatch_get_u16<F: FnMut(u32) -> Result<u8>>(mut fetch: F, address: u32) -> Result<u16> {
    Ok(u16::from_le_bytes([
        fetch(address)?,
        fetch(address + 1)?,
    ]))
}

pub fn dispatch_get_u32<F: FnMut(u32) -> Result<u8>>(mut fetch: F, address: u32) -> Result<u32> {
    Ok(u32::from_le_bytes([
        fetch(address)?,
        fetch(address + 1)?,
        fetch(address + 2)?,
        fetch(address + 3)?,
    ]))
}

pub fn dispatch_set_u16<F: FnMut(u32, u8) -> Result<()>>(mut set: F, address: u32, value: u16) -> Result<()> {
    let bytes = value.to_le_bytes();

    set(address, bytes[0])?;
    set(address + 1, bytes[1])
}

pub fn dispatch_set_u32<F: FnMut(u32, u8) -> Result<()>>(mut set: F, address: u32, value: u32) -> Result<()> {
    let bytes = value.to_le_bytes();

    set(address, bytes[0])?;
    set(address + 1, bytes[1])?;
    set(address + 2, bytes[2])?;
    set(address + 3, bytes[3])
}

pub trait Memory {
    fn get(&self, address: u32) -> Result<u8>;
    fn set(&mut self, address: u32, value: u8) -> Result<()>;

    fn get_u16(&self, address: u32) -> Result<u16> {
        dispatch_get_u16(|a| self.get(a), address)
    }

    fn get_u32(&self, address: u32) -> Result<u32> {
        dispatch_get_u32(|a| self.get(a), address)
    }

    fn set_u16(&mut self, address: u32, value: u16) -> Result<()> {
        dispatch_set_u16(|a, b| self.set(a, b), address, value)
    }

    fn set_u32(&mut self, address: u32, value: u32) -> Result<()> {
        dispatch_set_u32(|a, b| self.set(a, b), address, value)
    }
}

pub struct Region {
    pub start: u32,
    pub data: Vec<u8>,
}

pub trait Mountable {
    fn mount(&mut self, region: Region);
}
