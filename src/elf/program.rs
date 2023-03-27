use crate::elf::error::Error::InvalidHeaderType;
use crate::elf::error::Result;
use crate::elf::landmark::Landmark::Data;
use crate::elf::landmark::Landmarks;
use crate::elf::landmark::PointerSize::Bit32;
use crate::elf::program::ProgramHeaderType::Null;
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::io::SeekFrom::Start;
use std::io::{Read, Seek, Write};

#[derive(ToPrimitive, FromPrimitive, Copy, Clone, Debug)]
pub enum ProgramHeaderType {
    Null = 0,
    Load = 1,
    Dynamic = 2,
    Interpreter = 3,
    Note = 4,
    ProgramHeader = 6,
}

bitflags! {
    pub struct ProgramHeaderFlags: u32 {
        const EXECUTABLE = 1 << 0;
        const WRITABLE = 1 << 1;
        const READABLE = 1 << 2;
    }
}

impl ProgramHeaderFlags {
    pub fn known_mask() -> u32 {
        0b111
    }
}

#[derive(Debug)]
pub struct ProgramHeader {
    pub header_type: Option<ProgramHeaderType>,
    pub virtual_address: u32,
    pub padding: u32,
    pub memory_size: u32,
    pub flags: ProgramHeaderFlags,
    pub alignment: u32,
    pub data: Vec<u8>,
}

impl ProgramHeader {
    pub fn read<T: Read + Seek>(stream: &mut T) -> Result<ProgramHeader> {
        type Endian = LittleEndian;

        let raw_header_type = stream.read_u32::<Endian>()?;
        let header_type = FromPrimitive::from_u32(raw_header_type);

        let file_offset = stream.read_u32::<Endian>()?;
        let virtual_address = stream.read_u32::<Endian>()?;
        let padding = stream.read_u32::<Endian>()?;
        let file_size = stream.read_u32::<Endian>()?;
        let memory_size = stream.read_u32::<Endian>()?;
        let flags = stream.read_u32::<Endian>()?;
        let alignment = stream.read_u32::<Endian>()?;

        let mut data = vec![0; file_size as usize];
        stream.seek(Start(file_offset as u64))?;
        stream.read_exact(&mut data)?;

        let flags = ProgramHeaderFlags::from_bits(flags & ProgramHeaderFlags::known_mask())
            .unwrap_or(ProgramHeaderFlags::empty());

        Ok(ProgramHeader {
            header_type,
            virtual_address,
            padding,
            memory_size,
            flags,
            alignment,
            data,
        })
    }

    pub fn write<T: Write + Seek>(
        &self,
        stream: &mut T,
        landmark_index: usize,
    ) -> Result<Landmarks> {
        type Endian = LittleEndian;

        let mut landmarks = Landmarks::new();

        let raw_header_type = self
            .header_type
            .unwrap_or(Null)
            .to_u32()
            .ok_or(InvalidHeaderType)?;
        stream.write_u32::<Endian>(raw_header_type)?;

        landmarks.request(Bit32, Data(landmark_index), stream)?;
        stream.write_u32::<Endian>(0)?;

        stream.write_u32::<Endian>(self.virtual_address)?;
        stream.write_u32::<Endian>(self.padding)?;
        stream.write_u32::<Endian>(self.data.len() as u32)?;
        stream.write_u32::<Endian>(self.memory_size)?;
        stream.write_u32::<Endian>(self.flags.bits)?;
        stream.write_u32::<Endian>(self.alignment)?;

        Ok(landmarks)
    }
}
