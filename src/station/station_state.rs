use std::fmt::Display;

#[derive(Clone, Debug)]
pub enum StationState {
    Initial,
    Track,
    Narration,
}

impl StationState {
    pub fn name(&self) -> &'static str {
        match self {
            StationState::Initial => "Initial",
            StationState::Track => "Track",
            StationState::Narration => "Narration",
        }
    }
}

impl Display for StationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StationState::Initial => write!(f, "Initial"),
            StationState::Track => write!(f, "Track"),
            StationState::Narration => write!(f, "Narration"),
        }
    }
}
