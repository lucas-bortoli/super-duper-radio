use rocket::response::stream::{Event, EventStream};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
};
use tokio::sync::{
    broadcast::{self as tbroadcast, error::RecvError},
    oneshot,
};

use serde::Serialize;

#[derive(Clone, Serialize)]
pub enum Metadata {
    TrackChange { title: String, artist: String },
}

/// guarda as info de cada cliente conectado
struct ClientInfo {
    shutdown_tx: oneshot::Sender<()>, // canal pra mandar o sinal de desligar
}

pub struct MetadataOutputStream {
    // canal pra distribuir o audio pros clients
    tx: tbroadcast::Sender<Metadata>,
    // mapa de clientes ativos
    clients: Arc<Mutex<HashMap<usize, ClientInfo>>>,
    // gera os IDs únicos pros clients
    next_id: AtomicUsize,
}

impl MetadataOutputStream {
    /// cria um novo stream manager
    pub fn new() -> MetadataOutputStream {
        // canal com buffer de 24 mensagens
        // TODO: mexer nesse valor até ficar razoável. capacidade de 24 aguentou 301 clientes no meu PC
        let (tx, _) = tbroadcast::channel::<Metadata>(24);
        MetadataOutputStream {
            tx,
            clients: Arc::new(Mutex::new(HashMap::new())),
            next_id: AtomicUsize::new(0),
        }
    }

    /// Manda audio pra todos os clientes conectados
    pub fn push(&self, packet: Metadata) {
        let _ = self.tx.send(packet);

        // (se não tiver ninguém ouvindo, não tem problema, nada vai ocorrer)
    }

    /// Remover um cliente específico pelo ID
    pub fn terminate_client(&self, id: usize) {
        if let Some(info) = self.clients.lock().unwrap().remove(&id) {
            // sinalizar que a stream deve ser droppada
            let _ = info.shutdown_tx.send(());
            eprintln!("server_metadata: cliente {} foi removido", id);
        } else {
            eprintln!(
                "server_metadata: tentou matar o cliente {} que nem existe",
                id
            );
        }
    }

    /// Cria um novo stream de metadados pra um cliente
    pub fn create_consumer_sse_stream(&self) -> EventStream![] {
        // pega um ID novo pro cliente
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        // canal pra mandar o sinal de desligar
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // registra o cliente no mapa
        self.clients
            .lock()
            .unwrap()
            .insert(id, ClientInfo { shutdown_tx });

        let mut rx = self.tx.subscribe(); // cria um receptor pro canal de metadados

        // flag pra saber se terminou normalmente
        let normal_exit = Arc::new(AtomicBool::new(false));
        let exit_flag = Arc::clone(&normal_exit);
        let clients = Arc::clone(&self.clients);

        /// guardião que limpa tudo quando o stream acaba
        struct CleanupGuard {
            clients: Arc<Mutex<HashMap<usize, ClientInfo>>>,
            id: usize,
            exit_flag: Arc<AtomicBool>,
        }

        impl Drop for CleanupGuard {
            fn drop(&mut self) {
                if !self.exit_flag.load(Ordering::SeqCst) {
                    eprintln!("server_metadata({}): cliente caiu", self.id);
                }
                // remove o cliente do mapa automaticamente
                self.clients.lock().unwrap().remove(&self.id);
            }
        }

        // cria um guard pra esse stream, executado quando a stream deve ser droppada
        let guard = CleanupGuard {
            clients,
            id,
            exit_flag: exit_flag.clone(),
        };

        EventStream! {
            // mover receiver que fica ouvindo o sinal de desligar, e o guard da stream, pra cá
            let mut shutdown_rx = shutdown_rx;
            let _guard = guard;

            'receive: loop {
                tokio::select! {
                    // receber o próximo pacote de dados
                    result = rx.recv() => {
                        match result {
                            Ok(chunk) => {
                                yield Event::data(serde_json::to_string(&chunk).unwrap());
                            }
                            Err(err) => match err {
                                RecvError::Lagged(n) => {
                                    eprintln!(
                                        "server_metadata({}): cliente ficou {} mensagens atrasado - skip!",
                                        id, n
                                    );
                                },

                                // isso ocorre quando não há mais Sender para o canal, mas jamais deverá ocorrer na aplicação, já que as estações são permanentes e singletons
                                RecvError::Closed =>
                                    panic!("server_metadata({}): o canal de broadcast fechou do nada!", id)
                            },
                        }
                    }
                    // aguardar o sinal de desligar
                    _ = &mut shutdown_rx => {
                        eprintln!("server_metadata({}): sinal de shutdown para o cliente", id);
                        break 'receive;
                    }
                }
            }

            // marcar que terminou normalmente
            normal_exit.store(true, Ordering::SeqCst);
            eprintln!(
                "server_metadata({}): stream cliente acabou",
                id
            );
        }
    }
}
