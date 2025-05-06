use std::{
    fs::File,
    io::{BufReader, Read},
    process::{Child, ChildStdout, Command, Stdio},
};

use bytes::Bytes;

use crate::input_decoder::input_audio_file::calculate_buffer_length;

use super::input_audio_file::{AudioFile, AudioPacket, BYTE_DEPTH, CHANNEL_COUNT, SAMPLE_RATE};

pub const FFMPEG_STDOUT_BUFFER_SIZE: u32 = 1 * (SAMPLE_RATE * CHANNEL_COUNT * BYTE_DEPTH); // 1 segundo de áudio

pub struct ComplexCodecFile {
    file_path: String,
    file_size: u64,

    child: Child,
    reader: BufReader<ChildStdout>,
}

impl ComplexCodecFile {
    pub fn new(file_path: String) -> ComplexCodecFile {
        let file = File::open(file_path.clone()).expect("complex_codec_file: Failed to open file");
        let file_size = file.metadata().unwrap().len();

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
            .expect("complex_codec_file: Falha ao spawnar o ffmpeg");

        let stdout = child
            .stdout
            .take()
            .expect("complex_codec_file: Falha ao ler stdout");
        let reader = BufReader::new(stdout);

        ComplexCodecFile {
            file_path,
            file_size,
            reader,
            child,
        }
    }
}

impl AudioFile for ComplexCodecFile {
    fn audio_file_path(&self) -> String {
        self.file_path.clone()
    }

    fn audio_file_size_bytes(&self) -> u64 {
        self.file_size
    }
}

impl Iterator for ComplexCodecFile {
    type Item = AudioPacket;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = [0u8; FFMPEG_STDOUT_BUFFER_SIZE as usize];
        let n = self
            .reader
            .read(&mut buffer)
            .expect("complex_codec_file: Falha ao ler bytes");

        if n == 0 {
            println!("complex_codec_file: EOF.");
            return None;
        }

        let audio_length = calculate_buffer_length(n as u32);

        return Some(AudioPacket {
            audio_length,
            buffer: Bytes::copy_from_slice(&buffer[..n]),
        });
    }
}

impl Drop for ComplexCodecFile {
    fn drop(&mut self) {
        self.child
            .kill()
            .expect("complex_codec_file: ffmpeg não pôde ser fechado");
    }
}
