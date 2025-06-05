use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;
use titan::cpu::memory::section::{DefaultResponder, SectionMemory};
use titan::elf::Elf;
use titan::mips::assembler::string::assemble_from_path;
use titan::mips::cpu::registers::registers::RawRegisters;
use titan::mips::cpu::State;
use titan::mips::execution::elf::setup::create_simple_state;
use titan::mips::execution::Executor;
use titan::mips::execution::trackers::empty::EmptyTracker;
use crate::arguments::{Command, MipsConfig};

pub fn run_mips(config: &MipsConfig) -> anyhow::Result<()> {
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
            let elf: Elf = binary.create_elf();

            let instant = Instant::now();

            let state: State<SectionMemory<DefaultResponder>, RawRegisters> =
                create_simple_state(&elf, 0x100000);
            let debugger = Executor::new(state, EmptyTracker {});

            let frame = debugger.run(false);

            let end = instant.elapsed();

            println!(
                "Running finished in {}ms with mode: {:?}.",
                end.as_millis(),
                frame.mode
            );
        }
    }

    Ok(())
}
