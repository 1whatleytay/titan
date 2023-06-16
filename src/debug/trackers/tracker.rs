use crate::cpu::{Memory, State};

pub trait Tracker<Mem: Memory> {
    fn track(&mut self, state: &mut State<Mem>);
}
