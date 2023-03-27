use crate::assembler::binary::Binary;
use crate::elf::header::{BinaryType, Endian, InstructionSet, MAGIC};
use crate::elf::program::ProgramHeaderType::Load;
use crate::elf::program::{ProgramHeader, ProgramHeaderFlags};
use crate::elf::{Elf, Header};

impl Binary {
    fn default_header(&self) -> Header {
        Header {
            magic: MAGIC,
            binary_type: BinaryType::Binary32,
            endian: Endian::Little,
            header_version: 1,
            abi: 0,
            padding: [0; 8],
            package: 0,
            cpu: InstructionSet::Mips,
            elf_version: 0,
            program_entry: self.entry,
        }
    }

    fn program_headers(&self) -> Vec<ProgramHeader> {
        let mut result = vec![];

        for region in &self.regions {
            let default_flags = ProgramHeaderFlags::READABLE | ProgramHeaderFlags::WRITABLE;

            let flags = if region.address == self.entry {
                default_flags | ProgramHeaderFlags::EXECUTABLE
            } else {
                default_flags
            };

            let header = ProgramHeader {
                header_type: Some(Load),
                virtual_address: region.address,
                padding: 0,
                memory_size: region.data.len() as u32,
                flags,
                alignment: 1,
                data: region.data.clone(),
            };

            result.push(header);
        }

        result
    }
}

impl From<Binary> for Elf {
    fn from(val: Binary) -> Self {
        let header = val.default_header();
        let program_headers = val.program_headers();

        Elf {
            header,
            program_headers,
        }
    }
}
