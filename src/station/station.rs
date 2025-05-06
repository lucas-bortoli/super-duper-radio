use rocket::time::OffsetDateTime;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use crate::track::{
    track::{StationManifest, Track},
    track_iterator::TrackIterator,
};

use super::{station_snapshot::StationSnapshot, station_state::StationState};

pub struct Station {
    pub base_dir: PathBuf,
    pub manifest: StationManifest,
    pub track_tx: Sender<Track>,

    pub state: StationState,
    pub snapshots: Vec<StationSnapshot>,
    pub current_track: Track,
    pub iterator: TrackIterator,
    last_snapshot_time: OffsetDateTime,
}

impl Station {
    pub fn new(base_dir: PathBuf, manifest: StationManifest, track_tx: Sender<Track>) -> Station {
        let iterator = TrackIterator::new(manifest.tracks.clone(), manifest.seed);
        let current_track = iterator.get_current().clone();
        let _ = track_tx.send(current_track.clone());
        let now = OffsetDateTime::now_utc();

        Station {
            base_dir,
            manifest,
            track_tx,
            state: StationState::Down,
            snapshots: Vec::new(),
            current_track,
            iterator,
            last_snapshot_time: now,
        }
    }

    pub fn play(&mut self) {
        let old = std::mem::replace(&mut self.state, StationState::Down);
        let new_state = old.play(self);
        self.state = new_state;
    }

    pub fn stop(&mut self) {
        let old = std::mem::replace(&mut self.state, StationState::Down);
        let new_state = old.stop(self);
        self.state = new_state;
    }

    pub fn next(&mut self) {
        let old = std::mem::replace(&mut self.state, StationState::Down);
        let new_state = old.next(self);
        self.state = new_state;

        let _ = self.track_tx.send(self.current_track.clone());
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

    /// Retorna o nome da estação somente para fins de debug
    pub fn state_name(&self) -> &str {
        self.state.name()
    }

    /// Notifica para o fluxo de áudio que a faixa mudou
    pub fn notify_track_change(&self) {
        // TODO: implementar integração com Cytoplasm para reload de arquivo
        println!(
            "Station[{}]: track mudou para {}",
            self.manifest.title, self.current_track.title
        );
    }

    /// Inicia playback da faixa atual (State Down -> Playing)
    pub fn start_playback(&self) {
        // TODO: integrar com Cytoplasm
        println!(
            "Station[{}]: iniciando playback de {}",
            self.manifest.title, self.current_track.title
        );
    }

    /// Para o playback (State Playing -> Down)
    pub fn stop_playback(&self) {
        // TODO: integrar com Cytoplasm
        println!("Station[{}]: parou playback", self.manifest.title);
    }
}
