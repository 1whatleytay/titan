use crate::assembler::instructions::Encoding::{
    Branch, BranchZero, Destination, Immediate, Inputs, Jump, LoadImmediate, Offset, Parameterless,
    Register, RegisterShift, Sham, Source, SpecialBranch,
};
use crate::assembler::instructions::Opcode::{Algebra, Func, Op, Special};
use std::collections::HashMap;

pub enum Encoding {
    Register,                  // $, $, $, opcode: 0
    RegisterShift,             // t then s, same as Register
    Source,                    // $, opcode: 0
    Destination,               // $, opcode: 0
    Inputs,                    // $, $, opcode: 0
    Sham,                      // $, $, sham, opcode: 0
    SpecialBranch,             // opcode: 1
    Immediate(Option<Opcode>), // $, $, I
    LoadImmediate,
    Jump,   // I or Label
    Branch, // I or Label
    BranchZero,
    Parameterless,
    Offset,
}

pub enum Opcode {
    Op(u8),
    Func(u8),
    Special(u8),
    Algebra(u8),
}

pub struct Instruction<'a> {
    pub name: &'a str,
    pub opcode: Opcode,
    pub encoding: Encoding,
}

pub const INSTRUCTIONS: [Instruction; 61] = [
    Instruction {
        name: "sll",
        opcode: Func(0),
        encoding: Sham,
    },
    Instruction {
        name: "srl",
        opcode: Func(2),
        encoding: Sham,
    },
    Instruction {
        name: "sra",
        opcode: Func(3),
        encoding: Sham,
    },
    Instruction {
        name: "sllv",
        opcode: Func(4),
        encoding: RegisterShift,
    },
    Instruction {
        name: "srlv",
        opcode: Func(6),
        encoding: RegisterShift,
    },
    Instruction {
        name: "srav",
        opcode: Func(7),
        encoding: RegisterShift,
    },
    Instruction {
        name: "jr",
        opcode: Func(8),
        encoding: Source,
    },
    Instruction {
        name: "jalr",
        opcode: Func(9),
        encoding: Source,
    },
    Instruction {
        name: "mfhi",
        opcode: Func(16),
        encoding: Destination,
    },
    Instruction {
        name: "mthi",
        opcode: Func(17),
        encoding: Source,
    },
    Instruction {
        name: "mflo",
        opcode: Func(18),
        encoding: Destination,
    },
    Instruction {
        name: "mtlo",
        opcode: Func(19),
        encoding: Source,
    },
    Instruction {
        name: "mult",
        opcode: Func(24),
        encoding: Inputs,
    },
    Instruction {
        name: "multu",
        opcode: Func(25),
        encoding: Inputs,
    },
    Instruction {
        name: "div",
        opcode: Func(26),
        encoding: Inputs,
    },
    Instruction {
        name: "divu",
        opcode: Func(27),
        encoding: Inputs,
    },
    Instruction {
        name: "add",
        opcode: Func(32),
        encoding: Register,
    },
    Instruction {
        name: "addu",
        opcode: Func(33),
        encoding: Register,
    },
    Instruction {
        name: "sub",
        opcode: Func(34),
        encoding: Register,
    },
    Instruction {
        name: "subu",
        opcode: Func(35),
        encoding: Register,
    },
    Instruction {
        name: "and",
        opcode: Func(36),
        encoding: Register,
    },
    Instruction {
        name: "or",
        opcode: Func(37),
        encoding: Register,
    },
    Instruction {
        name: "xor",
        opcode: Func(38),
        encoding: Register,
    },
    Instruction {
        name: "nor",
        opcode: Func(39),
        encoding: Register,
    },
    Instruction {
        name: "sltu",
        opcode: Func(41),
        encoding: Register,
    },
    Instruction {
        name: "slt",
        opcode: Func(42),
        encoding: Register,
    },
    Instruction {
        name: "bltz",
        opcode: Special(0),
        encoding: SpecialBranch,
    },
    Instruction {
        name: "bgez",
        opcode: Special(1),
        encoding: SpecialBranch,
    },
    Instruction {
        name: "bltzal",
        opcode: Special(16),
        encoding: SpecialBranch,
    },
    Instruction {
        name: "bgezal",
        opcode: Special(17),
        encoding: SpecialBranch,
    },
    Instruction {
        name: "j",
        opcode: Op(2),
        encoding: Jump,
    },
    Instruction {
        name: "jal",
        opcode: Op(3),
        encoding: Jump,
    },
    Instruction {
        name: "beq",
        opcode: Op(4),
        encoding: Branch,
    },
    Instruction {
        name: "bne",
        opcode: Op(5),
        encoding: Branch,
    },
    Instruction {
        name: "blez",
        opcode: Op(6),
        encoding: BranchZero,
    },
    Instruction {
        name: "bgtz",
        opcode: Op(7),
        encoding: BranchZero,
    },
    Instruction {
        name: "addi",
        opcode: Op(8),
        encoding: Immediate(Some(Func(32))),
    },
    Instruction {
        name: "addiu",
        opcode: Op(9),
        encoding: Immediate(Some(Func(33))),
    },
    Instruction {
        name: "slti",
        opcode: Op(10),
        encoding: Immediate(Some(Func(42))),
    },
    Instruction {
        name: "sltiu",
        opcode: Op(11),
        encoding: Immediate(Some(Func(41))),
    },
    Instruction {
        name: "andi",
        opcode: Op(12),
        encoding: Immediate(Some(Func(36))),
    },
    Instruction {
        name: "ori",
        opcode: Op(13),
        encoding: Immediate(Some(Func(37))),
    },
    Instruction {
        name: "xori",
        opcode: Op(14),
        encoding: Immediate(Some(Func(38))),
    },
    Instruction {
        name: "lui",
        opcode: Op(15),
        encoding: LoadImmediate,
    },
    Instruction {
        name: "llo",
        opcode: Op(24),
        encoding: LoadImmediate,
    },
    Instruction {
        name: "lhi",
        opcode: Op(25),
        encoding: LoadImmediate,
    },
    Instruction {
        name: "trap",
        opcode: Op(26),
        encoding: Parameterless,
    },
    Instruction {
        name: "syscall",
        opcode: Func(12),
        encoding: Parameterless,
    },
    Instruction {
        name: "lb",
        opcode: Op(32),
        encoding: Offset,
    },
    Instruction {
        name: "lh",
        opcode: Op(33),
        encoding: Offset,
    },
    Instruction {
        name: "lw",
        opcode: Op(35),
        encoding: Offset,
    },
    Instruction {
        name: "lbu",
        opcode: Op(36),
        encoding: Offset,
    },
    Instruction {
        name: "lhu",
        opcode: Op(37),
        encoding: Offset,
    },
    Instruction {
        name: "sb",
        opcode: Op(40),
        encoding: Offset,
    },
    Instruction {
        name: "sh",
        opcode: Op(41),
        encoding: Offset,
    },
    Instruction {
        name: "sw",
        opcode: Op(43),
        encoding: Offset,
    },
    Instruction {
        name: "madd",
        opcode: Algebra(0),
        encoding: Inputs,
    },
    Instruction {
        name: "maddu",
        opcode: Algebra(1),
        encoding: Inputs,
    },
    Instruction {
        name: "mul",
        opcode: Algebra(2),
        encoding: Register,
    },
    Instruction {
        name: "msub",
        opcode: Algebra(4),
        encoding: Inputs,
    },
    Instruction {
        name: "msubu",
        opcode: Algebra(5),
        encoding: Inputs,
    },
];

pub fn instructions_map<'a, 'b>(
    instructions: &'b [Instruction<'a>],
) -> HashMap<&'a str, &'b Instruction<'a>> {
    instructions
        .iter()
        .map(|instruction| (instruction.name, instruction))
        .collect()
}
