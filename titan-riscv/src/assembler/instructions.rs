use crate::assembler::instructions::Encoding::{
    ArithmeticImmediate, Branch, JumpImmediate, JumpOffset, OffsetLoad, OffsetStore, Sham, Single,
    UpperImmediate,
};
use crate::assembler::instructions::Opcode::{
    BranchFunc, Executive, ImmediateFunc, LoadFunc, Op, RegisterFunc, StoreFunc,
};
use Encoding::Registers;

pub enum Encoding {
    // Empty
    UpperImmediate,
    JumpImmediate, // Special Immediate Encoding
    Branch,
    JumpOffset, // Same token layout as offset load, except also allows just one register argument
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
    LoadFunc(u8),   // Op = 0000011, 3 bits (UNS,WORD,HALF)
    ImmediateFunc(u8, bool), // Op = 0010011, bool - if true f7 = 0b0100000 else f7 = 9
    StoreFunc(u8),  // Op = 0100011
    RegisterFunc(u8, bool), // Op = 0110011, bool - if true f7 = 0b0100000 else f7 = 9
    Executive(u16), // Op = 1110011
}

pub struct Instruction<'a> {
    pub name: &'a str,
    pub opcode: Opcode,
    pub encoding: Encoding,
}

pub const LUI_OP: Opcode = Op(0b0110111);
pub const ADDI_OP: Opcode = ImmediateFunc(0b000, false);
pub const XORI_OP: Opcode = ImmediateFunc(0b100, false);
pub const SLLI_OP: Opcode = ImmediateFunc(0b001, false);
pub const SRLI_OP: Opcode = ImmediateFunc(0b101, false);
pub const SRAI_OP: Opcode = ImmediateFunc(0b101, true);

pub const SUB_OP: Opcode = RegisterFunc(0b000, true);

pub const JAL_OP: Opcode = Op(0b1101111);
pub const JALR_OP: Opcode = Op(0b1100111);

pub const BEQ_OP: Opcode = BranchFunc(0b000);
pub const BNE_OP: Opcode = BranchFunc(0b001);

pub const BLT_OP: Opcode = BranchFunc(0b100);
pub const BGE_OP: Opcode = BranchFunc(0b101);
pub const BLTU_OP: Opcode = BranchFunc(0b110);
pub const BGEU_OP: Opcode = BranchFunc(0b111);

pub const SLT_OP: Opcode = RegisterFunc(0b010, false);
pub const SLTU_OP: Opcode = RegisterFunc(0b011, false);

pub const SLTI_OP: Opcode = ImmediateFunc(0b010, false);
pub const SLTIU_OP: Opcode = ImmediateFunc(0b011, false);

// https://www.vicilogic.com/static/ext/RISCV/RV32I_BaseInstructionSet.pdf
pub const INSTRUCTIONS: [Instruction; 39] = [
    // Empty
    Instruction {
        name: "lui",
        opcode: LUI_OP,
        encoding: UpperImmediate,
    },
    Instruction {
        name: "auipc",
        opcode: Op(0b0010111),
        encoding: UpperImmediate,
    },
    Instruction {
        name: "jal",
        opcode: JAL_OP,
        encoding: JumpImmediate,
    },
    Instruction {
        name: "beq",
        opcode: BEQ_OP,
        encoding: Branch,
    },
    Instruction {
        name: "bne",
        opcode: BNE_OP,
        encoding: Branch,
    },
    Instruction {
        name: "blt",
        opcode: BLT_OP,
        encoding: Branch,
    },
    Instruction {
        name: "bge",
        opcode: BGE_OP,
        encoding: Branch,
    },
    Instruction {
        name: "bltu",
        opcode: BLTU_OP,
        encoding: Branch,
    },
    Instruction {
        name: "bgeu",
        opcode: BGEU_OP,
        encoding: Branch,
    },
    Instruction {
        name: "jalr",
        opcode: JALR_OP,
        encoding: JumpOffset,
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
        opcode: ADDI_OP,
        encoding: ArithmeticImmediate,
    },
    Instruction {
        name: "slti",
        opcode: SLTI_OP,
        encoding: ArithmeticImmediate,
    },
    Instruction {
        name: "sltiu",
        opcode: SLTIU_OP,
        encoding: ArithmeticImmediate,
    },
    Instruction {
        name: "xori",
        opcode: XORI_OP,
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
        opcode: SLLI_OP,
        encoding: Sham,
    },
    Instruction {
        name: "srli",
        opcode: SRLI_OP,
        encoding: Sham,
    },
    Instruction {
        name: "srai",
        opcode: SRAI_OP,
        encoding: Sham,
    },
    Instruction {
        name: "add",
        opcode: RegisterFunc(0b000, false),
        encoding: Registers,
    },
    Instruction {
        name: "sub",
        opcode: SUB_OP,
        encoding: Registers,
    },
    Instruction {
        name: "sll",
        opcode: RegisterFunc(0b001, false),
        encoding: Registers,
    },
    Instruction {
        name: "slt",
        opcode: SLT_OP,
        encoding: Registers,
    },
    Instruction {
        name: "sltu",
        opcode: SLTU_OP,
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
