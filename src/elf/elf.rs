use std::io::{Read, Write, Seek};
use std::io::SeekFrom::Start;
use crate::elf::Header;
use crate::elf::program::ProgramHeader;
use crate::elf::error::Result;
use crate::elf::header::HeaderDetails;
use crate::elf::landmark::Landmark::{ProgramHeaderCount, ProgramHeaderData, ProgramHeaderStart};
use crate::elf::landmark::Landmarks;

#[derive(Debug)]
pub struct Elf {
    pub header: Header,
    pub program_headers: Vec<ProgramHeader>
}

impl Elf {
    pub fn read<T: Read + Seek>(stream: &mut T) -> Result<Elf> {
        let (header, details) = Header::read(stream)?;

        let mut start_index = details.program_table_position as u64;
        let mut program_headers: Vec<ProgramHeader> = vec![];

        for _ in 0 .. details.program_entry_count {
            stream.seek(Start(start_index))?;

            if let Ok(header) = ProgramHeader::read(stream) {
                program_headers.push(header)
            }

            start_index += details.program_entry_size as u64;
        }

        Ok(Elf { header, program_headers })
    }

    pub fn write<T: Write + Seek>(&self, stream: &mut T) -> Result<()> {
        let mut landmarks = Landmarks::new();

        landmarks.set(ProgramHeaderCount, self.program_headers.len() as u64);

        self.header.write(stream)?;
        landmarks.merge(HeaderDetails::write_landmarks(stream)?);

        landmarks.mark(ProgramHeaderStart, stream)?;
        for (index, header) in self.program_headers.iter().enumerate() {
            landmarks.merge(header.write(stream, index)?);
        }

        for (index, header) in self.program_headers.iter().enumerate() {
            landmarks.mark(ProgramHeaderData(index), stream)?;

            stream.write(&header.data[..])?;
        }

        landmarks.fill_requests(stream)?;

        Ok(())
    }
}
