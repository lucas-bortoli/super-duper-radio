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

#[get("/")]
fn index() -> RawHtml<&'static [u8]> {
    return RawHtml(include_bytes!("ui/ui.html"));
}

#[get("/ui.css")]
fn stylesheet() -> RawCss<&'static [u8]> {
    return RawCss(include_bytes!("ui/ui.css"));
}

#[get("/app.js")]
fn javascript() -> (ContentType, &'static [u8]) {
    (ContentType::JavaScript, include_bytes!("ui/app.js"))
}

#[get("/get_stations")]
fn get_stations() -> &'static str {
    "Retorna a lista de estacoes ativas no momento!" 
}

#[launch]
fn rocket() -> _ {
    // let (tx, _) = broadcast::channel::<Box<[u8; POLL_BUFFER_SIZE_BYTES]>>(8);

    // let broadcaster = Arc::new(AudioBroadcaster { sender: tx.clone() });

    // let station = Station::from_file("./diamond_city_radio/radio.yaml")
    // .expect("Failed to parse station file");
    // let station_thread_clone = station.clone();

    let station ="teste";

    rocket::build()
        .mount("/", routes![index, stylesheet, javascript])     
        .mount("/", routes![])
}

// sobe