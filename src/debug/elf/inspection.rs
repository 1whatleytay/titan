use std::collections::HashMap;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::cpu::decoder::Decoder;
use crate::cpu::disassemble::Disassembler;
use crate::elf::Elf;
use crate::elf::header::{BinaryType, Endian};
use crate::elf::program::{ProgramHeader, ProgramHeaderFlags, ProgramHeaderType};

pub struct Inspection {
    pub breakpoints: HashMap<usize, u32>, // line to address
    pub lines: Vec<String>
}

impl Inspection {
    fn program_header_flags(flags: ProgramHeaderFlags) -> String {
        let entries = vec![
            (ProgramHeaderFlags::READABLE, "R"),
            (ProgramHeaderFlags::WRITABLE, "W"),
            (ProgramHeaderFlags::EXECUTABLE, "E")
        ];

        entries.iter()
            .map(|(key, value)| {
                if flags.contains(*key) { value } else { "-" }
            })
            .fold("".to_string(), |a, b| format!("{}{}", a, b))
    }

    fn program_header_type(header_type: &Option<ProgramHeaderType>) -> String {
        header_type
            .map_or("Unknown".into(), |value| format!("{:?}", value))
    }

    fn program_header(header: &ProgramHeader) -> String {
        format!(
            "{} (0x{:08x} - 0x{:08x}, size: {}, flags: {})",
            Inspection::program_header_type(&header.header_type),
            header.virtual_address,
            header.virtual_address + header.memory_size,
            header.memory_size,
            Inspection::program_header_flags(header.flags)
        )
    }

    fn header(elf: &Elf) -> Vec<String> {
        let binary_string = match elf.header.binary_type {
            BinaryType::Binary32 => "32-bit",
            BinaryType::Binary64 => "64-bit"
        };

        let endian_string = match elf.header.endian {
            Endian::Little => "le",
            Endian::Big => "be"
        };

        vec![
            format!("Version: {} (header: {})", elf.header.elf_version, elf.header.header_version),
            format!("CPU: {:?} ({}, {})", elf.header.cpu, binary_string, endian_string),
            format!("ABI: {}", elf.header.abi),
            format!("Entry Point: 0x{:08x}", elf.header.program_entry),
        ]
    }

    fn description(named: Option<&str>, elf: &Elf) -> Vec<String> {
        let mut result = named
            .map_or(vec![], |value| vec![format!("File: {}", value)]);

        result.append(&mut Inspection::header(elf));

        if !elf.program_headers.is_empty() {
            result.append(&mut vec![
                "".into(),
                format!("Program Headers (count: {})", elf.program_headers.len())
            ]);

            let mut headers: Vec<String> = elf.program_headers.iter()
                .map(|header| Inspection::program_header(header))
                .map(|text| format!("  {}", text))
                .collect();

            result.append(&mut headers);
        }

        result
    }

    // Assumption: Every instruction is the same size.
    fn disassemble(address: u32, data: &Vec<u8>) -> Vec<String> {
        let mut instructions = Cursor::new(data);
        let mut pc = address;

        let mut result = vec![];

        while let Ok(instruction) = instructions.read_u32::<LittleEndian>() {
            let text = Disassembler { pc }.dispatch(instruction)
                .unwrap_or_else(|| "INVALID".into());

            pc += 4;

            result.push(text)
        }

        result
    }

    pub fn new(named: Option<&str>, elf: &Elf) -> Inspection {
        let mut lines: Vec<String> = Inspection::description(named, elf).iter()
            .map(|text| format!("# {}", text))
            .collect();

        let mut breakpoints = HashMap::new();

        let executables = elf.program_headers.iter()
            .filter(|header| header.flags.contains(ProgramHeaderFlags::EXECUTABLE));

        for executable in executables {
            lines.append(&mut vec![
                "".into(),
                format!(
                    "# Program (0x{:08x}, {})",
                    executable.virtual_address,
                    Inspection::program_header_type(&executable.header_type)
                )
            ]);

            let start = lines.len();
            let raw = Inspection::disassemble(executable.virtual_address, &executable.data);
            let mut instructions = raw.iter()
                .map(|line| format!("    {}", line))
                .collect::<Vec<String>>();

            for i in 0 .. instructions.len() {
                breakpoints.insert(i + start, executable.virtual_address + (i as u32 * 4u32));
            }

            lines.append(&mut instructions)
        }

        Inspection { breakpoints, lines }
    }
}
