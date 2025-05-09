use super::audio_file_info::AudioFileInfo;
use crate::track::audio_file_info;
use serde::Deserialize;
use std::{
    error::Error,
    fs::{self},
    path::PathBuf,
};

#[derive(Clone, Deserialize, Debug)]
pub struct Narration {
    pub source: String,
    pub transcript: String,

    #[serde(skip_deserializing)]
    pub file_info: AudioFileInfo,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub album_art: String,
    pub source: String,

    #[serde(default)]
    pub narration_before: Vec<Narration>,
    #[serde(default)]
    pub narration_after: Vec<Narration>,

    #[serde(skip_deserializing)]
    pub file_info: AudioFileInfo,
}

#[derive(Clone, Deserialize, Debug)]
pub struct StationManifest {
    pub title: String,
    pub description: String,
    pub seed: u64,
    pub tracks: Vec<Track>,
}

impl StationManifest {
    pub fn from_base_dir(base_dir: PathBuf) -> Result<StationManifest, Box<dyn Error>> {
        let manifest_location = base_dir.join("manifest.json");
        let manifest_data = fs::read_to_string(manifest_location)?;
        let mut manifest: StationManifest = serde_json::from_str(&manifest_data).unwrap();

        for track in manifest.tracks.iter_mut() {
            let file_source = base_dir.join(track.source.clone());
            let file_info = audio_file_info::query(file_source).unwrap_or_else(|_| {
                panic!(
                    "Erro ao extrair informações do arquivo da track: {:#?}",
                    track
                )
            });

            track.file_info = file_info;

            println!("Carregado informações para a track: {:#?}", track);
        }

        return Ok(manifest);
    }
}
