use crate::assembler::instructions::BaseOpcode::{
    BranchFunc, Executive, ImmediateFunc, LoadFunc, MulDiv, Op, RegisterFunc, StoreFunc,
};
use crate::assembler::instructions::CompressedOpcode::{OpFunc, SpecHigh, SpecHighLow, SpecRd};
use crate::assembler::instructions::Encoding::{
    ArithmeticImmediate, Branch, CompressedAddi4, CompressedAddi16, CompressedAssignImmediate,
    CompressedBitImmediate, CompressedBranch, CompressedDoubleRegister, CompressedJump,
    CompressedLoadWord, CompressedLoadWordSp, CompressedLui, CompressedOnlyRegister,
    CompressedShift, CompressedShiftExtended, CompressedSingle, CompressedSmallRegs,
    CompressedStoreWord, CompressedStoreWordSp, JumpImmediate, JumpOffset, OffsetLoad, OffsetStore,
    Sham, Single, UpperImmediate,
};
use crate::assembler::registers::RegisterSlot;
use CompressedOpcode::Spec12;
use Encoding::Registers;

pub enum Encoding {
    // Base Encodings
    UpperImmediate { op: BaseOpcode },
    JumpImmediate { op: BaseOpcode }, // Special Immediate Encoding
    Branch { op: BaseOpcode },
    JumpOffset { op: BaseOpcode }, // Same token layout as offset load, except also allows just one register argument
    OffsetLoad { op: BaseOpcode },
    OffsetStore { op: BaseOpcode },
    ArithmeticImmediate { op: BaseOpcode },
    Sham { op: BaseOpcode },
    Registers { op: BaseOpcode },
    Single { op: BaseOpcode },
    // Fences are TODO - They have a pretty involved encoding.
    // Compressed Encodings
    CompressedAddi4 { op: CompressedOpcode }, // uses imm 549623
    CompressedLoadWord { op: CompressedOpcode }, // uses rd
    CompressedStoreWord { op: CompressedOpcode }, // uses rs2
    CompressedSingle { op: CompressedOpcode }, // for nop
    CompressedAssignImmediate { op: CompressedOpcode }, // c.addi, c.li
    CompressedJump { op: CompressedOpcode },  // c.jal, c.j
    CompressedAddi16 { op: CompressedOpcode }, // uses imm 946875
    CompressedLui { op: CompressedOpcode },   // c.lui
    CompressedShift { op: CompressedOpcode }, // c.srli, c.srai
    CompressedShiftExtended { op: CompressedOpcode }, // c.slli
    CompressedBitImmediate { op: CompressedOpcode }, // c.andi, similar to CompressedShift except signed imm
    CompressedSmallRegs { op: CompressedOpcode },    // c.sub, c.xor, c.or, etc...
    CompressedBranch { op: CompressedOpcode },
    CompressedOnlyRegister { op: CompressedOpcode }, // c.jr, c.jalr
    CompressedDoubleRegister { op: CompressedOpcode }, // c.add, c.mv
    CompressedLoadWordSp { op: CompressedOpcode },
    CompressedStoreWordSp { op: CompressedOpcode },
}

pub enum BaseOpcode {
    Op(u8),
    BranchFunc(u8), // Op = 1100011, 3 bits (0=EQ/1=LT, 0=SIGNED/1=UNSIGNED, 0=NORMAL,1=NOT)
    LoadFunc(u8),   // Op = 0000011, 3 bits (UNS,WORD,HALF)
    ImmediateFunc(u8, bool), // Op = 0010011, bool - if true f7 = 0b0100000 else f7 = 9
    StoreFunc(u8),  // Op = 0100011
    RegisterFunc(u8, bool), // Op = 0110011, bool - if true f7 = 0b0100000 else f7 = 9
    Executive(u16), // Op = 1110011

    MulDiv(u8), // Op = 0110011, 3 bits func, funct7 = 0000001
}

pub enum CompressedOpcode {
    OpFunc {
        op: u8,
        func: u8,
    }, // op - 2 bits, func - 3 bits
    SpecRd {
        op: u8,
        func: u8,
        rd: RegisterSlot,
    },
    SpecHigh {
        op: u8,
        func: u8,
        high: u8,
    }, // op - 2 bits, func - 3 bits, high - 2 bits
    SpecHighLow {
        op: u8,
        func: u8,
        high: u8,
        low: u8,
        bit12: bool,
    }, // op - 2 bits, func - 3 bits, high - 2 bits, low - 2 bits
    Spec12 {
        op: u8,
        func: u8,
        bit12: bool,
    },
}

pub struct Instruction<'a> {
    pub name: &'a str,
    pub encoding: Encoding,
}

pub const LUI_OP: BaseOpcode = Op(0b0110111);
pub const ADDI_OP: BaseOpcode = ImmediateFunc(0b000, false);
pub const XORI_OP: BaseOpcode = ImmediateFunc(0b100, false);
pub const SLLI_OP: BaseOpcode = ImmediateFunc(0b001, false);
pub const SRLI_OP: BaseOpcode = ImmediateFunc(0b101, false);
pub const SRAI_OP: BaseOpcode = ImmediateFunc(0b101, true);

pub const SUB_OP: BaseOpcode = RegisterFunc(0b000, true);

pub const JAL_OP: BaseOpcode = Op(0b1101111);
pub const JALR_OP: BaseOpcode = Op(0b1100111);

pub const BEQ_OP: BaseOpcode = BranchFunc(0b000);
pub const BNE_OP: BaseOpcode = BranchFunc(0b001);

pub const BLT_OP: BaseOpcode = BranchFunc(0b100);
pub const BGE_OP: BaseOpcode = BranchFunc(0b101);
pub const BLTU_OP: BaseOpcode = BranchFunc(0b110);
pub const BGEU_OP: BaseOpcode = BranchFunc(0b111);

pub const SLT_OP: BaseOpcode = RegisterFunc(0b010, false);
pub const SLTU_OP: BaseOpcode = RegisterFunc(0b011, false);

pub const SLTI_OP: BaseOpcode = ImmediateFunc(0b010, false);
pub const SLTIU_OP: BaseOpcode = ImmediateFunc(0b011, false);

// https://www.vicilogic.com/static/ext/RISCV/RV32I_BaseInstructionSet.pdf
pub const INSTRUCTIONS: &[Instruction] = &[
    // Empty
    Instruction {
        name: "lui",
        encoding: UpperImmediate { op: LUI_OP },
    },
    Instruction {
        name: "auipc",
        encoding: UpperImmediate { op: Op(0b0010111) },
    },
    Instruction {
        name: "jal",
        encoding: JumpImmediate { op: JAL_OP },
    },
    Instruction {
        name: "beq",
        encoding: Branch { op: BEQ_OP },
    },
    Instruction {
        name: "bne",
        encoding: Branch { op: BNE_OP },
    },
    Instruction {
        name: "blt",
        encoding: Branch { op: BLT_OP },
    },
    Instruction {
        name: "bge",
        encoding: Branch { op: BGE_OP },
    },
    Instruction {
        name: "bltu",
        encoding: Branch { op: BLTU_OP },
    },
    Instruction {
        name: "bgeu",
        encoding: Branch { op: BGEU_OP },
    },
    Instruction {
        name: "jalr",
        encoding: JumpOffset { op: JALR_OP },
    },
    Instruction {
        name: "lb",
        encoding: OffsetLoad {
            op: LoadFunc(0b000),
        },
    },
    Instruction {
        name: "lh",
        encoding: OffsetLoad {
            op: LoadFunc(0b001),
        },
    },
    Instruction {
        name: "lw",
        encoding: OffsetLoad {
            op: LoadFunc(0b010),
        },
    },
    Instruction {
        name: "lbu",
        encoding: OffsetLoad {
            op: LoadFunc(0b100),
        },
    },
    Instruction {
        name: "lhu",
        encoding: OffsetLoad {
            op: LoadFunc(0b101),
        },
    },
    Instruction {
        name: "addi",
        encoding: ArithmeticImmediate { op: ADDI_OP },
    },
    Instruction {
        name: "slti",
        encoding: ArithmeticImmediate { op: SLTI_OP },
    },
    Instruction {
        name: "sltiu",
        encoding: ArithmeticImmediate { op: SLTIU_OP },
    },
    Instruction {
        name: "xori",
        encoding: ArithmeticImmediate { op: XORI_OP },
    },
    Instruction {
        name: "ori",
        encoding: ArithmeticImmediate {
            op: ImmediateFunc(0b110, false),
        },
    },
    Instruction {
        name: "andi",
        encoding: ArithmeticImmediate {
            op: ImmediateFunc(0b111, false),
        },
    },
    Instruction {
        name: "sb",
        encoding: OffsetStore {
            op: StoreFunc(0b000),
        },
    },
    Instruction {
        name: "sh",
        encoding: OffsetStore {
            op: StoreFunc(0b001),
        },
    },
    Instruction {
        name: "sw",
        encoding: OffsetStore {
            op: StoreFunc(0b010),
        },
    },
    Instruction {
        name: "slli",
        encoding: Sham { op: SLLI_OP },
    },
    Instruction {
        name: "srli",
        encoding: Sham { op: SRLI_OP },
    },
    Instruction {
        name: "srai",
        encoding: Sham { op: SRAI_OP },
    },
    Instruction {
        name: "add",
        encoding: Registers {
            op: RegisterFunc(0b000, false),
        },
    },
    Instruction {
        name: "sub",
        encoding: Registers { op: SUB_OP },
    },
    Instruction {
        name: "sll",
        encoding: Registers {
            op: RegisterFunc(0b001, false),
        },
    },
    Instruction {
        name: "slt",
        encoding: Registers { op: SLT_OP },
    },
    Instruction {
        name: "sltu",
        encoding: Registers { op: SLTU_OP },
    },
    Instruction {
        name: "xor",
        encoding: Registers {
            op: RegisterFunc(0b100, false),
        },
    },
    Instruction {
        name: "srl",
        encoding: Registers {
            op: RegisterFunc(0b101, false),
        },
    },
    Instruction {
        name: "sra",
        encoding: Registers {
            op: RegisterFunc(0b101, true),
        },
    },
    Instruction {
        name: "or",
        encoding: Registers {
            op: RegisterFunc(0b110, false),
        },
    },
    Instruction {
        name: "and",
        encoding: Registers {
            op: RegisterFunc(0b111, false),
        },
    },
    // No Fence
    Instruction {
        name: "ecall",
        encoding: Single { op: Executive(0b0) },
    },
    Instruction {
        name: "ebreak",
        encoding: Single { op: Executive(0b1) },
    },
    // Compressed Instructions
    /*
       c.lwsp
       c.swsp
       c.lw
       c.sw
       c.j
       c.jal
       c.jr
       c.jalr
       c.beqz
       c.bnez
       c.li
       c.lui
       c.addi
       c.addi16sp
       c.addi4spn
       c.slli
       c.srli
       c.srai
       c.andi
       c.mv
       c.add
       c.and
       c.or
       c.xor
       c.sub
       c.nop
       c.ebreak
    */
    Instruction {
        name: "c.lwsp",
        encoding: CompressedLoadWordSp {
            op: OpFunc {
                op: 0b10,
                func: 0b010,
            },
        },
    },
    Instruction {
        name: "c.swsp",
        encoding: CompressedStoreWordSp {
            op: OpFunc {
                op: 0b10,
                func: 0b110,
            },
        },
    },
    Instruction {
        name: "c.lw",
        encoding: CompressedLoadWord {
            op: OpFunc {
                op: 0b00,
                func: 0b010,
            },
        },
    },
    Instruction {
        name: "c.sw",
        encoding: CompressedStoreWord {
            op: OpFunc {
                op: 0b00,
                func: 0b110,
            },
        },
    },
    Instruction {
        name: "c.j",
        encoding: CompressedJump {
            op: OpFunc {
                op: 0b01,
                func: 0b101,
            },
        },
    },
    Instruction {
        name: "c.jal",
        encoding: CompressedJump {
            op: OpFunc {
                op: 0b01,
                func: 0b001,
            },
        },
    },
    Instruction {
        name: "c.jr",
        encoding: CompressedOnlyRegister {
            op: Spec12 {
                op: 0b10,
                func: 0b100,
                bit12: false,
            },
        },
    },
    Instruction {
        name: "c.jalr",
        encoding: CompressedOnlyRegister {
            op: Spec12 {
                op: 0b10,
                func: 0b100,
                bit12: true,
            },
        },
    },
    Instruction {
        name: "c.beqz",
        encoding: CompressedBranch {
            op: OpFunc {
                op: 0b01,
                func: 0b110,
            },
        },
    },
    Instruction {
        name: "c.bnez",
        encoding: CompressedBranch {
            op: OpFunc {
                op: 0b01,
                func: 0b111,
            },
        },
    },
    Instruction {
        name: "c.li",
        encoding: CompressedAssignImmediate {
            op: OpFunc {
                op: 0b01,
                func: 0b010,
            },
        },
    },
    Instruction {
        name: "c.lui",
        encoding: CompressedLui {
            op: OpFunc {
                op: 0b01,
                func: 0b011,
            },
        }, // cannot be 2 as 2 is c.addi16sp
    },
    Instruction {
        name: "c.addi",
        encoding: CompressedAssignImmediate {
            op: OpFunc {
                op: 0b01,
                func: 0b000,
            },
        },
    },
    Instruction {
        name: "c.addi16sp",
        encoding: CompressedAddi16 {
            op: SpecRd {
                op: 0b01,
                func: 0b011,
                rd: RegisterSlot::StackPointer,
            },
        },
    },
    Instruction {
        name: "c.addi4spn",
        encoding: CompressedAddi4 {
            op: OpFunc {
                op: 0b00,
                func: 0b000,
            },
        },
    },
    Instruction {
        name: "c.slli",
        encoding: CompressedShiftExtended {
            op: OpFunc {
                op: 0b10,
                func: 0b000,
            },
        }, // different quad!
    },
    Instruction {
        name: "c.srli",
        encoding: CompressedShift {
            op: SpecHigh {
                op: 0b01,
                func: 0b100,
                high: 0b00,
            },
        },
    },
    Instruction {
        name: "c.srai",
        // not a shift! seems closer to encoding as compressed assign immediate
        encoding: CompressedShift {
            op: SpecHigh {
                op: 0b01,
                func: 0b100,
                high: 0b01,
            },
        },
    },
    Instruction {
        name: "c.andi",
        encoding: CompressedBitImmediate {
            op: SpecHigh {
                op: 0b01,
                func: 0b100,
                high: 0b10,
            },
        },
    },
    Instruction {
        name: "c.mv",
        encoding: CompressedDoubleRegister {
            op: Spec12 {
                op: 0b10,
                func: 0b100,
                bit12: false,
            },
        },
    },
    Instruction {
        name: "c.add",
        encoding: CompressedDoubleRegister {
            op: Spec12 {
                op: 0b10,
                func: 0b100,
                bit12: true,
            },
        },
    },
    Instruction {
        name: "c.and",
        encoding: CompressedSmallRegs {
            op: SpecHighLow {
                op: 0b01,
                func: 0b100,
                high: 0b11,
                low: 0b11,
                bit12: false,
            },
        },
    },
    Instruction {
        name: "c.or",
        encoding: CompressedSmallRegs {
            op: SpecHighLow {
                op: 0b01,
                func: 0b100,
                high: 0b11,
                low: 0b10,
                bit12: false,
            },
        },
    },
    Instruction {
        name: "c.xor",
        encoding: CompressedSmallRegs {
            op: SpecHighLow {
                op: 0b01,
                func: 0b100,
                high: 0b11,
                low: 0b01,
                bit12: false,
            },
        },
    },
    Instruction {
        name: "c.sub",
        encoding: CompressedSmallRegs {
            op: SpecHighLow {
                op: 0b01,
                func: 0b100,
                high: 0b11,
                low: 0b00,
                bit12: false,
            },
        },
    },
    Instruction {
        name: "c.nop",
        encoding: CompressedSingle {
            op: OpFunc {
                op: 0b01,
                func: 0b000,
            },
        },
    },
    Instruction {
        name: "c.ebreak",
        encoding: CompressedSingle {
            op: Spec12 {
                op: 0b10,
                func: 0b100,
                bit12: true,
            },
        },
    },
    // Multiplication and Division Extension
    /*
    mul
    mulh
    mulhsu
    mulhu
    div
    divu
    rem
    remu
     */
    Instruction {
        name: "mul",
        encoding: Registers { op: MulDiv(0b000) },
    },
    Instruction {
        name: "mulh",
        encoding: Registers { op: MulDiv(0b001) },
    },
    Instruction {
        name: "mulhsu",
        encoding: Registers { op: MulDiv(0b010) },
    },
    Instruction {
        name: "mulhu",
        encoding: Registers { op: MulDiv(0b011) },
    },
    Instruction {
        name: "div",
        encoding: Registers { op: MulDiv(0b100) },
    },
    Instruction {
        name: "divu",
        encoding: Registers { op: MulDiv(0b101) },
    },
    Instruction {
        name: "rem",
        encoding: Registers { op: MulDiv(0b110) },
    },
    Instruction {
        name: "remu",
        encoding: Registers { op: MulDiv(0b111) },
    },
];
