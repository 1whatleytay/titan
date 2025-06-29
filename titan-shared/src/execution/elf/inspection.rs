use crate::cpu::disassemble::LabelProvider;
use crate::elf::Elf;
use crate::elf::header::{BinaryType, Endian};
use crate::elf::program::{ProgramHeader, ProgramHeaderFlags, ProgramHeaderType};
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::{HashMap, HashSet};
use std::io::Cursor;

pub struct LabelManager {
    entry: Option<u32>,
    labels: HashSet<u32>,
}

impl LabelManager {
    fn label_string(&self, address: u32) -> String {
        if Some(address) == self.entry {
            format!("entry_{address:x}")
        } else {
            format!("address_{address:x}")
        }
    }

    pub fn new(entry: Option<u32>) -> LabelManager {
        LabelManager {
            entry,
            labels: HashSet::new(),
        }
    }
}

impl LabelProvider for LabelManager {
    fn label_for(&mut self, address: u32) -> String {
        self.labels.insert(address);

        self.label_string(address)
    }
}

pub struct Inspection {
    pub breakpoints: HashMap<u32, usize>, // pc -> line
    pub lines: Vec<String>,
}

pub struct InspectionDisassemblerResult {
    pub line: String,
    pub next_pc: u32,
}

pub enum InspectionReadStrategy {
    ReadU32Only,   // MIPS
    ReadU32PadU16, // RISC-V with C-Extension
}

pub trait InspectionDisassembler {
    fn read_strategy() -> InspectionReadStrategy {
        InspectionReadStrategy::ReadU32Only
    }

    fn disassemble(
        &mut self,
        pc: u32,
        instruction: u32,
        labels: &mut impl LabelProvider,
    ) -> Option<InspectionDisassemblerResult>;
}

impl Inspection {
    fn program_header_flags(flags: ProgramHeaderFlags) -> String {
        let entries = [
            (ProgramHeaderFlags::READABLE, "R"),
            (ProgramHeaderFlags::WRITABLE, "W"),
            (ProgramHeaderFlags::EXECUTABLE, "E"),
        ];

        entries
            .iter()
            .map(
                |(key, value)| {
                    if flags.contains(*key) { value } else { "-" }
                },
            )
            .fold("".to_string(), |a, b| format!("{a}{b}"))
    }

    fn program_header_type(header_type: &Option<ProgramHeaderType>) -> String {
        header_type.map_or("Unknown".into(), |value| format!("{value:?}"))
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
            BinaryType::Binary64 => "64-bit",
        };

        let endian_string = match elf.header.endian {
            Endian::Little => "le",
            Endian::Big => "be",
        };

        vec![
            format!(
                "Version: {} (header: {})",
                elf.header.elf_version, elf.header.header_version
            ),
            format!(
                "CPU: {:?} ({}, {})",
                elf.header.cpu, binary_string, endian_string
            ),
            format!("ABI: {}", elf.header.abi),
            format!("Entry Point: 0x{:08x}", elf.header.program_entry),
        ]
    }

    fn description(named: Option<&str>, elf: &Elf) -> Vec<String> {
        let mut result = named.map_or(vec![], |value| vec![format!("File: {value}")]);

        result.append(&mut Inspection::header(elf));

        if !elf.program_headers.is_empty() {
            result.append(&mut vec![
                "".into(),
                format!("Program Headers (count: {})", elf.program_headers.len()),
            ]);

            let mut headers: Vec<String> = elf
                .program_headers
                .iter()
                .map(Inspection::program_header)
                .map(|text| format!("  {text}"))
                .collect();

            result.append(&mut headers);
        }

        result
    }

    // Assumption: Every instruction is the same size.
    fn disassemble<T: InspectionDisassembler>(
        address: u32,
        data: &Vec<u8>,
        disassembler: &mut T,
        labels: &mut impl LabelProvider,
    ) -> Vec<(String, u32)> {
        let mut result = vec![];

        let mut pc = address;

        while let Some(instruction) = {
            let index = pc - address;

            match T::read_strategy() {
                InspectionReadStrategy::ReadU32Only => {
                    if index as usize + 4 <= data.len() {
                        Cursor::new(&data[index as usize..])
                            .read_u32::<LittleEndian>()
                            .ok()
                    } else {
                        None
                    }
                }
                InspectionReadStrategy::ReadU32PadU16 => {
                    if index as usize + 4 <= data.len() {
                        Cursor::new(&data[index as usize..])
                            .read_u32::<LittleEndian>()
                            .ok()
                    } else if index as usize + 2 <= data.len() {
                        Cursor::new(&data[index as usize..])
                            .read_u16::<LittleEndian>()
                            .map(|x| x as u32)
                            .ok()
                    } else {
                        None
                    }
                }
            }
        } {
            let line = disassembler.disassemble(pc, instruction, labels);

            let next_pc = line.as_ref().map(|result| result.next_pc).unwrap_or(pc + 4);

            let text = line
                .map(|result| result.line)
                .unwrap_or_else(|| format!("INVALID # 0x{instruction:08x}"));

            result.push((text, pc));

            pc = next_pc;
        }

        result
    }

    pub fn new(
        named: Option<&str>,
        elf: &Elf,
        disassembler: &mut impl InspectionDisassembler,
    ) -> Inspection {
        let mut lines: Vec<String> = Inspection::description(named, elf)
            .iter()
            .map(|text| format!("# {text}"))
            .collect();

        let mut breakpoints = HashMap::new();

        let mut manager = LabelManager::new(Some(elf.header.program_entry));

        let executables: Vec<(&ProgramHeader, Vec<(String, u32)>)> = elf
            .program_headers
            .iter()
            .filter(|header| header.flags.contains(ProgramHeaderFlags::EXECUTABLE))
            .map(|head| {
                (
                    head,
                    Inspection::disassemble(
                        head.virtual_address,
                        &head.data,
                        disassembler,
                        &mut manager,
                    ),
                )
            })
            .collect();

        for (header, instructions) in executables {
            lines.append(&mut vec![
                "".into(),
                format!(
                    "# Section (0x{:08x}, {})",
                    header.virtual_address,
                    Inspection::program_header_type(&header.header_type)
                ),
            ]);

            for (instruction, pc) in instructions {
                if manager.labels.contains(&pc) || manager.entry == Some(pc) {
                    lines.push(format!("{}:", manager.label_string(pc)));
                }

                breakpoints.insert(pc, lines.len());

                lines.push(format!("    {instruction}"));
            }
        }

        Inspection { breakpoints, lines }
    }
}
