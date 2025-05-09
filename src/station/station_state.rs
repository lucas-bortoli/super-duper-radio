use super::metadata_output_stream::MetadataOutputStream;
use crate::{
    station::metadata_output_stream::Metadata,
    track::{
        track::{StationManifest, Track},
        track_iterator::TrackIterator,
    },
};
use std::{
    fmt::Display,
    sync::{mpsc::SyncSender, Arc},
    thread,
};
use tokio::sync::oneshot;

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

    pub fn spawn_state_thread(
        manifest: StationManifest,
        mut cancel_rx: oneshot::Receiver<bool>,
        state_tx: SyncSender<StationState>,
        metadata_stream: Arc<MetadataOutputStream>,
    ) {
        thread::spawn(move || {
            let mut iterator = TrackIterator::new(manifest.tracks.clone(), manifest.seed);
            let mut current_state = StationState::Initial;

            'state_loop: loop {
                // recebemos o sinal de parada?
                match cancel_rx.try_recv() {
                    Ok(_) => break 'state_loop,
                    Err(oneshot::error::TryRecvError::Closed) => break 'state_loop,
                    _ => {}
                }

                // notificar citoplasma do estado atual
                state_tx
                    .send(current_state.clone())
                    .expect("falha ao transmitir estado atual");
                eprintln!("station: current state {}", current_state);

                // determinar próximo estado
                let next_state;

                match current_state {
                    StationState::Initial => {
                        next_state = StationState::Track {
                            track: iterator.next().unwrap(),
                        }
                    }
                    StationState::Narration => unimplemented!(),
                    StationState::Track { track: _ } => {
                        let picked_track = iterator.next().unwrap();

                        next_state = StationState::Track {
                            track: picked_track.clone(),
                        };

                        // notificar clientes do estado atual
                        metadata_stream.push(Metadata::TrackChange {
                            title: picked_track.title,
                            artist: picked_track.artist,
                        });
                    }
                    StationState::Ended => break 'state_loop,
                }

                current_state = next_state;
            }
        });
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
