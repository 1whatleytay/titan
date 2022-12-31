use std::collections::HashMap;
use std::io::{Seek, Write};
use std::io::SeekFrom::Start;
use byteorder::{LittleEndian, WriteBytesExt};

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Landmark {
    ProgramHeaderCount,
    ProgramHeaderStart,
    ProgramHeaderData(usize) // index
}

pub enum PointerSize {
    Bit8,
    Bit16,
    Bit32,
    Bit64
}

pub struct Landmarks {
    landmarks: HashMap<Landmark, u64>,
    requests: HashMap<u64, (PointerSize, Landmark)>
}

impl Landmarks {
    pub fn new() -> Landmarks {
        Landmarks {
            landmarks: HashMap::new(),
            requests: HashMap::new()
        }
    }

    pub fn request<T: Seek>(&mut self, size: PointerSize, landmark: Landmark, stream: &mut T)
        -> Result<(), std::io::Error> {
        self.requests.insert(stream.stream_position()?, (size, landmark));

        Ok(())
    }
    
    pub fn set(&mut self, landmark: Landmark, value: u64) {
        self.landmarks.insert(landmark, value);
    }

    pub fn mark<T: Seek>(&mut self, landmark: Landmark, stream: &mut T)
        -> Result<(), std::io::Error> {
        self.set(landmark, stream.stream_position()?);

        Ok(())
    }

    pub fn get(&mut self, landmark: Landmark) -> Option<u64> {
        self.landmarks.get(&landmark).cloned()
    }

    pub fn merge(&mut self, other: Landmarks) {
        for (key, value) in other.landmarks {
            self.landmarks.insert(key, value);
        }

        for (key, value) in other.requests {
            self.requests.insert(key, value);
        }
    }

    pub fn fill_requests<T: Write + Seek>(self, stream: &mut T) -> Result<(), std::io::Error> {
        for (position, (size, landmark)) in self.requests {
            let Some(value) = self.landmarks.get(&landmark).cloned() else { continue };

            stream.seek(Start(position))?;

            match size {
                PointerSize::Bit8 => stream.write_u8(value as u8)?,
                PointerSize::Bit16 => stream.write_u16::<LittleEndian>(value as u16)?,
                PointerSize::Bit32 => stream.write_u32::<LittleEndian>(value as u32)?,
                PointerSize::Bit64 => stream.write_u64::<LittleEndian>(value)?,
            }
        }

        Ok(())
    }
}
