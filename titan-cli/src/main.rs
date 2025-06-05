mod arguments;
mod mips;
mod risc_v;

use clap::Parser;
use crate::arguments::{Args, Platform};

fn main() {
    let args = Args::parse();
    
    match &args.platform {
        Platform::Mips(config) => mips::run_mips(config).unwrap(),
        Platform::RiscV(config) => risc_v::run_risc_v(config).unwrap(),
    }
}
