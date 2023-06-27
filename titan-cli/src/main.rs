use std::fs;
use std::fs::File;
use std::time::Instant;
use clap::Parser;
use titan::assembler::string::assemble_from;
use titan::elf::Elf;

use anyhow::Result;
use titan::cpu::memory::section::{DefaultResponder, SectionMemory};
use titan::cpu::State;
use titan::debug::Debugger;
use titan::debug::elf::setup::create_simple_state;
use titan::debug::trackers::empty::EmptyTracker;

#[derive(Parser, Debug)]
struct Args {
    filename: String,

    #[arg(short, long)]
    emit: Option<String>,

    #[arg(short, long)]
    run: bool
}

fn run(args: Args) -> Result<()> {
    println!("Building {}...", args.filename);

    let text = fs::read_to_string(args.filename)?;
    let binary = assemble_from(&text)?;

    println!("Binary built!");

    if let Some(emit) = args.emit {
        let elf: Elf = binary.into();

        let mut file = File::create(emit)?;

        elf.write(&mut file)?;
    } else if args.run {
        let elf: Elf = binary.into();

        let instant = Instant::now();

        let state: State<SectionMemory<DefaultResponder>> = create_simple_state(&elf, 0x100000);
        let debugger = Debugger::new(state, EmptyTracker { });

        let frame = debugger.run();

        let end = instant.elapsed();

        println!("Running finished in {}ms with mode: {:?}.", end.as_millis(), frame.mode);
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    run(args).unwrap()
}
