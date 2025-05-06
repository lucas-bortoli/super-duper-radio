use std::{
    fs::File,
    io::{Read, Seek},
    ops::DerefMut,
};

use bytes::Bytes;

use super::input_audio_file::{
    calculate_buffer_length, AudioFile, AudioPacket, BYTE_DEPTH, CHANNEL_COUNT, SAMPLE_RATE,
};

const READ_BUFFER_SIZE: usize = (SAMPLE_RATE * CHANNEL_COUNT * BYTE_DEPTH) as usize;
type ReadBuffer = [u8; READ_BUFFER_SIZE];

pub struct WavCodecFile {
    file_path: String,
    file_size: u64,

    file: File,
    audio_buffer: Box<ReadBuffer>,
}

impl WavCodecFile {
    pub fn new(file_path: String) -> Self {
        let mut file = File::open(file_path.clone())
            .unwrap_or_else(|_| panic!("wav_codec: File {} is not readable?", file_path));
        let file_size = file
            .metadata()
            .expect("wav_codec: File has no metadata?")
            .len();

        // TODO: validar headers do wav, ver se o sample rate e outros parâmetros do arquivo são equivalentes ao contrato em input_audio_file
        // TODO: (...validar aqui...)

        // skipar header WAV, para não ler como se fosse áudio
        file.seek(std::io::SeekFrom::Start(44))
            .expect("wav_codec: Skipping WAV header failed");

        WavCodecFile {
            file_path,
            file_size,
            file,
            audio_buffer: Box::new([0u8; READ_BUFFER_SIZE]),
        }
    }
}

impl AudioFile for WavCodecFile {
    fn audio_file_path(&self) -> String {
        self.file_path.clone()
    }

    fn audio_file_size_bytes(&self) -> u64 {
        self.file_size
    }
}

impl Iterator for WavCodecFile {
    type Item = AudioPacket;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes_read = self
            .file
            .read(self.audio_buffer.deref_mut())
            .expect("wav_codec: Audio file is unreadable");

        if bytes_read == 0 {
            println!("wav_codec: EOF.");
            return None;
        }

        let packet = AudioPacket {
            audio_length: calculate_buffer_length(bytes_read as u32),
            buffer: Bytes::copy_from_slice(&self.audio_buffer[..bytes_read]),
        };

        return Some(packet);
    }
}
