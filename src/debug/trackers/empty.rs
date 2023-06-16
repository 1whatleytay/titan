use crate::cpu::{Memory, State};
use crate::debug::trackers::Tracker;

pub struct EmptyTracker { }

impl<Mem: Memory> Tracker<Mem> for EmptyTracker {
    fn track(&mut self, _: &mut State<Mem>) { }
}
