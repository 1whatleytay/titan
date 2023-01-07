use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    MemoryAlign(u32),
    MemoryUnmapped(u32),
    MemoryBoundary(u32),
    CpuInvalid(u32),
    CpuTrap,
    CpuSyscall, // Intended to be caught by higher level.
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MemoryAlign(address) =>
                write!(f, "Invalid memory alignment for address 0x{:08x}", address),
            Error::MemoryUnmapped(address) =>
                write!(f, "Unmapped memory region for address 0x{:08x}", address),
            Error::MemoryBoundary(address) =>
                write!(f, "Invalid memory access across region boundary at 0x{:08x}", address),
            Error::CpuInvalid(instruction) =>
                write!(f, "Invalid CPU instruction 0x{:08x}", instruction),
            Error::CpuTrap =>
                write!(f, "CPU Trap was thrown"),
            Error::CpuSyscall =>
                write!(f, "CPU Syscall was not handled"),
        }
    }
}

impl std::error::Error for Error { }

pub type Result<T> = std::result::Result<T, Error>;
