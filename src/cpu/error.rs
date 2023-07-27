use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    MemoryAlign(u32),
    MemoryUnmapped(u32),
    CpuInvalid(u32),
    CpuTrap,
    CpuSyscall, // Intended to be caught by higher level.
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MemoryAlign(address) => {
                write!(f, "Memory access for address 0x{address:08x} is prohibited (not aligned for this instruction, ensure its a multiple of 2 or 4).")
            }
            Error::MemoryUnmapped(address) => {
                write!(f, "Memory access for address 0x{address:08x} is prohibited (unmapped memory).")
            }
            Error::CpuInvalid(instruction) => {
                write!(f, "Invalid CPU instruction 0x{instruction:08x}")
            }
            Error::CpuTrap => write!(f, "The instruction was given invalid parameters (CPU Trap was thrown)."),
            Error::CpuSyscall => write!(f, "CPU Syscall was not handled"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
