use crate::track::{track::Track, track_iterator::TrackIterator};
use frand::Rand;
use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug)]
pub enum State {
    SwitchTrack,
    NarrationBefore { related_track: Track },
    Track { track: Track },
    NarrationAfter { related_track: Track },
}

impl State {
    pub fn name(&self) -> &'static str {
        match self {
            State::SwitchTrack => "SwitchTrack",
            State::NarrationBefore { related_track: _ } => "NarrationBefore",
            State::Track { track: _ } => "Track",
            State::NarrationAfter { related_track: _ } => "NarrationAfter",
        }
    }

    pub fn determine_expected_state(tracks: Vec<Track>, seed: u64) -> (State, u64) {
        assert!(tracks.len() != 0, "track list is empty");

        const STATION_EPOCH: u64 = 1746794077052; // millis
        let current_time_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let mut rng = Rand::with_seed(seed);
        let mut current_state = State::SwitchTrack;
        let mut elapsed = current_time_unix.saturating_sub(STATION_EPOCH);

        let mut iterator = TrackIterator::new(tracks);

        loop {
            let current_step_duration = match &current_state {
                State::SwitchTrack => 0,
                State::NarrationBefore { related_track: _ } => 0,
                State::Track { track } => track.file_info.audio_milliseconds,
                State::NarrationAfter { related_track: _ } => 0,
            };

            if current_step_duration >= elapsed {
                // the current step couldn't be run to completion; in this case, "elapsed" means how further in it is in the current state
                break;
            }

            elapsed -= current_step_duration;

            current_state = match &current_state {
                State::SwitchTrack => {
                    let track = iterator.next(&mut rng).unwrap();
                    assert!(
                        track.file_info.audio_milliseconds != 0,
                        "track {} has zero duration",
                        track.title
                    );
                    State::Track { track }
                }
                State::NarrationBefore { related_track: _ } => todo!(),
                State::Track { track: _ } => State::SwitchTrack,
                State::NarrationAfter { related_track: _ } => todo!(),
            };
        }

        (current_state, elapsed)
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::SwitchTrack => write!(f, "SwitchTrack"),
            State::NarrationBefore { related_track: _ } => write!(f, "NarrationBefore"),
            State::Track { track } => write!(f, "Track[{}]", track.title),
            State::NarrationAfter { related_track: _ } => write!(f, "NarrationAfter"),
        }
    }
}
