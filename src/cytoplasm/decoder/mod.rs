use std::{
    io::{BufReader, Read},
    path::PathBuf,
    process::{Child, ChildStdout, Command, Stdio},
    time::Duration,
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

impl AudioPacket {
    /// Cria um novo `AudioPacket` contendo som silencioso (zeros) de um determinado comprimento.
    ///
    /// ```rust
    /// let silence_packet = AudioPacket::from_silence(Duration::from_secs(2)); // 2 segundos de silêncio
    /// ```
    pub fn from_silence(duration: Duration) -> AudioPacket {
        let duration_seconds = duration.as_secs() as f64 + duration.subsec_nanos() as f64 / 1e9;
        let sample_count = (duration_seconds * SAMPLE_RATE as f64) as u32;
        let byte_count = sample_count * CHANNEL_COUNT * BYTE_DEPTH;

        // cria um buffer de zeros (silêncio)
        let buffer = vec![0u8; byte_count as usize];

        AudioPacket {
            audio_length: duration_seconds,
            buffer: Bytes::from(buffer),
        }
    }
}

/// Converte um número de milissegundos em uma string de tempo formatada compatível com o parâmetro `-ss` do FFmpeg.
///
/// A string é formatada como `hh:mm:ss[.xxx]`, onde:
/// - `hh` é o número de horas (omitido se for zero)
/// - `mm` é o número de minutos (sempre com dois dígitos)
/// - `ss` é o número de segundos (sempre com dois dígitos)
/// - `.xxx` é o número de milissegundos representado como fração de segundo (até 6 dígitos)
///
/// # Argumentos
///
/// * `milliseconds` - Um número de milissegundos do tipo `u64`
///
/// # Retorna
///
/// * Uma `String` representando o tempo no formato `hh:mm:ss[.xxx]`, compatível com o parâmetro `-ss` do FFmpeg
pub fn ffmeg_seek_time_arg_format(milliseconds: u64) -> String {
    let hours = milliseconds / 3600000;
    let remaining = milliseconds % 3600000;
    let minutes = remaining / 60000;
    let remaining = remaining % 60000;
    let seconds = remaining / 1000;
    let milliseconds = remaining % 1000;

    let mut result = String::new();

    if hours > 0 {
        result.push_str(&format!("{:02}:", hours));
    }

    result.push_str(&format!("{:02}:", minutes));
    result.push_str(&format!("{:02}.", seconds));
    result.push_str(&format!("{:06}", milliseconds)); // 6 digits, zero-padded

    result
}

pub struct InputFile {
    child: Child,
    reader: BufReader<ChildStdout>,
}

impl InputFile {
    pub fn new(file_path: PathBuf, seek_ms: u64) -> InputFile {
        let mut child = Command::new("ffmpeg")
            .args(&[
                "-i",
                &file_path.to_str().unwrap(),
                "-ss",
                &ffmeg_seek_time_arg_format(seek_ms),
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
