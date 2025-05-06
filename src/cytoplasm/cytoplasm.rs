use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::mpsc::Receiver,
    sync::{Arc, Mutex},
    thread::{self},
    time::{Duration, Instant},
};

use crate::{
    input_decoder::input_audio_file::{self, AudioPacket},
    output_encoder::audio_encoder::{AudioEncoder, OutputCodec},
    output_stream::OutputStream,
    track::track::Track,
};

const FUCKALL_DURATION: Duration = Duration::from_millis(5);
const SETPOINT_HIGH: usize = 10;
const SETPOINT_LOW: usize = 5;

pub struct Cytoplasm {
    encoders: Arc<Mutex<HashMap<OutputCodec, AudioEncoder>>>,
    pub output_streams: Arc<HashMap<OutputCodec, Arc<OutputStream>>>,
}

impl Cytoplasm {
    pub fn new(
        station_directory: PathBuf,
        output_codecs: &[OutputCodec],
        track_tx: Receiver<Track>,
    ) -> Cytoplasm {
        let buffer = Arc::new(Mutex::new(VecDeque::<AudioPacket>::new()));
        let output_streams = Self::init_output_streams(&output_codecs);
        let encoders = Self::init_encoders(&output_codecs, &output_streams);

        Self::init_decoder_thread(station_directory.clone(), buffer.clone(), track_tx);
        Self::init_encoder_thread(encoders.clone(), buffer.clone());

        let output_streams_arc = Arc::new(output_streams);

        Self::init_reporting_thread(output_streams_arc.clone());

        return Cytoplasm {
            output_streams: output_streams_arc,
            encoders,
        };
    }

    fn init_output_streams(codecs: &[OutputCodec]) -> HashMap<OutputCodec, Arc<OutputStream>> {
        let mut streams = HashMap::new();

        for codec in codecs {
            let stream = OutputStream::new(codec.clone());
            streams.insert(codec.clone(), Arc::new(stream));
        }

        streams
    }

    /// cria e inicializa um encoder de áudio para cada codec de saída solicitado
    fn init_encoders(
        codecs: &[OutputCodec],
        streams: &HashMap<OutputCodec, Arc<OutputStream>>,
    ) -> Arc<Mutex<HashMap<OutputCodec, AudioEncoder>>> {
        let mut encoders = HashMap::new();
        for codec in codecs {
            let output_stream = streams.get(codec).unwrap().clone();
            let encoder = AudioEncoder::new(&codec, output_stream);
            encoders.insert(codec.clone(), encoder);
        }
        Arc::new(Mutex::new(encoders))
    }

    /// inicia a thread responsável por decodificar arquivos de áudio
    /// ela carrega trilhas conforme definidas e enfileira pacotes no buffer compartilhado
    fn init_decoder_thread(
        station_directory: PathBuf,
        buffer: Arc<Mutex<VecDeque<AudioPacket>>>,
        track_rx: Receiver<Track>,
    ) {
        thread::spawn(move || {
            while let Ok(track) = track_rx.recv() {
                let file_path = station_directory.join(&track.source);
                eprintln!("cytoplasm/d: abrindo arquivo: {}", file_path.display());

                let mut file = input_audio_file::open_input_file_strategy(
                    file_path.to_string_lossy().into_owned(),
                );
                for packet in &mut file {
                    let mut buf = buffer.lock().unwrap();
                    if buf.len() >= SETPOINT_HIGH {
                        drop(buf);
                        while buffer.lock().unwrap().len() > SETPOINT_LOW {
                            thread::sleep(FUCKALL_DURATION);
                        }
                        buffer.lock().unwrap().push_back(packet);
                    } else {
                        buf.push_back(packet);
                    }
                }
            }
            eprintln!("cytoplasm/d: canal de tracks fechado");
        });
    }

    /// inicia a thread que consome pacotes do buffer, envia para os encoders e mantém o timing de reprodução
    fn init_encoder_thread(
        encoders: Arc<Mutex<HashMap<OutputCodec, AudioEncoder>>>,
        buffer: Arc<Mutex<VecDeque<AudioPacket>>>,
    ) {
        thread::spawn(move || loop {
            fn block_until_buffer_full(buffer: &Arc<Mutex<VecDeque<AudioPacket>>>) {
                // fazer porra nenhuma até o buffer estar cheio
                loop {
                    thread::sleep(FUCKALL_DURATION);
                    let guard = buffer.lock().unwrap();
                    if guard.len() >= SETPOINT_HIGH {
                        // finalmente buffer cheio; a outra thread deve ter printado "BACKPRESSURE!!"
                        eprintln!("cytoplasm/e: Buffering alcançado!");
                        break;
                    }
                }
            }

            // inicialmente vamos deixar o buffer encher completamente, antes de começar a consumi-lo
            // isso previne underruns durante o setup
            block_until_buffer_full(&buffer);

            let start = Instant::now();
            let mut playback_time = 0.0;

            loop {
                let mut buf_guard = buffer.lock().unwrap();
                if buf_guard.len() == 0 {
                    eprintln!("cytoplasm/e: Underrun...");
                    drop(buf_guard);
                    block_until_buffer_full(&buffer);
                } else {
                    // consumir todo o áudio da fila
                    let mut consumed_audio = Vec::new();
                    while buf_guard.len() > 0 {
                        // eprintln!("cytoplasm/e: consume...");
                        consumed_audio.push(buf_guard.pop_front().unwrap());
                    }

                    // liberar mutex para que possam continuar enfileirando pacotes na outra thread
                    drop(buf_guard);

                    // transmitir o áudio para todos os encoders, dar sleep
                    let mut encoders_guard = encoders.lock().unwrap();
                    for packet in consumed_audio {
                        playback_time += packet.audio_length;
                        for encoder in encoders_guard.values_mut() {
                            encoder.push_audio_packet(packet.clone());
                        }
                    }
                    drop(encoders_guard);

                    // ao calcular o "next_time" com base em um start_time fixo, garantimos que pequenos atrasos não se acumulem ao longo do tempo.
                    // usar apenas thread::sleep() pela duração de cada packet causaria desvios cumulativos, já que o tempo de execução de cada iteração varia.
                    // assim, mesmo que uma iteração atrase um pouco, a próxima tentará se alinhar com o tempo real correto.
                    let next_time = start + Duration::from_secs_f64(playback_time);
                    let now = Instant::now();
                    if next_time > now {
                        thread::sleep(next_time - now);
                    } else {
                        eprintln!("cytoplasm/e: Time underrun...");
                    }
                }
            }
        });
    }

    fn init_reporting_thread(streams: Arc<HashMap<OutputCodec, Arc<OutputStream>>>) {
        thread::spawn(move || {
            let mut last_bytes = HashMap::new();
            let mut last_time = Instant::now();

            loop {
                for (codec, stream) in streams.iter() {
                    let mut bytes_total = 0usize;

                    for (bytes, _) in stream.get_bandwidth_stats().values() {
                        bytes_total += bytes;
                    }

                    let elapsed_secs = last_time.elapsed().as_secs_f64();
                    let kbps = if let Some(prev_bytes) = last_bytes.get(codec) {
                        let delta_bytes = bytes_total.saturating_sub(*prev_bytes) as f64;
                        delta_bytes / (1024.0 * elapsed_secs)
                    } else {
                        0.0
                    };

                    last_bytes.insert(codec.clone(), bytes_total);

                    eprintln!(
                        "cytoplasm: stats: {} clientes, {:.2} KB enviados, {:.2} kb/s",
                        stream.list_clients().len(),
                        bytes_total as f64 / 1024.0,
                        kbps
                    );
                }

                last_time = Instant::now();
                thread::sleep(Duration::from_secs(2));
            }
        });
    }
}
