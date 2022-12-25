use crate::cpu::Memory;

pub struct State<Mem: Memory> {
    pub pc: u32,
    pub registers: [u32; 32],
    pub lo: u32,
    pub hi: u32,
    pub memory: Mem
}

impl<Mem: Memory> State<Mem> {
    pub fn new(entry: u32, memory: Mem) -> State<Mem> {
        State {
            pc: entry,
            registers: [0; 32],
            lo: 0,
            hi: 0,
            memory
        }
    }
}
