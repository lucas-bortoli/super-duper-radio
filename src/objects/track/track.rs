use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct Narration {
    pub source: String,
    pub transcript: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Track {
    pub source: String,
    pub title: String,
    pub artist: String,
    pub album_art: String,
    #[serde(default)]
    pub narration_before: Vec<Narration>,
    #[serde(default)]
    pub narration_after: Vec<Narration>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Playlist {
    pub title: String,
    pub description: String,
    pub tracks: Vec<Track>,
}
