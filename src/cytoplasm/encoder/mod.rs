use bytes::Bytes;
use std::{
    io::{BufReader, BufWriter, Read, Write},
    process::{ChildStdin, Command, Stdio},
    sync::Arc,
    thread,
};

use super::{
    decoder::{self, AudioPacket},
    output_stream::audio_stream::AudioStream,
};

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum OutputCodec {
    Mp3_64kbps,
    Mp3_128kbps,
}

pub type ConsumerPacket = Bytes;

// singleton - um por estação
pub struct AudioEncoder {
    encoder_in: BufWriter<ChildStdin>,
    child: std::process::Child,
}

impl AudioEncoder {
    pub fn new(output_codec: &OutputCodec, output: Arc<AudioStream>) -> AudioEncoder {
        let args: Vec<String> = AudioEncoder::ffmpeg_args(&output_codec);

        println!("encoder: parâmetros ffmpeg: {:?}", args);

        let mut child = Command::new("ffmpeg")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("encoder: Falha ao spawnar o ffmpeg");

        if let Some(stdout) = child.stdout.take() {
            let mut stdout_reader = BufReader::new(stdout);
            thread::spawn(move || {
                println!("encoder: thread de consumidor de áudio iniciada.");

                let mut buf = vec![0u8; 8192];
                loop {
                    let n = stdout_reader
                        .read(&mut buf)
                        .expect("encoder: ler stdout do encoder falhou - processo crashou?");

                    match n {
                        0 => panic!("encoder: stdout finalizou, estação acabou!"),
                        1.. => {
                            // não é exatamente zero-copy, mas sim "one-copy"
                            // uma vez que alocamos esse Bytes, ele é reference-counted, igual o Arc
                            // ao transmití-lo pelo tokio::sync::broadcast::Sender ele não vai fazer novas cópias de memória
                            // então pagamos um custo fixo, uma vez só
                            let packet = Bytes::copy_from_slice(&buf[..n]);

                            output.push(packet);
                        }
                    }
                }
            });
        }

        let stdin = child.stdin.take().expect("encoder: falha ao ler stdin");
        let stdin_writer = BufWriter::new(stdin);

        AudioEncoder {
            encoder_in: stdin_writer,
            child: child,
        }
    }

    pub fn push_audio_packet(&mut self, packet: AudioPacket) {
        self.encoder_in
            .write(&packet.buffer)
            .expect("encoder: a fila do ffmpeg está cheia?");

        // bypass do buffer do stdin; manda direto pro ffmpeg, já que áudio é em real-time e talvez não seja legal ter esse comportamento de buffering
        // ignoramos o Result propositalmente, não há nenhuma ação cabível a ser tomada se o buffer de stdin não pode ser flushado - meio que não importa
        let _ = self.encoder_in.flush();
    }

    fn ffmpeg_args(output_codec: &OutputCodec) -> Vec<String> {
        let sample_rate = decoder::SAMPLE_RATE.to_string();
        let channel_count = decoder::CHANNEL_COUNT.to_string();

        let mut args = vec![
            "-f",
            "s16le",
            "-ar",
            &sample_rate,
            "-ac",
            &channel_count,
            "-i",
            "-", // stdin como input pro ffmpeg
        ];

        args.append(&mut match output_codec {
            OutputCodec::Mp3_64kbps => vec![
                "-b:a",
                "64k",
                "-f",
                "mp3",
                "-flush_packets",
                "1",
                "-write_xing",
                "0",
                "-id3v2_version",
                "0",
            ],
            OutputCodec::Mp3_128kbps => vec![
                "-b:a",
                "128k",
                "-f",
                "mp3",
                "-flush_packets",
                "1",
                "-write_xing",
                "0",
                "-id3v2_version",
                "0",
            ],
        });

        args.push("-"); // stdout como output pro ffmpeg

        return args.iter().map(|f| f.to_string()).collect();
    }
}

impl Drop for AudioEncoder {
    fn drop(&mut self) {
        self.child
            .kill()
            .expect("encoder: o ffmpeg não pôde ser fechado");
    }
}
