use crate::execution::trackers::tracker::Tracker;

pub struct EmptyTracker;

impl<State> Tracker<State> for EmptyTracker {
    fn pre_track(&mut self, _: &mut State) {}
    fn post_track(&mut self, _: &mut State) {}
}
