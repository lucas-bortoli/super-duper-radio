use super::track::Track;
use frand::{Rand, Shufflable};
use std::collections::HashSet;

/// Iterador que reproduz trilhas de uma lista de forma aleatória, sem repetição dentro de cada ciclo.
/// O iterador garante que cada trilha seja reproduzida apenas uma vez por ciclo, e ao final do ciclo,
/// as trilhas são reembaralhadas para começar novamente.
pub struct TrackIterator {
    tracks: Vec<Track>,
    indices: Vec<usize>,
    used_indices: HashSet<usize>,
}

impl TrackIterator {
    pub fn new(all_tracks: Vec<Track>) -> Self {
        let indices: Vec<usize> = (0..all_tracks.len()).collect();

        TrackIterator {
            tracks: all_tracks,
            indices,
            used_indices: HashSet::new(),
        }
    }

    /// Retorna a próxima trilha a ser reproduzida.
    ///
    /// O método garante que nenhuma trilha seja repetida dentro do mesmo ciclo, e reembaralha as trilhas
    /// automaticamente após todas terem sido reproduzidas.
    pub fn next(&mut self, rng: &mut Rand) -> Option<Track> {
        // se nenhum índice estiver disponível, reembaralhar e zerar a lista de usados
        if self.used_indices.is_empty() {
            self.indices.shuffle(rng);
            self.used_indices.clear();
        }

        // encontrar um índice não usado
        let mut picked_idx = None;
        for &idx in &self.indices {
            if !self.used_indices.contains(&idx) {
                picked_idx = Some(idx);
                break;
            }
        }

        // se nenhum índice não usado for encontrado (não deve acontecer), retornar None
        let picked_idx = picked_idx?;
        self.used_indices.insert(picked_idx);

        Some(self.tracks[picked_idx].clone())
    }
}
