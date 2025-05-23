use std::{
    collections::{HashMap, VecDeque},
    sync::{mpsc, Arc, Mutex},
    thread::{self},
    time::{Duration, Instant},
};

use decoder::{AudioPacket, InputFile};
use encoder::{AudioEncoder, OutputCodec};
use output_stream::{
    audio_stream::AudioStream,
    metadata_stream::{Metadata, MetadataStream},
};
use state::{State, StateManager};

use crate::track::track::StationManifest;

pub mod decoder;
pub mod encoder;
pub mod output_stream;
pub mod state;

const BACKPRESSURE_DELAY: Duration = Duration::from_millis(5);
const SETPOINT_HIGH: usize = 20;
const SETPOINT_LOW: usize = 10;

pub struct Cytoplasm {
    pub manifest: StationManifest,
    pub state_manager: StateManager,
    pub encoders: Arc<Mutex<HashMap<OutputCodec, AudioEncoder>>>,
    pub output_streams: Arc<HashMap<OutputCodec, Arc<AudioStream>>>,
    pub output_metadata_stream: Arc<MetadataStream>,
}

impl Cytoplasm {
    pub fn new(manifest: StationManifest, output_codecs: &[OutputCodec]) -> Cytoplasm {
        let (state_manager, state_rx) = StateManager::new(manifest.tracks.clone(), manifest.seed);
        let buffer = Arc::new(Mutex::new(VecDeque::<AudioPacket>::new()));
        let output_streams = Self::init_output_streams(&output_codecs);
        let output_metadata_stream = Arc::new(MetadataStream::new());
        let encoders = Self::init_encoders(&output_codecs, &output_streams);

        Self::init_decoder_thread(state_rx, buffer.clone(), output_metadata_stream.clone());
        Self::init_encoder_thread(encoders.clone(), buffer.clone());

        let output_streams_arc = Arc::new(output_streams);

        Self::init_reporting_thread(output_streams_arc.clone());

        return Cytoplasm {
            manifest,
            state_manager,
            output_streams: output_streams_arc,
            output_metadata_stream,
            encoders,
        };
    }

    fn init_output_streams(codecs: &[OutputCodec]) -> HashMap<OutputCodec, Arc<AudioStream>> {
        let mut streams = HashMap::new();

        for codec in codecs {
            let stream = AudioStream::new(codec.clone());
            streams.insert(codec.clone(), Arc::new(stream));
        }

        streams
    }

    /// cria e inicializa um encoder de áudio para cada codec de saída solicitado
    fn init_encoders(
        codecs: &[OutputCodec],
        streams: &HashMap<OutputCodec, Arc<AudioStream>>,
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
    /// ela carrega trilhas conforme recebidas e enfileira pacotes no buffer compartilhado
    fn init_decoder_thread(
        state_rx: mpsc::Receiver<State>,
        buffer: Arc<Mutex<VecDeque<AudioPacket>>>,
        metadata_stream: Arc<MetadataStream>,
    ) {
        thread::spawn(move || loop {
            eprintln!("cytoplasm/d: aguardando próximo estado da estação...");

            fn play_audio_blocking(file: InputFile, buffer: Arc<Mutex<VecDeque<AudioPacket>>>) {
                for packet in file {
                    let mut buf = buffer.lock().unwrap();
                    if buf.len() >= SETPOINT_HIGH {
                        drop(buf);
                        while buffer.lock().unwrap().len() > SETPOINT_LOW {
                            thread::sleep(BACKPRESSURE_DELAY);
                        }
                        buffer.lock().unwrap().push_back(packet);
                    } else {
                        buf.push_back(packet);
                    }
                }
            }

            fn play_silence(units: u8, buffer: Arc<Mutex<VecDeque<AudioPacket>>>) {
                let silence_unit: AudioPacket =
                    AudioPacket::from_silence(Duration::from_millis(500));
                for _ in 0..units {
                    let mut buf = buffer.lock().unwrap();
                    if buf.len() >= SETPOINT_HIGH {
                        drop(buf);
                        while buffer.lock().unwrap().len() > SETPOINT_LOW {
                            thread::sleep(BACKPRESSURE_DELAY);
                        }
                        buffer.lock().unwrap().push_back(silence_unit.clone());
                    } else {
                        buf.push_back(silence_unit.clone());
                    }
                }
            }

            let current_state = state_rx.recv();
            if let Err(err) = current_state {
                eprintln!("cytoplasm/d: o canal de state fechou: {}", err);
                break;
            }

            let current_state = current_state.unwrap();
            eprintln!("cytoplasm/d: estado atual: {}", current_state);

            match current_state {
                State::SwitchTrack => continue, // estação ainda está inicializando, ignorar
                State::NarrationBefore {
                    track: _,
                    narration,
                } => {
                    let file = InputFile::new(narration.file_info.location, 0);
                    play_audio_blocking(file, buffer.clone());
                }
                State::Track { track } => {
                    metadata_stream.push(Metadata::TrackChange {
                        title: track.title,
                        artist: track.artist,
                    });

                    let file = InputFile::new(track.file_info.location, 0);
                    play_audio_blocking(file, buffer.clone());
                }
                State::NarrationAfter {
                    track: _,
                    narration,
                } => {
                    let file = InputFile::new(narration.file_info.location, 0);
                    play_audio_blocking(file, buffer.clone());
                }
                State::IntentionalDelay {
                    duration_units,
                    next_state: _,
                } => {
                    play_silence(duration_units, buffer.clone());
                }
            }
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
                    thread::sleep(BACKPRESSURE_DELAY);
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

    fn init_reporting_thread(streams: Arc<HashMap<OutputCodec, Arc<AudioStream>>>) {
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
