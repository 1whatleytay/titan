use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Instant;
use anyhow::anyhow;
use titan::cpu::memory::section::{DefaultResponder, SectionMemory};
use titan::elf::Elf;
use titan::execution::elf::inspection::Inspection;
use titan::mips::assembler::string::assemble_from_path;
use titan::mips::cpu::disassemble::MipsInspectionDisassembler;
use titan::mips::cpu::registers::registers::RawRegisters;
use titan::mips::cpu::State;
use titan::mips::execution::elf::setup::create_simple_state;
use titan::mips::execution::Executor;
use titan::mips::execution::trackers::empty::EmptyTracker;
use crate::arguments::{Command, MipsConfig};

pub fn run_mips(config: &MipsConfig) -> anyhow::Result<()> {
    if let Command::Disassemble { filename, emit } = &config.command {
        let path = Path::new(filename);
        
        let elf = Elf::read(&mut File::open(path)?)
            .map_err(|err| anyhow!("Failed to parse attached file, which must be an elf: {}", err))?;
        
        let name = path.file_name()
            .map(|x| x.to_string_lossy().to_string())
            .unwrap_or_else(|| filename.clone());
        
        let inspection = Inspection::new(Some(&name), &elf, &mut MipsInspectionDisassembler);

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
        _ => panic!("Unhandled command!")
    }

    Ok(())
}
