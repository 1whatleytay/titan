use crate::cpu::registers::registers::RawRegisters;
use crate::cpu::registers::WhichRegister;
use crate::cpu::{Registers, State};
use titan_shared::cpu::error::Result;
use titan_shared::cpu::Memory;
use titan_shared::execution::executor::ExecutableState;
use WhichRegister::Pc;

impl<Mem: Memory, Reg: Registers> ExecutableState<RawRegisters, Mem> for State<Mem, Reg> {
    fn pc(&self) -> u32 {
        self.registers.get(Pc)
    }

    fn set_pc(&mut self, value: u32) {
        // Does this screw with the log?
        self.registers.set(Pc, value)
    }

    fn registers(&self) -> RawRegisters {
        self.registers.raw()
    }

    fn memory_mut(&mut self) -> &mut Mem {
        &mut self.memory
    }

    fn step(&mut self) -> Result<()> {
        self.step()
    }
}
