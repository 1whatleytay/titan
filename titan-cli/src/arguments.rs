use clap::{Parser, Subcommand};

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    Build {
        filename: String,

        #[arg(short, long)]
        emit: Option<String>,
    },
    Run {
        filename: String,

        #[arg(short, long)]
        emit: Option<String>,
    },
    Test {
        filename: String,

        #[arg(short, long)]
        emit: Option<String>,
    },
    Disassemble {
        filename: String,

        #[arg(short, long)]
        emit: Option<String>,
    }
}

impl Command {
    pub fn filename(&self) -> &str {
        match self {
            Command::Build { filename, .. } => filename,
            Command::Run { filename, .. } => filename,
            Command::Test { filename, .. } => filename,
            Command::Disassemble { filename, .. } => filename,
        }
    }

    pub fn emit(&self) -> Option<&str> {
        match self {
            Command::Build { emit, .. } => emit.as_ref().map(|x| x.as_str()),
            Command::Run { emit, .. } => emit.as_ref().map(|x| x.as_str()),
            Command::Test { emit, .. } => emit.as_ref().map(|x| x.as_str()),
            Command::Disassemble { emit, .. } => emit.as_ref().map(|x| x.as_str()),
        }
    }
}

#[derive(Parser, Debug)]
pub struct MipsConfig {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub struct RiscVConfig {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Platform {
    Mips(MipsConfig),
    RiscV(RiscVConfig),
}

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub platform: Platform,
}