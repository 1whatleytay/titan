use crate::cpu::Memory;
use crate::cpu::error::Result;
use crate::cpu::memory::watched::WriteValue::{Byte, Short, Word, Null};

enum WriteValue {
    Byte(u8),
    Short(u16),
    Word(u32),
    Null
}

struct WatchEntry {
    address: u32,
    write: WriteValue
}

struct WatchedMemory<T: Memory> {
    backing: T,
    log: Vec<WatchEntry>
}

impl<T: Memory> Memory for WatchedMemory<T> {
    fn get(&self, address: u32) -> Result<u8> {
        self.backing.get(address)
    }

    fn set(&mut self, address: u32, value: u8) -> Result<()> {
        self.log.push(WatchEntry {
            address, write: self.backing.get(address).map_or(Null, Byte)
        });

        self.backing.set(address, value)
    }

    fn get_u16(&self, address: u32) -> Result<u16> {
        self.backing.get_u16(address)
    }

    fn get_u32(&self, address: u32) -> Result<u32> {
        self.backing.get_u32(address)
    }

    fn set_u16(&mut self, address: u32, value: u16) -> Result<()> {
        self.log.push(WatchEntry {
            address, write: self.backing.get_u16(address).map_or(Null, Short)
        });

        self.backing.set_u16(address, value)
    }

    fn set_u32(&mut self, address: u32, value: u32) -> Result<()> {
        self.log.push(WatchEntry {
            address, write: self.backing.get_u32(address).map_or(Null, Word)
        });

        self.backing.set_u32(address, value)
    }
}
