use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use anyhow::anyhow;
use titan::elf::Elf;
use titan::execution::elf::inspection::Inspection;
use titan::riscv::assembler::string::assemble_from_path;
use titan::riscv::cpu::disassemble::RiscVInspectionDisassembler;
use crate::arguments::{Command, RiscVConfig};

pub fn run_risc_v(config: &RiscVConfig) -> anyhow::Result<()> {
    if let Command::Disassemble { filename, emit } = &config.command {
        let path = Path::new(filename);

        let elf = Elf::read(&mut File::open(path)?)
            .map_err(|err| anyhow!("Failed to parse attached file, which must be an elf: {}", err))?;

        let name = path.file_name()
            .map(|x| x.to_string_lossy().to_string())
            .unwrap_or_else(|| filename.clone());

        let inspection = Inspection::new(Some(&name), &elf, &mut RiscVInspectionDisassembler);

        let result = inspection.lines.join("\n");

        if let Some(emit) = emit {
            fs::write(emit, result)?;

            println!("Written disassembly to {emit}");
        } else {
            println!("{result}\n");
        }

        return Ok(())
    }
    
    let filename = config.command.filename();
    println!("Building {}...", filename);

    let text = fs::read_to_string(filename)?;
    let binary = assemble_from_path(text, PathBuf::from(filename))?;

    println!("Binary built!");

    if let Some(emit) = &config.command.emit() {
        let elf: Elf = binary.create_elf();

        let mut file = File::create(emit)?;

        elf.write(&mut file)?;
    }

    match config.command {
        Command::Build { .. } => {}
        Command::Run { .. } | Command::Test { .. } => {
            return Err(anyhow!("RISC-V does not yet support running binaries."))
        }
        _ => panic!("Unhandled command!")
    }

    Ok(())
}
