use std::collections::HashMap;
use crate::assembler::instructions::Encoding::{Register, Source, Destination, Inputs, Sham, SpecialBranch, Immediate, Jump, Branch, LoadImmediate, Parameterless, LoadOffset, StoreOffset};

pub enum Encoding {
    Register(u8), // $, $, $
    Source(u8), // $
    Destination(u8), // $
    Inputs(u8), // $, $
    Sham(u8), // $, $, sham
    SpecialBranch(u8),
    Immediate, // $, $, I
    LoadImmediate,
    Jump, // I or Label
    Branch, // I or Label
    Parameterless,
    LoadOffset,
    StoreOffset,
}

pub struct Instruction<'a> {
    name: &'a str,
    opcode: u8,
    encoding: Encoding,
}

pub const INSTRUCTIONS: [Instruction; 56] = [
    Instruction { name: "sll", opcode: 0, encoding: Sham(0) },
    Instruction { name: "srl", opcode: 0, encoding: Sham(2) },
    Instruction { name: "sra", opcode: 0, encoding: Sham(3) },
    Instruction { name: "sllv", opcode: 0, encoding: Register(4) },
    Instruction { name: "srlv", opcode: 0, encoding: Register(6) },
    Instruction { name: "srav", opcode: 0, encoding: Register(7) },
    Instruction { name: "jr", opcode: 0, encoding: Source(8) },
    Instruction { name: "jalr", opcode: 0, encoding: Source(9) },
    Instruction { name: "mfhi", opcode: 0, encoding: Destination(16) },
    Instruction { name: "mthi", opcode: 0, encoding: Source(17) },
    Instruction { name: "mflo", opcode: 0, encoding: Destination(18) },
    Instruction { name: "mtlo", opcode: 0, encoding: Source(19) },
    Instruction { name: "mult", opcode: 0, encoding: Inputs(24) },
    Instruction { name: "multu", opcode: 0, encoding: Inputs(25) },
    Instruction { name: "div", opcode: 0, encoding: Inputs(26) },
    Instruction { name: "divu", opcode: 0, encoding: Inputs(27) },
    Instruction { name: "add", opcode: 0, encoding: Register(32) },
    Instruction { name: "addu", opcode: 0, encoding: Register(33) },
    Instruction { name: "sub", opcode: 0, encoding: Register(34) },
    Instruction { name: "subu", opcode: 0, encoding: Register(35) },
    Instruction { name: "and", opcode: 0, encoding: Register(36) },
    Instruction { name: "or", opcode: 0, encoding: Register(37) },
    Instruction { name: "xor", opcode: 0, encoding: Register(38) },
    Instruction { name: "nor", opcode: 0, encoding: Register(39) },
    Instruction { name: "sltu", opcode: 0, encoding: Register(41) },
    Instruction { name: "slt", opcode: 0, encoding: Register(42) },
    Instruction { name: "bltz", opcode: 1, encoding: SpecialBranch(0) },
    Instruction { name: "bgez", opcode: 1, encoding: SpecialBranch(1) },
    Instruction { name: "bltzal", opcode: 1, encoding: SpecialBranch(16) },
    Instruction { name: "bgezal", opcode: 1, encoding: SpecialBranch(17) },
    Instruction { name: "j", opcode: 2, encoding: Jump },
    Instruction { name: "jal", opcode: 3, encoding: Jump },
    Instruction { name: "beq", opcode: 4, encoding: Branch },
    Instruction { name: "bne", opcode: 5, encoding: Branch },
    Instruction { name: "blez", opcode: 6, encoding: Branch },
    Instruction { name: "bgtz", opcode: 7, encoding: Branch },
    Instruction { name: "addi", opcode: 8, encoding: Immediate },
    Instruction { name: "addiu", opcode: 9, encoding: Immediate },
    Instruction { name: "slti", opcode: 10, encoding: Immediate },
    Instruction { name: "sltiu", opcode: 11, encoding: Immediate },
    Instruction { name: "andi", opcode: 12, encoding: Immediate },
    Instruction { name: "ori", opcode: 13, encoding: Immediate },
    Instruction { name: "xori", opcode: 14, encoding: Immediate },
    Instruction { name: "lui", opcode: 15, encoding: LoadImmediate },
    Instruction { name: "llo", opcode: 24, encoding: LoadImmediate },
    Instruction { name: "lhi", opcode: 25, encoding: LoadImmediate },
    Instruction { name: "trap", opcode: 26, encoding: Parameterless },
    Instruction { name: "syscall", opcode: 26, encoding: Parameterless },
    Instruction { name: "lb", opcode: 32, encoding: LoadOffset },
    Instruction { name: "lh", opcode: 33, encoding: LoadOffset },
    Instruction { name: "lw", opcode: 35, encoding: LoadOffset },
    Instruction { name: "lbu", opcode: 36, encoding: LoadOffset },
    Instruction { name: "lhu", opcode: 37, encoding: LoadOffset },
    Instruction { name: "sb", opcode: 40, encoding: StoreOffset },
    Instruction { name: "sh", opcode: 41, encoding: StoreOffset },
    Instruction { name: "sw", opcode: 43, encoding: StoreOffset },
];

pub fn instructions_map<'a, 'b>(instructions: &'b [Instruction<'a>]) -> HashMap<&'a str, &'b Instruction<'a>> {
    instructions.iter()
        .map(|instruction| (instruction.name, instruction))
        .collect()
}
