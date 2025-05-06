use std::sync::mpsc::channel;
use std::{collections::HashMap, env, path::Path};

use bytes::Bytes;
use cytoplasm::cytoplasm::Cytoplasm;
use output_encoder::audio_encoder::OutputCodec;
use rocket::{
    http::ContentType,
    response::{content::RawHtml, stream::ByteStream},
};
use station::station::Station;
use track::track::{StationManifest, Track};

pub mod cytoplasm;
pub mod input_decoder;
pub mod output_encoder;
pub mod output_stream;
pub mod station;
pub mod track;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> RawHtml<&'static [u8]> {
    return RawHtml(b"<!DOCTYPE html>\n<audio controls src='/station'>");
}

struct StationCyto {
    pub cytoplasm: Cytoplasm,
    pub station: Station,
}

type StationMap = HashMap<String, StationCyto>;

#[get("/station")]
fn station_endpoint(state: &rocket::State<StationMap>) -> (ContentType, ByteStream![Bytes]) {
    let station = state.get("diamondcityradio").unwrap();
    let stream = station
        .cytoplasm
        .output_streams
        .get(&OutputCodec::Mp3_64kbps)
        .unwrap();

    stream.create_consumer_http_stream()
}

#[launch]
fn rocket() -> _ {
    let mut stations: StationMap = HashMap::new();

    for station_id in vec!["RadioZero"] {
        let (track_tx, track_rx) = channel::<Track>();
        let station_base_dir = Path::new(env::current_dir().unwrap().to_str().unwrap())
            .join("stations")
            .join(station_id);

        let manifest = StationManifest::from_base_dir(station_base_dir.clone())
            .expect("falha ao interpretar manifesto da estação");

        let station = Station::new(station_base_dir.clone(), manifest, track_tx);

        let cytoplasm = Cytoplasm::new(station_base_dir, &[OutputCodec::Mp3_64kbps], track_rx);

        stations.insert(station_id.to_owned(), StationCyto { cytoplasm, station });
    }

    rocket::build()
        .manage(stations)
        .mount("/", routes![index, station_endpoint])
}
