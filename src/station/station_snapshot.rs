use rocket::time::OffsetDateTime;

use crate::track::track::Track;

pub struct StationSnapshot {
    pub name: String,
    pub current_track: Track,
    pub created_on: OffsetDateTime,
    pub duration_secs: f64,
}
