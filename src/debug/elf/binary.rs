use crate::elf::{Elf, Header};
use crate::assembler::binary::Binary;
use crate::elf::header::{BinaryType, Endian, InstructionSet, MAGIC};
use crate::elf::program::{ProgramHeader, ProgramHeaderFlags};
use crate::elf::program::ProgramHeaderType::Load;

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
                flags: ProgramHeaderFlags::READABLE
                    | ProgramHeaderFlags::WRITABLE
                    | ProgramHeaderFlags::EXECUTABLE,
                alignment: 1,
                data: region.data.clone(),
            };

            result.push(header);
        }

        result
    }
}

impl Into<Elf> for Binary {
    fn into(self) -> Elf {
        let header = self.default_header();
        let program_headers = self.program_headers();

        Elf { header, program_headers }
    }
}
