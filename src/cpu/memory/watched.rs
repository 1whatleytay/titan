use smallvec::SmallVec;
use crate::cpu::Memory;
use crate::cpu::error::Result;
use crate::cpu::memory::{Mountable, Region};
use crate::cpu::memory::watched::BackupValue::{Byte, Short, Word, Null};

pub enum BackupValue {
    Byte(u8),
    Short(u16),
    Word(u32),
    Null
}

pub struct WatchEntry {
    pub address: u32,
    pub previous: BackupValue
}

pub struct WatchedMemory<T: Memory> {
    pub backing: T,
    log: SmallVec<[WatchEntry; 4]>
}

impl WatchEntry {
    pub fn apply<Mem: Memory>(self, memory: &mut Mem) -> Result<()> {
        match self.previous {
            Byte(value) => memory.set(self.address, value),
            Short(value) => memory.set_u16(self.address, value),
            Word(value) => memory.set_u32(self.address, value),
            Null => { Ok(()) }
        }
    }
}

impl<T: Memory> WatchedMemory<T> {
    pub fn new(backing: T) -> WatchedMemory<T> {
        WatchedMemory { backing, log: SmallVec::new() }
    }

    pub fn take(&mut self) -> SmallVec<[WatchEntry; 4]> {
        std::mem::take(&mut self.log)
    }
}

impl<T: Memory> Memory for WatchedMemory<T> {
    fn get(&self, address: u32) -> Result<u8> {
        self.backing.get(address)
    }

    fn set(&mut self, address: u32, value: u8) -> Result<()> {
        self.log.push(WatchEntry {
            address, previous: self.backing.get(address).map_or(Null, Byte)
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
            address, previous: self.backing.get_u16(address).map_or(Null, Short)
        });

        self.backing.set_u16(address, value)
    }

    fn set_u32(&mut self, address: u32, value: u32) -> Result<()> {
        self.log.push(WatchEntry {
            address, previous: self.backing.get_u32(address).map_or(Null, Word)
        });

        self.backing.set_u32(address, value)
    }
}

impl<T: Memory + Mountable> Mountable for WatchedMemory<T> {
    fn mount(&mut self, region: Region) {
        self.backing.mount(region)
    }
}
