use rocket::time::OffsetDateTime;
use serde_json;
use std::sync::mpsc::Sender;
use std::{fs::File, path::Path};

use crate::objects::{
    station::{
        station_snapshot::StationSnapshot,
        station_state::{DownState, StationState},
    },
    subscriber::Subscriber,
    track::{self, track::Track, track_iterator::TrackIterator},
};

pub struct Station {
    pub name: String,
    pub subscribers: Vec<Subscriber>,
    pub path: String,
    pub frequency: f32,
    pub state: Box<dyn StationState>,
    pub snapshots: Vec<StationSnapshot>,
    pub current_track: Track,
    pub iterator: TrackIterator,
    pub track_tx: Sender<Track>,
    last_snapshot_time: OffsetDateTime,
}

impl Station {
    pub fn new(
        name: String,
        path: String,
        frequency: f32,
        seed: u64,
        track_tx: Sender<Track>,
    ) -> Station {
        let metadata_path = Path::new(&path).join("metadata.json");
        let file = File::open(&metadata_path).expect("Station: Falha ao abrir metadata.json");
        let tracks: Vec<Track> = serde_json::from_reader(file).expect("Station: JSON inválido");

        let iterator = TrackIterator::new(tracks.clone(), seed);
        let current_track = iterator.get_current().clone();
        let _ = track_tx.send(current_track.clone());
        let now = OffsetDateTime::now_utc();

        Station {
            name,
            subscribers: Vec::new(),
            path,
            frequency,
            state: Box::new(DownState::new()),
            snapshots: Vec::new(),
            current_track,
            iterator,
            track_tx,
            last_snapshot_time: now,
        }
    }

    pub fn play(&mut self) {
        let old = std::mem::replace(&mut self.state, Box::new(DownState::new()));
        let new_state = old.play(self);
        self.state = new_state;
    }

    pub fn stop(&mut self) {
        let old = std::mem::replace(&mut self.state, Box::new(DownState::new()));
        let new_state = old.stop(self);
        self.state = new_state;
    }

    pub fn next(&mut self) {
        let old = std::mem::replace(&mut self.state, Box::new(DownState::new()));
        let new_state = old.next(self);
        self.state = new_state;

        let _ = self.track_tx.send(self.current_track.clone());
    }

    pub fn save_snapshot(&mut self) {
        let now = OffsetDateTime::now_utc();
        let delta = now - self.last_snapshot_time;
        let duration_secs = delta.whole_seconds() as f64;

        let snapshot = StationSnapshot {
            name: self.name.clone(),
            current_track: self.current_track.clone(),
            subscribers: self.subscribers.clone(),
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
            self.name, self.current_track.title
        );
    }

    /// Inicia playback da faixa atual (State Down -> Playing)
    pub fn start_playback(&self) {
        // TODO: integrar com Cytoplasm
        println!(
            "Station[{}]: iniciando playback de {}",
            self.name, self.current_track.title
        );
    }

    /// Para o playback (State Playing -> Down)
    pub fn stop_playback(&self) {
        // TODO: integrar com Cytoplasm
        println!("Station[{}]: parou playback", self.name);
    }
}
