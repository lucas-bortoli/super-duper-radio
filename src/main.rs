use std::net::IpAddr;
use std::str::FromStr;
use std::{collections::HashMap, env, path::Path};

use bytes::Bytes;
use cytoplasm::encoder::OutputCodec;
use cytoplasm::Cytoplasm;
use rocket::response::stream::EventStream;
use rocket::{
    fs::{relative, FileServer},
    http::ContentType,
    response::{content::RawHtml, stream::ByteStream},
};
use track::track::StationManifest;

pub mod cytoplasm;
pub mod id_gen;
mod process_priority;
pub mod track;

#[macro_use]
extern crate rocket;

type StationMap = HashMap<String, Cytoplasm>;

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

#[get("/station/64")]
fn station_endpoint_64(state: &rocket::State<StationMap>) -> (ContentType, ByteStream![Bytes]) {
    let station = state.get("RadioZero").unwrap();
    let stream = station
        .output_streams
        .get(&OutputCodec::Mp3_64kbps)
        .unwrap();

    stream.create_consumer_http_stream()
}

#[get("/station/128")]
fn station_endpoint_128(state: &rocket::State<StationMap>) -> (ContentType, ByteStream![Bytes]) {
    let station = state.get("RadioZero").unwrap();
    let stream = station
        .output_streams
        .get(&OutputCodec::Mp3_128kbps)
        .unwrap();

    stream.create_consumer_http_stream()
}

#[get("/station/events")]
fn station_event_endpoint(state: &rocket::State<StationMap>) -> EventStream![] {
    let station = state.get("RadioZero").unwrap();
    let stream = station.output_metadata_stream.clone();

    stream.create_consumer_sse_stream()
}

#[launch]
fn rocket() -> _ {
    process_priority::set_high_priority();

    let mut stations: StationMap = HashMap::new();

    for station_id in vec!["RadioZero"] {
        let station_base_dir = Path::new(env::current_dir().unwrap().to_str().unwrap())
            .join("stations")
            .join(station_id);

        let manifest = StationManifest::from_base_dir(station_base_dir.clone())
            .expect("falha ao interpretar manifesto da estação");

        let cytoplasm = Cytoplasm::new(
            manifest,
            &[OutputCodec::Mp3_64kbps, OutputCodec::Mp3_128kbps],
        );

        stations.insert(station_id.to_owned(), cytoplasm);
    }

    let address = env::var("ROCKET_ADDRESS").unwrap_or("127.0.0.1".to_string());

    let config = rocket::Config {
        address: IpAddr::from_str(&address).unwrap(),
        ..rocket::Config::debug_default()
    };

    rocket::custom(&config)
        .manage(stations)
        .mount(
            "/",
            routes![
                index,
                stylesheet,
                javascript,
                favicon,
                get_stations,
                station_endpoint_64,
                station_endpoint_128,
                station_event_endpoint
            ],
        )
        .mount(
            "/backgrounds",
            FileServer::from(relative!("src/ui/backgrounds")),
        )
}
