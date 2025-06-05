use std::fs;
use std::fs::File;
use std::path::PathBuf;
use anyhow::anyhow;
use titan::elf::Elf;
use titan::riscv::assembler::string::assemble_from_path;
use crate::arguments::{Command, RiscVConfig};

pub fn run_risc_v(config: &RiscVConfig) -> anyhow::Result<()> {
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
    }

    Ok(())
}
