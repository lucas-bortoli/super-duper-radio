use std::sync::mpsc::sync_channel;
use std::{collections::HashMap, env, path::Path};

use bytes::Bytes;
use cytoplasm::Cytoplasm;
use output_encoder::audio_encoder::OutputCodec;
use rocket::response::stream::EventStream;
use rocket::{
    fs::{relative, FileServer},
    http::ContentType,
    response::{content::RawHtml, stream::ByteStream},
};
use station::station::Station;
use station::station_state::StationState;
use track::track::StationManifest;

pub mod cytoplasm;
pub mod id_gen;
pub mod input_decoder;
pub mod output_encoder;
pub mod output_stream;
pub mod station;
pub mod track;

#[macro_use]
extern crate rocket;

type StationMap = HashMap<String, Station>;

#[get("/")]
fn index() -> RawHtml<&'static [u8]> {
    RawHtml(include_bytes!("ui/ui.html"))
}

#[get("/ui.css")]
fn stylesheet() -> (ContentType, &'static [u8]) {
    (ContentType::CSS, include_bytes!("ui/ui.css"))
}

#[get("/app.js")]
fn javascript() -> (ContentType, &'static [u8]) {
    (ContentType::JavaScript, include_bytes!("ui/app.js"))
}

#[get("/favicon.ico")]
fn favicon() -> (ContentType, &'static [u8]) {
    (ContentType::Icon, include_bytes!("ui/favicon.ico"))
}

#[get("/get_stations")]
fn get_stations() -> &'static str {
    "Retorna a lista de estacoes ativas no momento!"
}

#[get("/station")]
fn station_endpoint(state: &rocket::State<StationMap>) -> (ContentType, ByteStream![Bytes]) {
    let station = state.get("RadioZero").unwrap();
    let stream = station
        .cytoplasm
        .output_streams
        .get(&OutputCodec::Mp3_64kbps)
        .unwrap();

    stream.create_consumer_http_stream()
}

#[get("/station/events")]
fn station_event_endpoint(state: &rocket::State<StationMap>) -> EventStream![] {
    let station = state.get("RadioZero").unwrap();
    let stream = station.metadata_stream.clone();

    stream.create_consumer_sse_stream()
}

#[launch]
fn rocket() -> _ {
    let mut stations: StationMap = HashMap::new();

    for station_id in vec!["RadioZero"] {
        let (state_tx, state_rx) = sync_channel::<StationState>(0);
        let station_base_dir = Path::new(env::current_dir().unwrap().to_str().unwrap())
            .join("stations")
            .join(station_id);

        let manifest = StationManifest::from_base_dir(station_base_dir.clone())
            .expect("falha ao interpretar manifesto da estação");

        let cytoplasm = Cytoplasm::new(&[OutputCodec::Mp3_64kbps], state_rx);
        let station = Station::new(station_base_dir, manifest, cytoplasm, state_tx);

        stations.insert(station_id.to_owned(), station);
    }

    rocket::build()
        .manage(stations)
        .mount(
            "/",
            routes![
                index,
                stylesheet,
                javascript,
                favicon,
                get_stations,
                station_endpoint,
                station_event_endpoint
            ],
        )
        .mount(
            "/backgrounds",
            FileServer::from(relative!("src/ui/backgrounds")),
        )
}
