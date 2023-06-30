use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;
use clap::{Parser, Subcommand};
use titan::elf::Elf;

use anyhow::Result;
use titan::assembler::string::assemble_from_path;
use titan::cpu::memory::section::{DefaultResponder, SectionMemory};
use titan::cpu::State;
use titan::debug::Debugger;
use titan::debug::elf::setup::create_simple_state;
use titan::debug::trackers::empty::EmptyTracker;

#[derive(Subcommand, Debug)]
enum Command {
    Build { filename: String },
    Run { filename: String },
    Test { filename: String }
}

impl Command {
    fn filename(&self) -> &str {
        match self {
            Command::Build { filename } => filename,
            Command::Run { filename } => filename,
            Command::Test { filename } => filename,
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Command,

    #[arg(short, long)]
    emit: Option<String>
}

fn run(args: Args) -> Result<()> {
    let filename = args.command.filename();
    println!("Building {}...", filename);

    let text = fs::read_to_string(filename)?;
    let binary = assemble_from_path(text, PathBuf::from(filename))?;

    println!("Binary built!");

    if let Some(emit) = args.emit {
        let elf: Elf = binary.create_elf();

        let mut file = File::create(emit)?;

        elf.write(&mut file)?;
    }

    match args.command {
        Command::Build { filename: _ } => {}
        Command::Run { filename: _ } | Command::Test { filename: _ } => {
            let elf: Elf = binary.create_elf();

            let instant = Instant::now();

            let state: State<SectionMemory<DefaultResponder>> = create_simple_state(&elf, 0x100000);
            let debugger = Debugger::new(state, EmptyTracker { });

            let frame = debugger.run();

            let end = instant.elapsed();

            println!("Running finished in {}ms with mode: {:?}.", end.as_millis(), frame.mode);
        }
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    run(args).unwrap()
}
