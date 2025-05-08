use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::collections::HashSet;

use super::track::Track;

/// Iterador que reproduz trilhas de uma lista de forma aleatória, sem repetição dentro de cada ciclo.
/// O iterador garante que cada trilha seja reproduzida apenas uma vez por ciclo, e ao final do ciclo,
/// as trilhas são reembaralhadas para começar novamente.
pub struct TrackIterator {
    tracks: Vec<Track>,
    indices: Vec<usize>,
    used_indices: HashSet<usize>,
    rng: StdRng,
}

impl TrackIterator {
    pub fn new(all_tracks: Vec<Track>, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut indices: Vec<usize> = (0..all_tracks.len()).collect();
        indices.shuffle(&mut rng);

        TrackIterator {
            tracks: all_tracks,
            indices,
            used_indices: HashSet::new(),
            rng,
        }
    }
}

impl Iterator for TrackIterator {
    type Item = Track;

    /// Retorna a próxima trilha a ser reproduzida.
    ///
    /// O método garante que nenhuma trilha seja repetida dentro do mesmo ciclo, e reembaralha as trilhas
    /// automaticamente após todas terem sido reproduzidas.
    fn next(&mut self) -> Option<Self::Item> {
        // se nenhum índice estiver disponível, reembaralhar e zerar a lista de usados
        if self.used_indices.is_empty() {
            self.indices.shuffle(&mut self.rng);
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
