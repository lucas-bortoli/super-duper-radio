use rocket::{
    http::ContentType,
    response::{
        content::{RawCss, RawHtml},
        stream::ByteStream,
    },
    tokio::sync::broadcast,
};

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    ops::DerefMut,
    sync::Arc,
    thread,
    time::Duration,
};

#[macro_use]
extern crate rocket;

// #[get("/")]
// fn index() -> RawHtml<&'static [u8]> {
//     return RawHtml(include_bytes!("ui.html"));
// }

// #[get("/ui.css")]
// fn stylesheet() -> RawCss<&'static [u8]> {
//     return RawCss(include_bytes!("ui.css"));
// }

// struct AudioBroadcaster {
//     sender: broadcast::Sender<Box<[u8; POLL_BUFFER_SIZE_BYTES]>>,
// }

#[get("/get_stations")]
fn get_stations() -> &'static str {
    "Rentorna a lista de estacoes ativas no momento!" 
}

#[post("/like/<station>")]
fn like(station: &str) -> (){
    // adiciona o like na track
}

#[get("/station/<station>")]
fn station(station: &str) -> &'static str {
 ""
}

// #[post("/send_message", data = "<message>")]
// fn send_message(message: Json<String>) -> String {
//     format!("message: {}", message.0)
// }

// #[post("/play")]
// fn play_music(state: &State<AudioBroadcaster>) -> &'static str {
//     "Reproduzindo música!"
// }

#[launch]
fn rocket() -> _ {
    // let (tx, _) = broadcast::channel::<Box<[u8; POLL_BUFFER_SIZE_BYTES]>>(8);

    // let broadcaster = Arc::new(AudioBroadcaster { sender: tx.clone() });

    // let station = Station::from_file("./diamond_city_radio/radio.yaml")
    // .expect("Failed to parse station file");
    // let station_thread_clone = station.clone();

    let station ="teste";

    rocket::build()
        // .manage(broadcaster)
        // .mount("/", routes![index, stylesheet])
        .mount("/station", routes![station])
}