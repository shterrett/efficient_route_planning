use std::collections::HashSet;
use std::collections::HashMap;

extern crate csv;

type ServiceId = String;
type TripId = String;
type StopId = String;

type ScheduleRow = (String,
                    usize,
                    usize,
                    usize,
                    usize,
                    usize,
                    usize,
                    usize,
                    String,
                    String);

fn service_on_day(path: &str, day: &str) -> HashSet<ServiceId> {
    let mut reader = csv::Reader::from_file(path).unwrap();
    reader.decode()
          .filter_map(|row|
              match row {
                  Ok(data) => Some(data),
                  Err(_) => None
              }
          )
          .filter(|row: &ScheduleRow| runs_on_day(&day, row))
          .map(|row: ScheduleRow| row.0)
          .collect::<HashSet<ServiceId>>()
}

fn runs_on_day(day: &&str, row: &ScheduleRow) -> bool {
    let mut days = HashMap::new();
    days.insert("monday", row.1);
    days.insert("tuesday", row.2);
    days.insert("wednesday", row.3);
    days.insert("thursday", row.4);
    days.insert("friday", row.5);
    days.insert("saturday", row.6);
    days.insert("sunday", row.7);

    days.get(day).map(|&val| val == 1).unwrap_or(false)
}

type TripRow = (String,
                String,
                String,
                String,
                String,
                String,
                String);

fn trips_for_services(path: &str, services: &HashSet<ServiceId>) -> HashSet<TripId> {
    let mut reader = csv::Reader::from_file(path).unwrap();
    reader.decode()
          .filter_map(|row|
               match row {
                   Ok(data) => Some(data),
                   Err(_) => None
               }
          ).filter_map(|row: TripRow|
            if services.contains(&row.1) {
                Some(row.2)
            } else {
                None
            }
          ).collect::<HashSet<TripId>>()
}

type StopRow = (String,
                Option<String>,
                String,
                Option<String>,
                f64,
                f64,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>);

#[derive(Clone, PartialEq, Debug)]
struct Location {
    x: f64,
    y: f64
}

fn stops_data(path: &str) -> HashMap<StopId, Location> {
    let mut reader = csv::Reader::from_file(path).unwrap();
    reader.decode()
          .filter_map(|row|
               match row {
                   Ok(data) => Some(data),
                   Err(_) => None
               }
          )
          .map(|row: StopRow|
                (row.0, Location { x: row.5, y: row.4 })
          )
          .collect()
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::collections::HashSet;
    use super::{ TripId,
                 Location,
                 service_on_day,
                 trips_for_services,
                 stops_data
               };

    #[test]
    fn return_services_active_on_a_day() {
        let services = service_on_day("data/gtfs_example/calendar.txt", "wednesday");

        let mut expected = HashSet::new();
        expected.insert("weekday".to_string());

        assert_eq!(services, expected);
    }

    #[test]
    fn return_trips_for_services() {
        let mut services = HashSet::new();
        services.insert("weekday".to_string());

        let trips = trips_for_services("data/gtfs_example/trips.txt", &services);

        let expected_trips = vec!["g1",
                                  "g2",
                                  "g3",
                                  "g4",
                                  "g5",
                                  "r1",
                                  "r2",
                                  "r3"];
        let expected = expected_trips.iter()
                                     .map(|t| t.to_string())
                                     .collect::<HashSet<TripId>>();

        assert_eq!(trips, expected);

    }

    #[test]
    fn build_stop_data_map() {
        let stops = stops_data("data/gtfs_example/stops.txt");

        let expected_stops = vec![("A".to_string(), Location { x: 0.0, y: 1.0 }),
                                  ("B".to_string(), Location { x: 1.0, y: 3.0 }),
                                  ("C".to_string(), Location { x: 1.0, y: 0.0 }),
                                  ("D".to_string(), Location { x: 2.0, y: 1.0 }),
                                  ("E".to_string(), Location { x: 3.0, y: 2.0 }),
                                  ("F".to_string(), Location { x: 4.0, y: 1.0 })];
        let expected = expected_stops.into_iter()
                                     .collect::<HashMap<String, Location>>();
        assert_eq!(stops, expected);
    }
}
