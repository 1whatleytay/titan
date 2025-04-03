use crate::cpu::{Memory, Registers, State};
use crate::execution::trackers::Tracker;

pub struct EmptyTracker {}

impl<Mem: Memory, Reg: Registers> Tracker<Mem, Reg> for EmptyTracker {
    fn pre_track(&mut self, _: &mut State<Mem, Reg>) {}
    fn post_track(&mut self, _: &mut State<Mem, Reg>) {}
}
