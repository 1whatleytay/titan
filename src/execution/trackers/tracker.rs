use crate::cpu::{Memory, Registers, State};

pub trait Tracker<Mem: Memory, Reg: Registers> {
    fn pre_track(&mut self, state: &mut State<Mem, Reg>);
    fn post_track(&mut self, state: &mut State<Mem, Reg>);
}
