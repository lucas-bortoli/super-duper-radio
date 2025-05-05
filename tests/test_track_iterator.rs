use web_radio::objects::track::track::Track;
use web_radio::objects::track::track_iterator::TrackIterator;

fn mock_track(id: usize) -> Track {
    Track::new(
        format!("Title {}", id),
        format!("Artist {}", id),
        "Album".to_string(),
        200,
        "wav".to_string(),
        format!("file{}.wav", id),
        vec![],
        vec![]
    )
}

#[test]
fn iterator_initialization_and_current() {
    let tracks = vec![ mock_track(1), mock_track(2), mock_track(3) ];
    let iter = TrackIterator::new(tracks.clone(), 1234);
    // get_current nunca será vazio
    assert!(!iter.get_current().title.is_empty());
    assert!(!iter.get_current().source.is_empty());
}

#[test]
fn iterator_has_more() {
    let tracks = vec![ mock_track(1) ];
    let iter = TrackIterator::new(tracks.clone(), 42);
    // só 1 item => has_more deve ser false
    assert!(!iter.has_more());
}

#[test]
fn iterator_go_next_exhausts_queue() {
    let mut tracks = vec![ mock_track(1), mock_track(2) ];
    let mut iter = TrackIterator::new(tracks.clone(), 999);
    let first = iter.get_current().clone();
    // after first, queue len == original-1
    assert!(iter.has_more());
    iter.go_next().expect("deve ter next");
    let second = iter.get_current().clone();
    assert_ne!(first.title, second.title);
    // agora sem mais
    assert!(!iter.has_more());
}
