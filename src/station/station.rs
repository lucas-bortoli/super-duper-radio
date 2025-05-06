use rocket::time::OffsetDateTime;
use std::sync::mpsc::SyncSender;
use std::{path::PathBuf, thread};
use tokio::sync::oneshot;

use crate::{
    cytoplasm::cytoplasm::Cytoplasm,
    track::{
        track::{StationManifest, Track},
        track_iterator::TrackIterator,
    },
};

use super::{station_snapshot::StationSnapshot, station_state::StationState};

pub struct Station {
    pub base_dir: PathBuf,
    pub manifest: StationManifest,
    pub track_tx: SyncSender<Track>,
    pub cytoplasm: Cytoplasm,

    pub state: StationState,
    pub snapshots: Vec<StationSnapshot>,
    pub current_track: Track,
    pub iterator: TrackIterator,
    last_snapshot_time: OffsetDateTime,

    _cancel_signal_sender: Option<oneshot::Sender<bool>>,
}

impl Station {
    pub fn new(
        base_dir: PathBuf,
        manifest: StationManifest,
        cytoplasm: Cytoplasm,
        track_tx: SyncSender<Track>,
    ) -> Station {
        let iterator = TrackIterator::new(manifest.tracks.clone(), manifest.seed);
        let current_track = iterator.get_current().clone();
        let _ = track_tx.send(current_track.clone());
        let now = OffsetDateTime::now_utc();

        let (cancel_signal_sender, mut cancel_signal_receiver) = oneshot::channel::<bool>();
        thread::spawn(move || {
            let mut current_state = StationState::Initial;

            'state_loop: loop {
                // recebemos o sinal de parada?
                match cancel_signal_receiver.try_recv() {
                    Ok(_) => break 'state_loop,
                    Err(oneshot::error::TryRecvError::Closed) => break 'state_loop,
                    _ => {}
                }

                let next_state = current_state.clone();

                match current_state {
                    StationState::Initial => {}
                    StationState::Narration => {}
                    StationState::Track => {}
                }

                eprintln!("Station: {} -> {}", current_state, next_state);

                current_state = next_state;
            }
        });

        Station {
            base_dir,
            manifest,
            track_tx,
            cytoplasm,
            state: StationState::Initial,
            snapshots: Vec::new(),
            current_track,
            iterator,
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
            current_track: self.current_track.clone(),
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
