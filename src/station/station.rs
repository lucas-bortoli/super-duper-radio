use rocket::time::OffsetDateTime;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::{path::PathBuf, thread};
use tokio::sync::oneshot;

use crate::station::metadata_output_stream::Metadata;
use crate::{
    cytoplasm::cytoplasm::Cytoplasm,
    track::{track::StationManifest, track_iterator::TrackIterator},
};

use super::metadata_output_stream::{self, MetadataOutputStream};
use super::{station_snapshot::StationSnapshot, station_state::StationState};

pub struct Station {
    pub base_dir: PathBuf,
    pub manifest: StationManifest,
    pub cytoplasm: Cytoplasm,

    pub metadata_stream: Arc<MetadataOutputStream>,

    pub state: StationState,
    pub snapshots: Vec<StationSnapshot>,
    last_snapshot_time: OffsetDateTime,

    _cancel_signal_sender: Option<oneshot::Sender<bool>>,
}

impl Station {
    pub fn new(
        base_dir: PathBuf,
        manifest: StationManifest,
        cytoplasm: Cytoplasm,
        state_tx: SyncSender<StationState>,
    ) -> Station {
        let now = OffsetDateTime::now_utc();

        let metadata_stream = Arc::new(MetadataOutputStream::new());

        let state_thread_metadata_stream = metadata_stream.clone();
        let state_thread_manifest = manifest.clone();
        let (cancel_signal_sender, mut cancel_signal_receiver) = oneshot::channel::<bool>();
        thread::spawn(move || {
            let mut iterator = TrackIterator::new(
                state_thread_manifest.tracks.clone(),
                state_thread_manifest.seed,
            );

            let mut current_state = StationState::Initial;

            'state_loop: loop {
                // recebemos o sinal de parada?
                match cancel_signal_receiver.try_recv() {
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
                        state_thread_metadata_stream.push(Metadata::TrackChange {
                            title: picked_track.title,
                            artist: picked_track.artist,
                        });
                    }
                    StationState::Ended => break 'state_loop,
                }

                current_state = next_state;
            }
        });

        Station {
            base_dir,
            manifest,
            cytoplasm,
            metadata_stream,
            state: StationState::Initial,
            snapshots: Vec::new(),
            last_snapshot_time: now,
            _cancel_signal_sender: Some(cancel_signal_sender),
        }
    }

    pub fn save_snapshot(&mut self) {
        let now = OffsetDateTime::now_utc();
        let delta = now - self.last_snapshot_time;
        let duration_secs = delta.whole_seconds() as f64;

        let snapshot = StationSnapshot {
            name: self.manifest.title.clone(),
            created_on: now,
            duration_secs,
        };
        self.snapshots.push(snapshot);

        self.last_snapshot_time = now;
    }
}

impl Drop for Station {
    fn drop(&mut self) {
        // sinalizar a thread de producer de estado que deve finalizar
        if let Some(signal) = self._cancel_signal_sender.take() {
            let _ = signal.send(true);
        }
    }
}
