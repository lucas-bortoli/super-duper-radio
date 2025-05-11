use crate::track::{track::Track, track_iterator::TrackIterator};
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
    NarrationBefore { related_track: Track },
    Track { track: Track },
    NarrationAfter { related_track: Track },
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

pub struct StateManager {
    cancel_signal_tx: Option<oneshot::Sender<()>>,
    current_state: Arc<RwLock<State>>,
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

                let next_state = match *current_state_thread.read().unwrap() {
                    State::SwitchTrack => {
                        let track = iterator.next(&mut rng).unwrap();

                        State::Track { track }
                    }
                    State::NarrationBefore { related_track: _ } => todo!(),
                    State::Track { track: _ } => State::SwitchTrack,
                    State::NarrationAfter { related_track: _ } => todo!(),
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
