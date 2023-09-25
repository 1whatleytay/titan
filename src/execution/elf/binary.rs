use crate::assembler::binary::{Binary, RegionFlags};
use crate::elf::header::{BinaryType, Endian, InstructionSet, MAGIC};
use crate::elf::program::ProgramHeaderType::Load;
use crate::elf::program::{ProgramHeader, ProgramHeaderFlags};
use crate::elf::{Elf, Header};

impl From<RegionFlags> for ProgramHeaderFlags {
    fn from(value: RegionFlags) -> Self {
        value.iter()
            .map(|item| match item {
                RegionFlags::EXECUTABLE => ProgramHeaderFlags::EXECUTABLE,
                RegionFlags::READABLE => ProgramHeaderFlags::READABLE,
                RegionFlags::WRITABLE => ProgramHeaderFlags::WRITABLE,
                _ => ProgramHeaderFlags::empty(),
            })
            .reduce(|x, y| x | y)
            .unwrap_or(ProgramHeaderFlags::empty())
    }
}

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
            let header = ProgramHeader {
                header_type: Some(Load),
                virtual_address: region.address,
                padding: 0,
                memory_size: region.data.len() as u32,
                flags: region.flags.into(),
                alignment: 1,
                data: region.data.clone(),
            };

            result.push(header);
        }

        result
    }

    pub fn create_elf(&self) -> Elf {
        let header = self.default_header();
        let program_headers = self.program_headers();

        Elf {
            header,
            program_headers,
        }
    }
}
