use super::audio_encoder::OutputCodec;

/// Um frame silencioso para adicionar ao início de um stream de áudio de diversos formatos.
///
/// **Raciocínio, focado em MP3, mas o princípio é o mesmo:**
/// Ao transmitir dados MP3 raw, no início da stream, o decoder MP3 do cliente pode começar a
/// receber dados do meio de um frame, levando a erros de decoding e falhas no playback.
/// Esse é um frame MP3 válido e silencioso. Ao adicionar este frame ao início do stream,
/// garantimos que o decoder sempre comece com um ponto de SYNC conhecido, mesmo que o stream real
/// comece no meio de um frame MP3. Isso melhora significativamente a robustez da stream.
///
/// **Situação:**
/// Usada em aplicações que transmitem dados MP3 brutos via HTTP sem manipular explicitamente os
/// quadros frames MP3 no lado do servidor. É particularmente útil quando o servidor está gerando
/// o stream dinamicamente (por exemplo, usando ffmpeg) e não tem controle preciso
/// sobre o início da stream.
///
/// **Tradeoff**: O cliente recebe um pacote de 0.02s de áudio silencioso no começo da stream.
///
/// `ffmpeg -f lavfi -i anullsrc=channel_layout=stereo:sample_rate=44100 -acodec libmp3lame -ab 128k -ac 2 -ar 44100 -t 0.02 -vn -sn -f mp3 - | head -c 1472 > mp3_null_frame.bin`
/// `ffmpeg -f lavfi -i anullsrc=channel_layout=stereo:sample_rate=48000 -acodec libopus -ab 128k -ac 2 -ar 48000 -t 0.02 -vn -sn -f opus - > opus_null_frame.opus`
/// `ffmpeg -f lavfi -i anullsrc=channel_layout=stereo:sample_rate=44100 -acodec libvorbis -ab 128k -ac 2 -ar 44100 -t 0.02 -vn -sn -f ogg - > ogg_vorbis_null_frame.ogg`
pub fn get_null_frame(codec: &OutputCodec) -> &'static [u8] {
    match codec {
        OutputCodec::Mp3_64kbps => include_bytes!("./mp3.bin"),
        OutputCodec::Ogg96kbps => include_bytes!("./ogg.bin"),
        OutputCodec::Opus128kbps => include_bytes!("./opus.bin"),
    }
}

pub fn get_mime_type(codec: &OutputCodec) -> &'static str {
    match codec {
        OutputCodec::Mp3_64kbps => "mpeg",
        OutputCodec::Ogg96kbps => "ogg",
        OutputCodec::Opus128kbps => "opus",
    }
}
