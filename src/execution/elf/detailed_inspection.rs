use crate::elf::program::{ProgramHeader, ProgramHeaderFlags};
use crate::elf::Elf;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashSet;
use std::io::Cursor;
use crate::unit::instruction::{InstructionDecoder, InstructionParameter};

pub struct InstructionInfo {
    pub pc: u32,
    pub instruction: u32,
    pub name: &'static str,
    pub parameters: Vec<InstructionParameter>
}

pub enum InspectionLine {
    Instruction(InstructionInfo),
    Blank,
    Comment(String),
    Label(String)
}

struct LabelManager {
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

    fn new(entry: Option<u32>) -> LabelManager {
        LabelManager {
            entry,
            labels: HashSet::new(),
        }
    }
}

fn disassemble(mut address: u32, data: &Vec<u8>, manager: &mut LabelManager) -> Vec<InstructionInfo> {
    let mut instructions = Cursor::new(data);

    let mut result = vec![];

    while let Ok(instruction) = instructions.read_u32::<LittleEndian>() {
        let inst = InstructionDecoder::decode(address, instruction);

        if let Some(inst) = inst {
            let name = inst.name();
            let parameters = inst.parameters();

            for parameter in &parameters {
                if let InstructionParameter::Address(value) = parameter {
                    manager.labels.insert(*value);
                }
            }

            result.push(InstructionInfo {
                pc: address,
                instruction,
                name,
                parameters
            })
        } else {
            result.push(InstructionInfo {
                pc: address,
                instruction,
                name: "INVALID",
                parameters: vec![]
            })
        }

        address += 4;
    }

    result
}

pub fn make_inspection_lines(elf: &Elf) -> Vec<InspectionLine> {
    let mut manager = LabelManager::new(Some(elf.header.program_entry));

    let mut lines: Vec<InspectionLine> = vec![];

    let executables: Vec<(&ProgramHeader, Vec<InstructionInfo>)> = elf
        .program_headers
        .iter()
        .filter(|header| header.flags.contains(ProgramHeaderFlags::EXECUTABLE))
        .map(|head| {
            (
                head,
                disassemble(head.virtual_address, &head.data, &mut manager),
            )
        })
        .collect();

    for (header, instructions) in executables {
        lines.append(&mut vec![
            InspectionLine::Blank,
            InspectionLine::Comment(format!(
                "# Section 0x{:08x}",
                header.virtual_address
            ))
        ]);

        for instruction in instructions {
            if manager.labels.contains(&instruction.pc) || manager.entry == Some(instruction.pc) {
                lines.push(InspectionLine::Label(manager.label_string(instruction.pc)));
            }

            lines.push(InspectionLine::Instruction(instruction));
        }
    }

    lines
}
