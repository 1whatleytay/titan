use crate::assembler::instructions::Encoding::{
    Branch, BranchZero, Destination, FP2Register, FP3Register, FPBranch, FPCond, FPCrossMove,
    FPMove, FPOffset, Immediate, Inputs, Jump, LoadImmediate, Offset, Parameterless, Register,
    RegisterShift, Sham, Source, SpecialBranch,
};
use crate::assembler::instructions::Opcode::{Algebra, Cop1, Cop1I, Func, Op, Special};
use crate::assembler::instructions::Size::{Double, Single, Word};
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
    FPOffset,
    FP3Register(Size),  // Size, $, $, $
    FP2Register(Size),  // Size, 0, $, $
    FPMove(Size, bool), // Size, $, $, cc|bool
    FPCond(Size),       // Size, cc, $, $
    FPCrossMove(bool),  // direction
    FPBranch(bool),
}

#[derive(Debug, Clone, Copy)]
pub enum Size {
    Single,
    Double,
    Word,
}
pub enum Opcode {
    Op(u8),
    Func(u8),
    Special(u8),
    Algebra(u8),
    Cop1(u8),
    Cop1I(u8),
}

pub struct Instruction<'a> {
    pub name: &'a str,
    pub opcode: Opcode,
    pub encoding: Encoding,
}

pub const INSTRUCTIONS: [Instruction; 115] = [
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
    Instruction {
        name: "add.s",
        opcode: Cop1(0),
        encoding: FP3Register(Single),
    },
    Instruction {
        name: "sub.s",
        opcode: Cop1(1),
        encoding: FP3Register(Single),
    },
    Instruction {
        name: "mul.s",
        opcode: Cop1(2),
        encoding: FP3Register(Single),
    },
    Instruction {
        name: "div.s",
        opcode: Cop1(3),
        encoding: FP3Register(Single),
    },
    Instruction {
        name: "sqrt.s",
        opcode: Cop1(4),
        encoding: FP2Register(Single),
    },
    Instruction {
        name: "abs.s",
        opcode: Cop1(5),
        encoding: FP2Register(Single),
    },
    Instruction {
        name: "mov.s",
        opcode: Cop1(6),
        encoding: FP2Register(Single),
    },
    Instruction {
        name: "neg.s",
        opcode: Cop1(7),
        encoding: FP2Register(Single),
    },
    Instruction {
        name: "round.w.s",
        opcode: Cop1(12),
        encoding: FP2Register(Single),
    },
    Instruction {
        name: "trunc.w.s",
        opcode: Cop1(13),
        encoding: FP2Register(Single),
    },
    Instruction {
        name: "ceil.w.s",
        opcode: Cop1(14),
        encoding: FP2Register(Single),
    },
    Instruction {
        name: "floor.w.s",
        opcode: Cop1(15),
        encoding: FP2Register(Single),
    },
    Instruction {
        name: "add.d",
        opcode: Cop1(0),
        encoding: FP3Register(Double),
    },
    Instruction {
        name: "sub.d",
        opcode: Cop1(1),
        encoding: FP3Register(Double),
    },
    Instruction {
        name: "mul.d",
        opcode: Cop1(2),
        encoding: FP3Register(Double),
    },
    Instruction {
        name: "div.d",
        opcode: Cop1(3),
        encoding: FP3Register(Double),
    },
    Instruction {
        name: "sqrt.d",
        opcode: Cop1(4),
        encoding: FP2Register(Double),
    },
    Instruction {
        name: "abs.d",
        opcode: Cop1(5),
        encoding: FP2Register(Double),
    },
    Instruction {
        name: "mov.d",
        opcode: Cop1(6),
        encoding: FP2Register(Double),
    },
    Instruction {
        name: "neg.d",
        opcode: Cop1(7),
        encoding: FP2Register(Double),
    },
    Instruction {
        name: "round.w.d",
        opcode: Cop1(12),
        encoding: FP2Register(Double),
    },
    Instruction {
        name: "trunc.w.d",
        opcode: Cop1(13),
        encoding: FP2Register(Double),
    },
    Instruction {
        name: "ceil.w.d",
        opcode: Cop1(14),
        encoding: FP2Register(Double),
    },
    Instruction {
        name: "floor.w.d",
        opcode: Cop1(15),
        encoding: FP2Register(Double),
    },
    Instruction {
        name: "c.eq.s",
        opcode: Cop1(0b110010),
        encoding: FPCond(Single),
    },
    Instruction {
        name: "c.le.s",
        opcode: Cop1(0b111110),
        encoding: FPCond(Single),
    },
    Instruction {
        name: "c.lt.s",
        opcode: Cop1(0b111100),
        encoding: FPCond(Single),
    },
    Instruction {
        name: "c.eq.d",
        opcode: Cop1(0b110010),
        encoding: FPCond(Double),
    },
    Instruction {
        name: "c.le.d",
        opcode: Cop1(0b111110),
        encoding: FPCond(Double),
    },
    Instruction {
        name: "c.lt.d",
        opcode: Cop1(0b111100),
        encoding: FPCond(Double),
    },
    Instruction {
        name: "bc1t",
        opcode: Cop1I(0b01000),
        encoding: FPBranch(true),
    },
    Instruction {
        name: "bc1f",
        opcode: Cop1I(0b01000),
        encoding: FPBranch(false),
    },
    Instruction {
        name: "mov.s",
        opcode: Cop1(0b000110),
        encoding: FP3Register(Single),
    },
    Instruction {
        name: "movf.s",
        opcode: Cop1(0b010001),
        encoding: FPMove(Single, false),
    },
    Instruction {
        name: "movt.s",
        opcode: Cop1(0b010001),
        encoding: FPMove(Single, true),
    },
    Instruction {
        name: "movn.s",
        opcode: Cop1(0b010011),
        encoding: FP3Register(Single),
    },
    Instruction {
        name: "movz.s",
        opcode: Cop1(0b010010),
        encoding: FP3Register(Single),
    },
    Instruction {
        name: "mov.d",
        opcode: Cop1(0b000110),
        encoding: FP3Register(Double),
    },
    Instruction {
        name: "movf.d",
        opcode: Cop1(0b010001),
        encoding: FPMove(Double, false),
    },
    Instruction {
        name: "movt.d",
        opcode: Cop1(0b010001),
        encoding: FPMove(Double, true),
    },
    Instruction {
        name: "movn.d",
        opcode: Cop1(0b010011),
        encoding: FP3Register(Double),
    },
    Instruction {
        name: "movz.d",
        opcode: Cop1(0b010010),
        encoding: FP3Register(Double),
    },
    Instruction {
        name: "cvt.s.w",
        opcode: Cop1(0b100000),
        encoding: FP2Register(Single),
    },
    Instruction {
        name: "cvt.s.d",
        opcode: Cop1(0b100000),
        encoding: FP2Register(Double),
    },
    Instruction {
        name: "cvt.w.s",
        opcode: Cop1(0b100100),
        encoding: FP2Register(Single),
    },
    Instruction {
        name: "cvt.w.d",
        opcode: Cop1(0b100100),
        encoding: FP2Register(Double),
    },
    Instruction {
        name: "cvt.d.s",
        opcode: Cop1(0b100001),
        encoding: FP2Register(Double),
    },
    Instruction {
        name: "cvt.d.w",
        opcode: Cop1(0b100001),
        encoding: FP2Register(Word),
    },
    Instruction {
        name: "mfc1",
        opcode: Cop1I(0b00000),
        encoding: FPCrossMove(false),
    },
    Instruction {
        name: "mtc1",
        opcode: Cop1I(0b00100),
        encoding: FPCrossMove(true),
    },
    Instruction {
        name: "lwc1",
        opcode: Op(0b110001),
        encoding: FPOffset,
    },
    Instruction {
        name: "swc1",
        opcode: Op(0b111001),
        encoding: FPOffset,
    },
    Instruction {
        name: "ldc1",
        opcode: Op(0b110101),
        encoding: FPOffset,
    },
    Instruction {
        name: "sdc1",
        opcode: Op(0b111101),
        encoding: FPOffset,
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
