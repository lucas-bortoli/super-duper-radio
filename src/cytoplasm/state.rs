use crate::track::{
    track::{Narration, Track},
    track_iterator::TrackIterator,
};
use frand::Rand;
use std::{
    fmt::Display,
    sync::{mpsc, Arc, RwLock},
    thread,
};
use tokio::sync::oneshot;

#[derive(Clone, Debug)]
pub enum State {
    SwitchTrack,
    NarrationBefore {
        narration: Narration,
        track: Track,
    },
    Track {
        track: Track,
    },
    NarrationAfter {
        narration: Narration,
        track: Track,
    },
    IntentionalDelay {
        duration_units: u8,
        next_state: Box<State>,
    },
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::SwitchTrack => write!(f, "SwitchTrack"),
            State::NarrationBefore {
                narration,
                track: _,
            } => write!(
                f,
                "NarrationBefore[\"{}\", {} ms]",
                narration.transcript, narration.file_info.audio_milliseconds,
            ),
            State::Track { track } => write!(f, "Track[{}]", track.title),
            State::NarrationAfter {
                narration,
                track: _,
            } => write!(
                f,
                "NarrationAfter[\"{}\", {} ms]",
                narration.transcript, narration.file_info.audio_milliseconds,
            ),
            State::IntentionalDelay {
                duration_units,
                next_state,
            } => write!(
                f,
                "IntentionalDelay[{} units, {}]",
                duration_units, next_state,
            ),
        }
    }
}

pub struct StateManager {
    pub current_state: Arc<RwLock<State>>,
    cancel_signal_tx: Option<oneshot::Sender<()>>,
}

impl StateManager {
    pub fn new(tracks: Vec<Track>, seed: u64) -> (StateManager, mpsc::Receiver<State>) {
        let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
        let (state_tx, state_rx) = mpsc::sync_channel(0);

        let current_state = Arc::new(RwLock::new(State::SwitchTrack));

        let current_state_thread = current_state.clone();
        thread::spawn(move || {
            let mut iterator = TrackIterator::new(tracks);
            let mut rng = Rand::with_seed(seed);

            loop {
                if let Ok(_) = cancel_rx.try_recv() {
                    eprintln!("state_manager: stop signal received");
                    break;
                }

                let next_state = match current_state_thread.read().unwrap().clone() {
                    State::SwitchTrack => {
                        let track = iterator.next(&mut rng).unwrap();
                        let narration = pick_random_narration(&track.narration_before, &mut rng);

                        if let Some(narration) = narration {
                            State::IntentionalDelay {
                                duration_units: 4,
                                next_state: Box::new(State::NarrationBefore { narration, track }),
                            }
                        } else {
                            State::IntentionalDelay {
                                duration_units: 2,
                                next_state: Box::new(State::Track { track }),
                            }
                        }
                    }
                    State::NarrationBefore {
                        narration: _,
                        track,
                    } => State::IntentionalDelay {
                        duration_units: 2,
                        next_state: Box::new(State::Track { track }),
                    },
                    State::Track { track } => {
                        let narration = pick_random_narration(&track.narration_after, &mut rng);
                        if let Some(narration) = narration {
                            State::IntentionalDelay {
                                duration_units: 4,
                                next_state: Box::new(State::NarrationAfter { narration, track }),
                            }
                        } else {
                            State::SwitchTrack
                        }
                    }
                    State::NarrationAfter {
                        narration: _,
                        track: _,
                    } => State::SwitchTrack,
                    State::IntentionalDelay {
                        duration_units: _,
                        next_state,
                    } => *next_state,
                };

                *current_state_thread.write().unwrap() = next_state.clone();

                // notify that the state changed, then block until the receiver acknowleges
                if let Err(err) = state_tx.send(next_state.clone()) {
                    eprintln!("state_manager: state send error: {}", err);
                    break;
                }
            }
        });

        let manager = StateManager {
            cancel_signal_tx: Some(cancel_tx),
            current_state,
        };

        (manager, state_rx)
    }
}

impl Drop for StateManager {
    fn drop(&mut self) {
        if let Some(tx) = self.cancel_signal_tx.take() {
            let _ = tx.send(());
        }
    }
}

fn pick_random_narration(pool: &Vec<Narration>, rng: &mut Rand) -> Option<Narration> {
    if pool.len() == 0 {
        return None;
    } else {
        let idx = rng.gen_range(0..pool.len() as u64) as usize;
        Some(pool.get(idx).unwrap().clone())
    }
}
