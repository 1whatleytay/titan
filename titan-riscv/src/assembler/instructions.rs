use Encoding::Registers;
use crate::assembler::binary_builder::InstructionLabelKind::Upper;
use crate::assembler::instructions::Encoding::{ArithmeticImmediate, Branch, JumpImmediate, OffsetLoad, OffsetStore, Sham, Single, UpperImmediate};
use crate::assembler::instructions::Opcode::{BranchFunc, Executive, ImmediateFunc, LoadFunc, Op, RegisterFunc, StoreFunc};

pub enum Encoding {
    // Empty
    UpperImmediate,
    JumpImmediate, // Special Immediate Encoding
    Branch,
    OffsetLoad,
    OffsetStore,
    ArithmeticImmediate,
    Sham,
    Registers,
    Single,
    // Fences are TODO - They have a pretty involved encoding.
}

pub enum Opcode {
    // Empty
    Op(u8),
    BranchFunc(u8), // Op = 1100011, 3 bits (0=EQ/1=LT, 0=SIGNED/1=UNSIGNED, 0=NORMAL,1=NOT)
    LoadFunc(u8), // Op = 0000011, 3 bits (UNS,WORD,HALF)
    ImmediateFunc(u8, bool), // Op = 0010011, bool - if true f7 = 0b0100000 else f7 = 9
    StoreFunc(u8), // Op = 0100011
    RegisterFunc(u8, bool), // Op = 0110011, bool - if true f7 = 0b0100000 else f7 = 9
    Executive(u16), // Op = 1110011
}

pub struct Instruction<'a> {
    pub name: &'a str,
    pub opcode: Opcode,
    pub encoding: Encoding,
}

// https://www.vicilogic.com/static/ext/RISCV/RV32I_BaseInstructionSet.pdf
pub const INSTRUCTIONS: [Instruction; 39] = [
    // Empty
    Instruction {
        name: "lui",
        opcode: Op(0b0110111),
        encoding: UpperImmediate,
    },
    Instruction {
        name: "auipc",
        opcode: Op(0b0010111),
        encoding: UpperImmediate,
    },
    Instruction {
        name: "jal",
        opcode: Op(0b1101111),
        encoding: JumpImmediate,
    },
    Instruction {
        name: "beq",
        opcode: BranchFunc(0b000),
        encoding: Branch,
    },
    Instruction {
        name: "bne",
        opcode: BranchFunc(0b001),
        encoding: Branch,
    },
    Instruction {
        name: "blt",
        opcode: BranchFunc(0b100),
        encoding: Branch,
    },
    Instruction {
        name: "bge",
        opcode: BranchFunc(0b101),
        encoding: Branch,
    },
    Instruction {
        name: "bltu",
        opcode: BranchFunc(0b110),
        encoding: Branch,
    },
    Instruction {
        name: "bgeu",
        opcode: BranchFunc(0b111),
        encoding: Branch,
    },
    Instruction {
        name: "jalr",
        opcode: Op(0b1100111),
        encoding: OffsetLoad,
    },
    Instruction {
        name: "lb",
        opcode: LoadFunc(0b000),
        encoding: OffsetLoad,
    },
    Instruction {
        name: "lh",
        opcode: LoadFunc(0b001),
        encoding: OffsetLoad,
    },
    Instruction {
        name: "lw",
        opcode: LoadFunc(0b010),
        encoding: OffsetLoad,
    },
    Instruction {
        name: "lbu",
        opcode: LoadFunc(0b100),
        encoding: OffsetLoad,
    },
    Instruction {
        name: "lhu",
        opcode: LoadFunc(0b101),
        encoding: OffsetLoad,
    },
    Instruction {
        name: "addi",
        opcode: ImmediateFunc(0b000, false),
        encoding: ArithmeticImmediate,
    },
    Instruction {
        name: "slti",
        opcode: ImmediateFunc(0b010, false),
        encoding: ArithmeticImmediate,
    },
    Instruction {
        name: "sltiu",
        opcode: ImmediateFunc(0b011, false),
        encoding: ArithmeticImmediate,
    },
    Instruction {
        name: "xori",
        opcode: ImmediateFunc(0b100, false),
        encoding: ArithmeticImmediate,
    },
    Instruction {
        name: "ori",
        opcode: ImmediateFunc(0b110, false),
        encoding: ArithmeticImmediate,
    },
    Instruction {
        name: "andi",
        opcode: ImmediateFunc(0b111, false),
        encoding: ArithmeticImmediate,
    },
    Instruction {
        name: "sb",
        opcode: StoreFunc(0b000),
        encoding: OffsetStore,
    },
    Instruction {
        name: "sh",
        opcode: StoreFunc(0b001),
        encoding: OffsetStore,
    },
    Instruction {
        name: "sw",
        opcode: StoreFunc(0b010),
        encoding: OffsetStore,
    },
    Instruction {
        name: "slli",
        opcode: ImmediateFunc(0b001, false),
        encoding: Sham,
    },
    Instruction {
        name: "srli",
        opcode: ImmediateFunc(0b101, false),
        encoding: Sham,
    },
    Instruction {
        name: "srai",
        opcode: ImmediateFunc(0b101, true),
        encoding: Sham,
    },
    Instruction {
        name: "add",
        opcode: RegisterFunc(0b000, false),
        encoding: Registers,
    },
    Instruction {
        name: "sub",
        opcode: RegisterFunc(0b000, true),
        encoding: Registers,
    },
    Instruction {
        name: "sll",
        opcode: RegisterFunc(0b001, false),
        encoding: Registers,
    },
    Instruction {
        name: "slt",
        opcode: RegisterFunc(0b010, false),
        encoding: Registers,
    },
    Instruction {
        name: "sltu",
        opcode: RegisterFunc(0b011, false),
        encoding: Registers,
    },
    Instruction {
        name: "xor",
        opcode: RegisterFunc(0b100, false),
        encoding: Registers,
    },
    Instruction {
        name: "srl",
        opcode: RegisterFunc(0b101, false),
        encoding: Registers,
    },
    Instruction {
        name: "sra",
        opcode: RegisterFunc(0b101, true),
        encoding: Registers,
    },
    Instruction {
        name: "or",
        opcode: RegisterFunc(0b110, false),
        encoding: Registers,
    },
    Instruction {
        name: "and",
        opcode: RegisterFunc(0b111, false),
        encoding: Registers,
    },
    // No Fence
    Instruction {
        name: "ecall",
        opcode: Executive(0b0),
        encoding: Single,
    },
    Instruction {
        name: "ebreak",
        opcode: Executive(0b1),
        encoding: Single,
    },
];
