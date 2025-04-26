use std::{fs::File, path::Path};
use rocket::time::OffsetDateTime;
use serde_json;

use crate::objects::{
    station::station_state::StationState,
    station::station_state::DownState,
    station::station_snapshot::StationSnapshot,
    subscriber::Subscriber,
    track::track::Track,
    track::track_iterator::TrackIterator,
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
}


impl Station {
    pub fn new(
        name: String,
        path: String,
        frequency: f32,
        seed: u64,
    ) -> Station {
        let metadata_path = Path::new(&path).join("metadata.json");
        let file = File::open(&metadata_path)
            .expect("Station: Falha ao abrir metadata.json");
        let tracks: Vec<Track> = serde_json::from_reader(file)
            .expect("Station: JSON inválido");

        let iterator = TrackIterator::new(tracks.clone(), seed);
        let current_track = iterator.get_current().clone();

        Station {
            name,
            subscribers: Vec::new(),
            path,
            frequency,
            state: Box::new(DownState::new()),
            snapshots: Vec::new(),
            current_track,
            iterator,
        }
    }

    pub fn add_subscriber(&mut self, subscriber: Subscriber) {
        self.subscribers.push(subscriber);
    }

    pub fn remove_subscriber(&mut self, subscriber: &Subscriber) {
        self.subscribers.retain(|s| s != subscriber);
    }

    pub fn change_state(&mut self, new_state: Box<dyn StationState>) {
        self.state = new_state;
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
    }

    pub fn save_snapshot(&mut self) {
        let snapshot = StationSnapshot {
            name: self.name.clone(),
            current_track: self.current_track.clone(),
            subscribers: self.subscribers.clone(),
            created_on: OffsetDateTime::now_utc().date(),
        };
        self.snapshots.push(snapshot);
    }


    /// Retorna o nome da estação somente para fins de debug
    pub fn state_name(&self) -> &str {
        self.state.name()
    }

    /// Notifica para o fluxo de áudio que a faixa mudou
    pub fn notify_track_change(&self) {
        // TODO: implementar integração com Cytoplasm para reload de arquivo
        println!("Station[{}]: track mudou para {}", self.name, self.current_track.title);
    }

    /// Inicia playback da faixa atual (State Down -> Playing)
    pub fn start_playback(&self) {
        // TODO: integrar com Cytoplasm
        println!("Station[{}]: iniciando playback de {}", self.name, self.current_track.title);
    }

    /// Para o playback (State Playing -> Down)
    pub fn stop_playback(&self) {
        // TODO: integrar com Cytoplasm
        println!("Station[{}]: parou playback", self.name);
    }

}