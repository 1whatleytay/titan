use crate::cpu::Memory;

pub use crate::cpu::Registers;

#[derive(Clone)]
pub struct State<Mem: Memory, Reg: Registers> {
    pub registers: Reg,
    pub memory: Mem,
}

impl<Mem: Memory, Reg: Registers> State<Mem, Reg> {
    pub fn new(registers: Reg, memory: Mem) -> State<Mem, Reg> {
        State { registers, memory }
    }
}
