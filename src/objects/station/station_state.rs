use crate::objects::station::station::Station;

pub trait StationState {
    fn play(self: Box<Self>, station: &mut Station) -> Box<dyn StationState>;
    fn stop(self: Box<Self>, station: &mut Station) -> Box<dyn StationState>;
    fn next(self: Box<Self>, station: &mut Station) -> Box<dyn StationState>;
    fn name(&self) -> &'static str;
}

pub struct DownState;
impl DownState {
    pub fn new() -> Self { DownState }
}
impl StationState for DownState {
    fn play(self: Box<Self>, station: &mut Station) -> Box<dyn StationState> {
        station.start_playback();
        Box::new(PlayingState::new())
    }
    fn stop(self: Box<Self>, _station: &mut Station) -> Box<dyn StationState> {
        self
    }
    fn next(self: Box<Self>, _station: &mut Station) -> Box<dyn StationState> {
        self
    }
    fn name(&self) -> &'static str { "Down" }
}

pub struct PlayingState;
impl PlayingState {
    pub fn new() -> Self { PlayingState }
}
impl StationState for PlayingState {
    fn play(self: Box<Self>, _station: &mut Station) -> Box<dyn StationState> {
        self
    }
    fn stop(self: Box<Self>, station: &mut Station) -> Box<dyn StationState> {
        station.stop_playback();
        Box::new(DownState::new())
    }
    fn next(self: Box<Self>, station: &mut Station) -> Box<dyn StationState> {
        station.save_snapshot();
        if let Ok(next_track) = station.iterator.get_next() {
            station.current_track = next_track;
            station.notify_track_change();
            Box::new(PlayingState::new())
        } else {
            station.stop_playback();
            Box::new(DownState::new())
        }
    }
    fn name(&self) -> &'static str { "Playing" }
}

pub struct SwitchingState;
impl SwitchingState {
    pub fn new() -> Self { SwitchingState }
}
impl StationState for SwitchingState {
    fn play(self: Box<Self>, _station: &mut Station) -> Box<dyn StationState> {
        self
    }
    fn stop(self: Box<Self>, station: &mut Station) -> Box<dyn StationState> {
        station.stop_playback();
        Box::new(DownState::new())
    }
    fn next(self: Box<Self>, station: &mut Station) -> Box<dyn StationState> {
        Box::new(PlayingState::new())
    }
    fn name(&self) -> &'static str { "Switching" }
}