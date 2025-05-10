use bytes::Bytes;
use rocket::{http::ContentType, response::stream::ByteStream};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Instant,
};
use tokio::sync::{
    broadcast::{self as tbroadcast, error::RecvError},
    oneshot,
};

use crate::{
    cytoplasm::encoder::OutputCodec,
    id_gen::{generate_id, UniqueId},
};

mod null_frames;

/// guarda as info de cada cliente conectado
struct ClientInfo {
    shutdown_tx: oneshot::Sender<()>, // canal pra mandar o sinal de desligar
    bytes_sent: Arc<AtomicUsize>,     // contador de bytes enviados (thread-safe)
    connected_at: Instant,            // quando o cliente conectou
}

pub struct OutputStream {
    // codec de audio que a gente usa
    codec: OutputCodec,
    // canal pra distribuir o audio pros clients
    tx: tbroadcast::Sender<Bytes>,
    // mapa de clientes ativos
    clients: Arc<Mutex<HashMap<UniqueId, ClientInfo>>>,
}

impl OutputStream {
    /// cria um novo stream manager
    pub fn new(codec: OutputCodec) -> OutputStream {
        // canal com buffer de 24 mensagens
        // TODO: mexer nesse valor até ficar razoável. capacidade de 24 aguentou 301 clientes no meu PC
        let (tx, _) = tbroadcast::channel::<Bytes>(24);
        OutputStream {
            codec,
            tx,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Manda audio pra todos os clientes conectados
    pub fn push(&self, packet: Bytes) {
        let _ = self.tx.send(packet);

        // (se não tiver ninguém ouvindo, não tem problema, nada vai ocorrer)
    }

    /// Remover um cliente específico pelo ID
    pub fn terminate_client(&self, id: usize) {
        if let Some(info) = self.clients.lock().unwrap().remove(&id) {
            // sinalizar que a stream deve ser droppada
            let _ = info.shutdown_tx.send(());
            eprintln!("server: cliente {} foi removido", id);
        } else {
            eprintln!("server: tentou matar o cliente {} que nem existe", id);
        }
    }

    /// Estatísticas de bandwidth de todos os clients
    pub fn get_bandwidth_stats(&self) -> HashMap<usize, (usize, f64)> {
        let clients = self.clients.lock().unwrap();
        let now = Instant::now();

        clients
            .iter()
            .map(|(id, info)| {
                let bytes = info.bytes_sent.load(Ordering::Relaxed); // bytes totais
                let tempo = now.duration_since(info.connected_at).as_secs_f64();
                // calcula os bits por segundo (bps)
                let bps = if tempo > 0.0 {
                    (bytes as f64 * 8.0) / tempo // bits = bytes * 8
                } else {
                    0.0 // evita divisão por zero
                };
                (*id, (bytes, bps))
            })
            .collect()
    }

    /// Lista os IDs de todos os clients conectados
    pub fn list_clients(&self) -> Vec<usize> {
        self.clients.lock().unwrap().keys().copied().collect()
    }

    /// Cria um novo stream de audio pra um cliente
    pub fn create_consumer_http_stream(&self) -> (ContentType, ByteStream![Bytes]) {
        let this_id = generate_id();

        // canal pra mandar o sinal de desligar
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        // contador de bytes enviados
        let bytes_sent = Arc::new(AtomicUsize::new(0));

        // registra o cliente no mapa
        self.clients.lock().unwrap().insert(
            this_id,
            ClientInfo {
                shutdown_tx,
                bytes_sent: Arc::clone(&bytes_sent),
                connected_at: Instant::now(), // marca o horário que conectou
            },
        );

        let codec = self.codec.clone();
        let mut rx = self.tx.subscribe(); // cria um receptor pro canal de audio

        // flag pra saber se terminou normalmente
        let normal_exit = Arc::new(AtomicBool::new(false));
        let exit_flag = Arc::clone(&normal_exit);
        let clients = Arc::clone(&self.clients);

        /// guardião que limpa tudo quando o stream acaba
        struct CleanupGuard {
            clients: Arc<Mutex<HashMap<usize, ClientInfo>>>,
            id: usize,
            exit_flag: Arc<AtomicBool>,
            bytes_sent: Arc<AtomicUsize>,
        }

        impl Drop for CleanupGuard {
            fn drop(&mut self) {
                if !self.exit_flag.load(Ordering::SeqCst) {
                    eprintln!(
                        "server({}): cliente caiu - enviou {} bytes no total",
                        self.id,
                        self.bytes_sent.load(Ordering::Relaxed)
                    );
                }
                // remove o cliente do mapa automaticamente
                self.clients.lock().unwrap().remove(&self.id);
            }
        }

        // cria um guard pra esse stream, executado quando a stream deve ser droppada
        let guard = CleanupGuard {
            clients,
            id: this_id,
            exit_flag: exit_flag.clone(),
            bytes_sent: Arc::clone(&bytes_sent),
        };

        let stream = ByteStream! {
            // mover receiver que fica ouvindo o sinal de desligar, e o guard da stream, pra cá
            let mut shutdown_rx = shutdown_rx;
            let _guard = guard;

            // manda o frame null inicial
            let null_frame = Bytes::from(null_frames::get_null_frame(&codec));
            let null_size = null_frame.len();
            bytes_sent.fetch_add(null_size, Ordering::Relaxed);
            yield null_frame;
            eprintln!(
                "server({}): mandou frame null ({} bytes) para o cliente",
                this_id, null_size
            );

            'receive: loop {
                tokio::select! {
                    // receber o próximo pacote de dados
                    result = rx.recv() => {
                        match result {
                            Ok(chunk) => {
                                let size = chunk.len();
                                bytes_sent.fetch_add(size, Ordering::Relaxed);  // atualiza contador de I/O
                                yield chunk;
                            }
                            Err(err) => match err {
                                RecvError::Lagged(n) => {
                                    eprintln!(
                                        "server({}): cliente ficou {} mensagens atrasado - skip!",
                                        this_id, n
                                    );
                                },

                                // isso ocorre quando não há mais Sender para o canal, mas jamais deverá ocorrer na aplicação, já que as estações são permanentes e singletons
                                RecvError::Closed =>
                                    panic!("server({}): o canal de broadcast fechou do nada!", this_id)
                            },
                        }
                    }
                    // aguardar o sinal de desligar
                    _ = &mut shutdown_rx => {
                        eprintln!("server({}): sinal de shutdown para o cliente", this_id);
                        break 'receive;
                    }
                }
            }

            // marcar que terminou normalmente
            normal_exit.store(true, Ordering::SeqCst);
            let total_bytes = bytes_sent.load(Ordering::Relaxed);
            eprintln!(
                "server({}): stream cliente acabou ({} bytes no total)",
                this_id, total_bytes
            );
        };

        (
            ContentType::new("audio", OutputStream::get_mime_type(&self.codec)),
            stream,
        )
    }

    fn get_mime_type(codec: &OutputCodec) -> &'static str {
        match codec {
            OutputCodec::Mp3_64kbps => "mpeg",
            OutputCodec::Mp3_128kbps => "mpeg",
        }
    }
}
