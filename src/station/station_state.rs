use super::station::Station;

pub enum StationState {
    Down,
    Playing,
    Switching,
}

impl StationState {
    pub fn play(self, station: &mut Station) -> Self {
        match self {
            StationState::Down => {
                station.start_playback();
                StationState::Playing
            }
            StationState::Playing => StationState::Playing,
            StationState::Switching => StationState::Switching,
        }
    }

    pub fn stop(self, station: &mut Station) -> Self {
        match self {
            StationState::Down => StationState::Down,
            StationState::Playing => {
                station.stop_playback();
                StationState::Down
            }
            StationState::Switching => {
                station.stop_playback();
                StationState::Down
            }
        }
    }

    pub fn next(self, station: &mut Station) -> Self {
        match self {
            StationState::Down => StationState::Down,
            StationState::Playing => {
                station.save_snapshot();
                if let Ok(next_track) = station.iterator.get_next() {
                    station.current_track = next_track;
                    station.notify_track_change();
                    StationState::Playing
                } else {
                    station.stop_playback();
                    StationState::Down
                }
            }
            StationState::Switching => StationState::Playing,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            StationState::Down => "Down",
            StationState::Playing => "Playing",
            StationState::Switching => "Switching",
        }
    }
}
