use std::collections::HashMap;

pub enum Encoding {
    // Empty
}

pub enum Opcode {
    // Empty
}

pub struct Instruction<'a> {
    pub name: &'a str,
    pub opcode: Opcode,
    pub encoding: Encoding,
}

pub const INSTRUCTIONS: [Instruction; 0] = [
    // Empty
];
