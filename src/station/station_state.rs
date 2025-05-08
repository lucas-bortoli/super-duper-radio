use std::fmt::Display;

use crate::track::track::Track;

#[derive(Clone, Debug)]
pub enum StationState {
    Initial,
    Track { track: Track },
    Narration,
    Ended,
}

impl StationState {
    pub fn name(&self) -> &'static str {
        match self {
            StationState::Initial => "Initial",
            StationState::Track { track: _ } => "Track",
            StationState::Narration => "Narration",
            StationState::Ended => "Ended",
        }
    }
}

impl Display for StationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StationState::Initial => write!(f, "Initial"),
            StationState::Track { track } => write!(f, "Track[{}]", track.title),
            StationState::Narration => write!(f, "Narration"),
            StationState::Ended => write!(f, "Ended"),
        }
    }
}
