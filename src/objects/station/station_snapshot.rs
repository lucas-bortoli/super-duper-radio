use rocket::time::Date;

use crate::objects::{subscriber::Subscriber, track::track::Track};

pub struct StationSnapshot {
    pub name: String,
    pub current_track: Track,
    pub subscribers: Vec<Subscriber>,
    pub created_on: Date,
}
