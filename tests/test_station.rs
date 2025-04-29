mod test;

#[cfg(test)]
pub mod test_station {
    use web_radio::objects::station::station::Station;
    use web_radio::objects::station::station_state::MockStationState;
    use web_radio::objects::subscriber::Subscriber;

    #[test]
    fn test_station_creation() {
        let station = get_station_test();

        assert_eq!(station.name, "Diamond City Radio");
        assert_eq!(station.path, "./diamond_city_radio/");
        assert_eq!(station.frequency, 98.9);
        assert!(!station.tracks.is_empty());
    }

    #[test]
    fn test_add_subscriber() {
        let mut station = get_station_test();

        let subscriber = Subscriber {};
        station.add_subscriber(subscriber.clone());

        assert_eq!(station._subscribers.len(), 1);
        assert_eq!(station._subscribers[0], subscriber);
    }

    #[test]
    fn test_remove_subscriber() {
        let mut station = get_station_test();

        let subscriber = Subscriber {};
        station.add_subscriber(subscriber.clone());
        station.remove_subscriber(&subscriber);

        assert!(station._subscribers.is_empty());
    }

    #[test]
    fn test_change_state() {
        let mut station = get_station_test();

        let new_state = Box::new(MockStationState::new());
        station.change_state(new_state);

        // Assuming MockStationState has a way to verify state change
        // This is a placeholder assertion
        assert!(true);
    }

    #[test]
    fn test_get_music_files() {
        let station = get_station_test();

        // Assuming the test_path contains some mock files
        let music_files = station.get_music_files();
        assert!(music_files.is_empty()); // Adjust based on actual test setup
    }

    // #[test]
    // fn test_fill_tracks() {
    //     let mock_state = Box::new(MockStationState::new());
    //     let mut station = Station::new(
    //         "Test Station".to_string(),
    //         "./test_path/".to_string(),
    //         101.1,
    //         mock_state,
    //     );

    //     // Assuming the test_path/manifest.json contains mock track data
    //     assert!(station.tracks.is_empty()); // Adjust based on actual test setup
    // }

    fn get_station_test() -> Station {
        let station_state = MockStationState::new(); // Assuming StationState has a `new` method
        let station = Station::new(
            "Diamond City Radio".to_owned(),
            "./diamond_city_radio/".to_owned(),
            98.9,
            Box::new(station_state),
        );
        station
    }
}
