use std::fmt::{Debug, Display, Formatter};
use crate::elf::error::Error::IoError;

#[derive(Debug)]
pub enum Error {
    InvalidMagic(u32),
    InvalidBinaryType,
    InvalidEndian,
    InvalidCPU,
    InvalidHeaderType,
    Requires32Bit,
    IoError(std::io::Error)
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        IoError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Error::InvalidMagic(magic) =>
                format!("Invalid ELF file (magic is 0x{:08x})", magic),
            Error::InvalidBinaryType => "Invalid binary type found".into(),
            Error::InvalidEndian => "Invalid endian type found".into(),
            Error::InvalidCPU => "Invalid CPU type found".into(),
            Error::Requires32Bit => "32-bit elf expected, but found other (64-bit ELF?)".into(),
            Error::InvalidHeaderType => "Invaid program header type found".into(),
            IoError(error) => format!("{}", error)
        })
    }
}

impl std::error::Error for Error { }

pub type Result<T> = std::result::Result<T, Error>;
