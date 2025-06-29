use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MemoryAlignment {
    Half,
    Word,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    MemoryAlign(MemoryAlignment, u32),
    MemoryUnmapped(u32),
    CpuInvalid(u32),
    CpuTrap,
    // Intended to be caught by higher level. Parameter is instruction size (# of bytes to skip PC when handled).
    CpuSyscall(u32),
    CpuBreak, // Intended for breakpoint work.
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MemoryAlign(alignment, address) => {
                let align = match alignment {
                    MemoryAlignment::Half => 2,
                    MemoryAlignment::Word => 4,
                };

                write!(f, "Address 0x{address:08x} is not aligned for this instruction (ensure it is a multiple of {align}).")
            }
            Error::MemoryUnmapped(address) => {
                write!(
                    f,
                    "Memory access for address 0x{address:08x} is prohibited (unmapped memory)."
                )
            }
            Error::CpuInvalid(instruction) => {
                write!(f, "Invalid CPU instruction 0x{instruction:08x}")
            }
            Error::CpuTrap => write!(
                f,
                "The instruction was given invalid parameters (CPU Trap was thrown)."
            ),
            Error::CpuSyscall(bytes) => write!(f, "CPU Syscall was not handled ({bytes} instruction)"),
            Error::CpuBreak => write!(f, "CPU Breakport was not handled"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
