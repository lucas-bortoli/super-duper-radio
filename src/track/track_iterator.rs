// estrutura para armazenamento de estado anterior da estação

use super::track::Track;
use rand::seq::SliceRandom;
use rand::{SeedableRng, Rng};
use rand::rngs::StdRng;
use core::result::Result;

pub struct TrackIterator {
    track: Track,
    track_queue: Vec<Track>,
    current_index: usize,
    _rng: StdRng,

}

impl TrackIterator {
    pub fn new(mut track_queue: Vec<Track>, seed: u64) -> Self {

        TrackIterator::shuffle_vec(&mut track_queue, seed);

        let track = track_queue[0].clone();
        track_queue.remove(0);

        let _rng = StdRng::seed_from_u64(seed);

        TrackIterator {
            track,
            track_queue,
            current_index: 0,
            _rng
        }
    }

    pub fn has_more(&self) -> bool {
        !self.track_queue.is_empty()
    }

    pub fn go_next(&mut self) -> Result<(), &'static str> {
        let nx_track = self.get_next()?;
        self.track = nx_track;
        Ok(())
    }
    
    pub fn get_next(&mut self) -> Result<Track, &'static str> {
        if !self.has_more() {
            return Err("No more tracks to play");
        }
    
        self.shuffle();
        let next_index = self.pick_next();
    
        // Usando clone para obter um valor owned
        let nx_track = self.track_queue[next_index].clone();
        self.track_queue.remove(next_index);
        Ok(nx_track)
    }

    fn shuffle(&mut self) {
        self.track_queue.shuffle(&mut self._rng);
    }

    fn pick_next(&mut self) -> usize {
        let next_index = self._rng.random_range(0..self.track_queue.len());
        self.current_index = next_index;
        next_index
    }

    pub fn get_current(&self) -> &Track {
        &self.track
    }

    pub fn shuffle_vec(vec: &mut Vec<Track>, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);

        vec.shuffle(&mut rng)
    }
}