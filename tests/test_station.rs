// use web_radio::objects::station::station::Station;
// use web_radio::objects::subscriber::Subscriber;

// #[test]
// fn station_lifecycle_and_state_transitions() {
//     let mut station = Station::new(
//         "Test".to_string(),
//         "./diamond_city_radio/".to_string(),
//         98.9,
//         42,
//     );
//     assert_eq!(station.name, "Test");
//     assert_eq!(station.path, "./diamond_city_radio/");
//     assert!((station.frequency - 98.9).abs() < f32::EPSILON);
//     assert_eq!(station.state_name(), "Down");


//     assert!(station.subscribers.is_empty());
//     let sub = Subscriber {};
//     assert_eq!(station.subscribers.len(), 1);
//     assert_eq!(station.subscribers[0], sub);
//     assert!(station.subscribers.is_empty());

//     station.play();
//     assert_eq!(station.state_name(), "Playing");

//     let current = station.current_track.title.clone();
//     station.next();
//     let after = station.current_track.title.clone();
//     assert!(
//         current != after || station.state_name() == "Down",
//         "ou mudou a faixa, ou caiu para Down"
//     );

//     assert!(station.snapshots.len() >= 1);

//     station.stop();
//     assert_eq!(station.state_name(), "Down");
// }
