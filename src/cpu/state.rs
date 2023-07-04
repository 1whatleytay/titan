use crate::cpu::Memory;

#[derive(Copy, Clone, Debug)]
pub struct Registers {
    pub pc: u32,
    pub line: [u32; 32],
    pub lo: u32,
    pub hi: u32,
}

#[derive(Clone)]
pub struct State<Mem: Memory> {
    pub registers: Registers,
    pub memory: Mem,
}

impl Registers {
    pub fn new(entry: u32) -> Registers {
        Registers {
            pc: entry,
            line: [0; 32],
            lo: 0,
            hi: 0,
        }
    }
}

impl<Mem: Memory> State<Mem> {
    pub fn new(entry: u32, memory: Mem) -> State<Mem> {
        State {
            registers: Registers::new(entry),
            memory,
        }
    }
}
