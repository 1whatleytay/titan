use crate::cpu::{Memory, State};

pub trait Tracker<Mem: Memory> {
    fn pre_track(&mut self, state: &mut State<Mem>);
    fn post_track(&mut self, state: &mut State<Mem>);
}
