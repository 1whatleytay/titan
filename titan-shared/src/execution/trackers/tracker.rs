pub trait Tracker<State> {
    fn pre_track(&mut self, state: &mut State);
    fn post_track(&mut self, state: &mut State);
}
