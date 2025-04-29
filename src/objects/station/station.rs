use std::{fs::File, path};

use rocket::serde;

use crate::objects::{
    station::station_state::StationState, subscriber::Subscriber, track::track::Track,
};

pub struct Station {
    pub name: String,
    pub _subscribers: Vec<Subscriber>,
    pub path: String,
    pub frequency: f32,
    pub _state: Box<dyn StationState>,
    pub tracks: Vec<Track>,
}

impl Station {
    pub fn new(
        name: String,
        path: String,
        frequency: f32,
        _state: Box<dyn StationState>,
    ) -> Station {
        let mut station = Station {
            name,
            _subscribers: Vec::new(),
            path,
            frequency,
            _state,
            tracks: Vec::new(),
        };

        station.fill_tracks();

        station
    }

    pub fn add_subscriber(&mut self, subscriber: Subscriber) {
        self._subscribers.push(subscriber);
    }

    pub fn remove_subscriber(&mut self, subscriber: &Subscriber) {
        self._subscribers.retain(|s| s != subscriber);
    }

    pub fn change_state(&mut self, state: Box<dyn StationState>) {
        self._state = state;
    }

    pub fn get_music_files(&self) -> Vec<String> {
        let mut music_vec = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(file_name) = entry.path().to_str() {
                            music_vec.push(file_name.to_string());
                        }
                    }
                }
            }
        }

        music_vec
    }

    fn get_music_vec(&self) -> Vec<Track> {
        let binding = self.path.clone() + "manifest.json";
        let manifest_path = path::Path::new(&binding);

        let manifest_file: Vec<Track> =
            serde_json::from_reader(File::open(manifest_path).unwrap()).unwrap();

        manifest_file
    }

    fn fill_tracks(&mut self) {
        self.tracks = self.get_music_vec();
    }

    // Funções que acredito que vão estar no state
    pub fn go_next(&self) {
        // self._state.go_next();
    }

    pub fn play(&self) {
        // self._state.play();
    }
}
