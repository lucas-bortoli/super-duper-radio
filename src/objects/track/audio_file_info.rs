use std::{fs::File, path::PathBuf, process::Command};

/// Representa as informações de um arquivo de áudio
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct AudioFileInfo {
    /// Localização do arquivo de áudio
    location: PathBuf,
    /// Tamanho do arquivo em bytes
    size_bytes: u64,
    /// Duração do áudio em milissegundos
    audio_milliseconds: u64,
    // TODO: talvez mais campos legais de extrair do arquivo de áudio? bitrate, contagem de canais, título da música (se houver), outras..?
}

// Extrair as informações de um arquivo de áudio
pub fn query(location: PathBuf) -> Result<AudioFileInfo, String> {
    let metadata = File::open(&location)
        .map_err(|e| format!("query: falha ao abrir arquivo para inspeção: {}", e))?
        .metadata()
        .map_err(|e| format!("query: falha ao obter metadados do arquivo: {}", e))?;

    // usamos o ffprobe, que vem de brinde com o ffmpeg, para obter a duração do arquivo de áudio
    let output = Command::new("ffprobe")
        .args(&[
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            location.to_str().ok_or("query: localização inválida")?,
        ])
        .output()
        .map_err(|e| format!("query: falha no probe do arquivo: {}", e))?;

    // sucesso?
    if !output.status.success() {
        return Err(format!(
            "query: status de saída do probe: {}",
            output.status.code().unwrap_or(-1)
        ));
    }

    let audio_seconds_float = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .map_err(|e| format!("query: falha ao interpretar saída como f64: {}", e))?;

    Ok(AudioFileInfo {
        location,
        size_bytes: metadata.len(),
        audio_milliseconds: (audio_seconds_float * 1000.0) as u64,
    })
}
