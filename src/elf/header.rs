use std::io::{Read, Seek, Write};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use crate::elf::error::Error::{
    InvalidBinaryType, InvalidEndian, InvalidMagic, InvalidCPU, Requires32Bit
};
use crate::elf::error::Result;
use crate::elf::landmark::Landmark::{ProgramHeaderStart, ProgramHeaderCount};
use crate::elf::landmark::Landmarks;
use crate::elf::landmark::PointerSize::{Bit16, Bit32};

#[derive(FromPrimitive, ToPrimitive, PartialEq, Debug)]
pub enum BinaryType {
    Binary32 = 1,
    Binary64 = 2
}

#[derive(FromPrimitive, ToPrimitive, PartialEq, Debug)]
pub enum Endian {
    Little = 1,
    Big = 2
}

#[derive(FromPrimitive, ToPrimitive, PartialEq, Debug)]
pub enum InstructionSet {
    Generic = 0x00,
    Sparc = 0x02,
    X86 = 0x03,
    Mips = 0x08,
    PowerPC = 0x14,
    Arm = 0x28,
    SuperH = 0x2A,
    IA64 = 0x32,
    X64 = 0x3E,
    AArch64 = 0xB7,
    RiscV = 0xF3
}

#[derive(Debug)]
pub struct Header {
    pub magic: u32,
    pub binary_type: BinaryType,
    pub endian: Endian,
    pub header_version: u8,
    pub abi: u8,
    pub padding: [u8; 8],
    pub package: u16,
    pub cpu: InstructionSet,
    pub elf_version: u32,
    pub program_entry: u32
}

#[derive(Debug)]
pub struct HeaderDetails {
    pub program_table_position: u32,
    pub section_table_point: u32,
    pub flags: u32,
    pub header_size: u16,
    pub program_entry_size: u16,
    pub program_entry_count: u16,
    pub section_entry_size: u16,
    pub section_entry_count: u16,
    pub names_point: u16,
}

pub const MAGIC: u32 = 0x464c457f;

impl Header {
    pub fn read<T: Read>(stream: &mut T) -> Result<(Header, HeaderDetails)> {
        type Endian = LittleEndian;

        let header = Header {
            magic: stream.read_u32::<Endian>()?,
            binary_type: FromPrimitive::from_u8(stream.read_u8()?).ok_or(InvalidBinaryType)?,
            endian: FromPrimitive::from_u8(stream.read_u8()?).ok_or(InvalidEndian)?,
            header_version: stream.read_u8()?,
            abi: stream.read_u8()?,
            padding: {
                let mut buffer = [0; 8];

                stream.read_exact(&mut buffer)?;
                buffer
            },
            package: stream.read_u16::<Endian>()?,
            cpu: FromPrimitive::from_u16(stream.read_u16::<Endian>()?).ok_or(InvalidCPU)?,
            elf_version: stream.read_u32::<Endian>()?,
            program_entry: stream.read_u32::<Endian>()?,
        };

        if header.magic != MAGIC {
            Err(InvalidMagic(header.magic))
        } else if header.binary_type != BinaryType::Binary32 {
            Err(Requires32Bit)
        } else {
            Ok((header, HeaderDetails::read(stream)?))
        }
    }

    pub fn write<T: Write + Seek>(&self, stream: &mut T) -> Result<()> {
        type Endian = LittleEndian;

        stream.write_u32::<Endian>(MAGIC)?;
        stream.write_u8(self.binary_type.to_u8().ok_or(InvalidBinaryType)?)?;
        stream.write_u8(self.endian.to_u8().ok_or(InvalidBinaryType)?)?;
        stream.write_u8(self.header_version)?;
        stream.write_u8(self.abi)?;
        stream.write(&self.padding)?;
        stream.write_u16::<Endian>(self.package)?;
        stream.write_u16::<Endian>(self.cpu.to_u16().ok_or(InvalidCPU)?)?;
        stream.write_u32::<Endian>(self.elf_version)?;
        stream.write_u32::<Endian>(self.program_entry)?;

        Ok(())
    }
}

const HEADER_SIZE: u16 = 52;
const PROGRAM_HEADER_SIZE: u16 = 32;

impl HeaderDetails {
    pub fn read<T: Read>(stream: &mut T) -> Result<HeaderDetails> {
        type Endian = LittleEndian;

        let details = HeaderDetails {
            program_table_position: stream.read_u32::<Endian>()?,
            section_table_point: stream.read_u32::<Endian>()?,
            flags: stream.read_u32::<Endian>()?,
            header_size: stream.read_u16::<Endian>()?,
            program_entry_size: stream.read_u16::<Endian>()?,
            program_entry_count: stream.read_u16::<Endian>()?,
            section_entry_size: stream.read_u16::<Endian>()?,
            section_entry_count: stream.read_u16::<Endian>()?,
            names_point: stream.read_u16::<Endian>()?,
        };

        Ok(details)
    }

    pub fn write_landmarks<T: Write + Seek>(stream: &mut T) -> Result<Landmarks> {
        type Endian = LittleEndian;

        let mut landmarks = Landmarks::new();

        landmarks.request(Bit32, ProgramHeaderStart, stream)?;
        stream.write_u32::<Endian>(0)?; // program_table_position:
        stream.write_u32::<Endian>(0)?; // section_table_point:
        stream.write_u32::<Endian>(0)?; // flags:
        stream.write_u16::<Endian>(HEADER_SIZE)?; // header_size:
        stream.write_u16::<Endian>(PROGRAM_HEADER_SIZE)?; // program_entry_size:
        landmarks.request(Bit16, ProgramHeaderCount, stream)?;
        stream.write_u16::<Endian>(0)?; // program_entry_count:
        stream.write_u16::<Endian>(0)?; // section_entry_size:
        stream.write_u16::<Endian>(0)?; // section_entry_count:
        stream.write_u16::<Endian>(0)?; // names_point:

        Ok(landmarks)
    }
}