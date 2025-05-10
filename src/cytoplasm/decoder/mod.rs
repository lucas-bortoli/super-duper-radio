use std::{
    io::{BufReader, Read},
    process::{Child, ChildStdout, Command, Stdio},
};

use bytes::Bytes;

pub const CHANNEL_COUNT: u32 = 2;
pub const SAMPLE_RATE: u32 = 44100;
pub const BYTE_DEPTH: u32 = 2; //16bits
pub const FFMPEG_STDOUT_BUFFER_SIZE: u32 = 1 * (SAMPLE_RATE * CHANNEL_COUNT * BYTE_DEPTH); // 1 segundo de áudio

#[derive(Debug, Clone)]
pub struct AudioPacket {
    /**
     * Quanto tempo de áudio este quadro tem, em segundos
     */
    pub audio_length: f64,

    /**
     * O buffer de áudio, formato PCM, com as especificações acima.
     */
    pub buffer: Bytes,
}

pub struct InputFile {
    child: Child,
    reader: BufReader<ChildStdout>,
}

impl InputFile {
    pub fn new(file_path: String, seek_ms: u64) -> InputFile {
        let mut child = Command::new("ffmpeg")
            .args(&[
                "-i",
                &file_path,
                "-f",
                "s16le",
                "-ac",
                &CHANNEL_COUNT.to_string(),
                "-ar",
                &SAMPLE_RATE.to_string(),
                "-",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("input_file: falha ao spawnar o ffmpeg");

        let stdout = child
            .stdout
            .take()
            .expect("input_file: falha ao ler stdout");
        let reader = BufReader::new(stdout);

        InputFile { reader, child }
    }

    /// Converte o número de bytes de um buffer PCM para sua duração em segundos
    pub fn calculate_buffer_length(buffer_capacity_bytes: u32) -> f64 {
        let bytes_per_sample = CHANNEL_COUNT * BYTE_DEPTH;
        let samples_per_second = SAMPLE_RATE;
        let buffer_length_seconds =
            buffer_capacity_bytes as f64 / (bytes_per_sample as f64 * samples_per_second as f64);
        buffer_length_seconds
    }
}

impl Iterator for InputFile {
    type Item = AudioPacket;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = [0u8; FFMPEG_STDOUT_BUFFER_SIZE as usize];
        let n = self
            .reader
            .read(&mut buffer)
            .expect("input_file: falha ao ler bytes");

        if n == 0 {
            println!("input_file: arquivo finalizado");
            return None;
        }

        let audio_length = InputFile::calculate_buffer_length(n as u32);

        return Some(AudioPacket {
            audio_length,
            buffer: Bytes::copy_from_slice(&buffer[..n]),
        });
    }
}

impl Drop for InputFile {
    fn drop(&mut self) {
        self.child
            .kill()
            .expect("input_file: ffmpeg não pôde ser fechado");
    }
}
