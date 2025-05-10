use rocket::response::stream::{Event, EventStream};
use tokio::sync::broadcast::{self as tbroadcast};

use serde::Serialize;

#[derive(Clone, Serialize)]
pub enum Metadata {
    TrackChange { title: String, artist: String },
}

pub struct MetadataStream {
    // canal pra distribuir o audio pros clients
    tx: tbroadcast::Sender<Metadata>,
}

impl MetadataStream {
    /// cria um novo stream manager
    pub fn new() -> MetadataStream {
        let (tx, _) = tbroadcast::channel::<Metadata>(4);
        MetadataStream { tx }
    }

    /// Manda um pacote de metadados pra todos os clientes conectados
    pub fn push(&self, packet: Metadata) {
        let _ = self.tx.send(packet);
    }

    /// Cria um novo stream de metadados pra um cliente
    pub fn create_consumer_sse_stream(&self) -> EventStream![] {
        let mut rx = self.tx.subscribe(); // cria um receptor pro canal de metadados

        EventStream! {
            while let Ok(item) = rx.recv().await {
                yield Event::data(serde_json::to_string(&item).unwrap());
            }
        }
    }
}
