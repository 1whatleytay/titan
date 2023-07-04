use crate::cpu::{Memory, State};
use crate::execution::trackers::Tracker;

pub struct EmptyTracker { }

impl<Mem: Memory> Tracker<Mem> for EmptyTracker {
    fn pre_track(&mut self, _: &mut State<Mem>) { }
    fn post_track(&mut self, _: &mut State<Mem>) { }
}
