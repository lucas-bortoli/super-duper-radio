use rocket::time::OffsetDateTime;

pub struct StationSnapshot {
    pub name: String,
    pub created_on: OffsetDateTime,
    pub duration_secs: f64,
}
