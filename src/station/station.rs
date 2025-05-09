use std::path::PathBuf;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::{cytoplasm::Cytoplasm, track::track::StationManifest};

use super::metadata_output_stream::MetadataOutputStream;
use super::station_state::StationState;

pub struct Station {
    pub base_dir: PathBuf,
    pub manifest: StationManifest,
    pub cytoplasm: Cytoplasm,
    pub metadata_stream: Arc<MetadataOutputStream>,

    _cancel_signal_sender: Option<oneshot::Sender<bool>>,
}

impl Station {
    pub fn new(
        base_dir: PathBuf,
        manifest: StationManifest,
        cytoplasm: Cytoplasm,
        state_tx: SyncSender<StationState>,
    ) -> Station {
        let metadata_stream = Arc::new(MetadataOutputStream::new());
        let (cancel_signal_sender, cancel_rx) = oneshot::channel::<bool>();

        StationState::spawn_state_thread(
            manifest.clone(),
            cancel_rx,
            state_tx,
            metadata_stream.clone(),
        );

        Station {
            base_dir,
            manifest,
            cytoplasm,
            metadata_stream,
            _cancel_signal_sender: Some(cancel_signal_sender),
        }
    }
}

impl Drop for Station {
    fn drop(&mut self) {
        // sinalizar a thread de producer de estado que deve finalizar
        if let Some(signal) = self._cancel_signal_sender.take() {
            let _ = signal.send(true);
        }
    }
}
