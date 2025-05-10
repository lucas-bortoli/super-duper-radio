use super::track::Track;
use frand::{Rand, Shufflable};
use std::collections::VecDeque;

/// Iterador que reproduz trilhas de uma lista de forma aleatória, sem repetição dentro de cada ciclo.
/// O iterador garante que cada trilha seja reproduzida apenas uma vez por ciclo, e ao final do ciclo,
/// as trilhas são reembaralhadas para começar novamente.
pub struct TrackIterator {
    tracks: Vec<Track>,
    track_queue: VecDeque<Track>,
}

impl TrackIterator {
    pub fn new(all_tracks: Vec<Track>) -> Self {
        TrackIterator {
            tracks: all_tracks,
            track_queue: VecDeque::new(),
        }
    }

    /// Retorna a próxima trilha a ser reproduzida.
    ///
    /// O método garante que nenhuma trilha seja repetida dentro do mesmo ciclo, e reembaralha as trilhas
    /// automaticamente após todas terem sido reproduzidas.
    pub fn next(&mut self, rng: &mut Rand) -> Option<Track> {
        // se nenhum índice estiver disponível, reembaralhar e zerar a lista de usados
        if self.track_queue.is_empty() {
            // repopular a fila de tracks, com ordem aleatória
            self.track_queue = VecDeque::from(self.tracks.clone().shuffled(rng));
        }

        self.track_queue.pop_front()
    }
}
