use std::collections::HashSet;
use std::collections::HashMap;
use time::{ strptime };

use weighted_graph::{ GraphKey, Graph };

extern crate csv;

type ServiceId = String;
pub type TripId = String;
pub type StopId = String;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Clone)]
pub enum NodeType {
    Arrival,
    Departure,
    Transfer
}

impl NodeType {
    pub fn is_arrival(&self) -> bool {
        match self {
            &NodeType::Arrival => true,
            _ => false
        }
    }

    pub fn is_departure(&self) -> bool {
        match self {
            &NodeType::Departure => true,
            _ => false
        }
    }

    pub fn is_transfer(&self) -> bool {
        match self {
            &NodeType::Transfer => true,
            _ => false
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct GtfsId {
    pub stop_id: StopId,
    pub time: i64,
    pub node_type: NodeType,
    pub trip_id: Option<TripId>
}
impl GraphKey for GtfsId {}

const FIVE_MINUTES: i64 = 5 * 60;

pub fn build_graph_from_gtfs(gtfs_dir: &str, day: &str) -> Graph<GtfsId> {
    let schedule_path = gtfs_dir.to_string() + "calendar.txt";
    let trip_path = gtfs_dir.to_string() + "trips.txt";
    let stops_path = gtfs_dir.to_string() + "stops.txt";

    let services = service_on_day(&schedule_path, &day);
    let trips = trips_for_services(&trip_path,
                                   &services);
    let stops = stops_data(&stops_path);

    assemble_graph(gtfs_dir, &trips, &stops)
}

type StopTimeRow = (String,
                    String,
                    String,
                    String,
                    Option<String>,
                    Option<String>,
                    Option<String>,
                    Option<String>,
                    Option<String>);

fn assemble_graph(gtfs_dir: &str,
                  trips: &HashSet<TripId>,
                  stops: &HashMap<StopId, Location>) -> Graph<GtfsId> {
    let mut reader = csv::Reader::from_file(gtfs_dir.to_string() + "stop_times.txt").unwrap();
    let mut graph = Graph::new();
    for row in reader.decode() {
        let data: StopTimeRow = row.unwrap();
        if trips.contains(&data.0) {
            build_nodes(&data, stops, &mut graph);
        }
    }
    build_trip_edges(&mut graph);
    link_transfer_nodes(&mut graph);
    graph
}

fn build_nodes(data: &StopTimeRow,
               stops: &HashMap<StopId, Location>,
               graph: &mut Graph<GtfsId>) {
    if let (Some(arrival_time),
            Some(departure_time)) = (time_to_seconds_after_midnight(&data.1),
                                     time_to_seconds_after_midnight(&data.2)) {

        let arr_node_id = GtfsId { stop_id: data.3.clone(),
                                   time: arrival_time,
                                   node_type: NodeType::Arrival,
                                   trip_id: Some(data.0.clone())
                                 };
        let dep_node_id = GtfsId { stop_id: data.3.clone(),
                                   time: departure_time,
                                   node_type: NodeType::Departure,
                                   trip_id: Some(data.0.clone())
                                 };
        let trf_node_id = GtfsId { stop_id: data.3.clone(),
                                   time: arrival_time + FIVE_MINUTES,
                                   node_type: NodeType::Transfer,
                                   trip_id: None
                                 };

        if let Some(stop_data) = stops.get(&data.3) {
            for node_id in vec![&arr_node_id, &dep_node_id, &trf_node_id] {
                graph.add_node(node_id.clone(), stop_data.x, stop_data.y);
            }
            graph.add_edge(edge_id(&arr_node_id, &trf_node_id),
                           arr_node_id,
                           trf_node_id,
                           FIVE_MINUTES);
        }
    }
}

fn edge_id(from: &GtfsId, to: &GtfsId) -> GtfsId {
    GtfsId {
        stop_id: from.stop_id.clone() + &to.stop_id.clone(),
        time: from.time.clone(),
        node_type: to.node_type.clone(),
        trip_id: None
    }
}

fn build_trip_edges(graph: &mut Graph<GtfsId>) {
    let mut trip_nodes = HashMap::new();
    for node in graph.all_nodes() {
        if let Some(ref trip_id) = node.id.trip_id {
            let mut nodes_for_trip = trip_nodes.entry(trip_id.clone()).or_insert(Vec::new());
            nodes_for_trip.push(node.id.clone());
        }
    }

    for (_, nodes) in trip_nodes.iter_mut() {
        let mut ns = nodes.iter().filter(|n| !n.node_type.is_transfer()).collect::<Vec<&GtfsId>>();
        ns.sort_by(|a, b|
                   if a.time == b.time {
                       a.node_type.cmp(&b.node_type)
                   } else {
                       a.time.cmp(&b.time)
                   });

        for adj_nodes in ns.windows(2) {
            let from = adj_nodes[0].clone();
            let to = adj_nodes[1].clone();
            let edge_weight = to.time - from.time;
            graph.add_edge(edge_id(&from, &to),
                           from,
                           to,
                           edge_weight);
        }
    }
}

fn link_transfer_nodes(graph: &mut Graph<GtfsId>) {
    let mut stop_nodes = HashMap::new();
    for node in graph.all_nodes().iter().filter(|n| !n.id.node_type.is_arrival()) {
        let mut nodes_for_stop = stop_nodes.entry(node.id.stop_id.clone()).or_insert(Vec::new());
        nodes_for_stop.push(node.id.clone());
    }

    for (_, nodes) in stop_nodes.into_iter() {
        let (mut transfers,
             mut departures): (Vec<GtfsId>,
                               Vec<GtfsId>) = nodes.into_iter()
                                                   .partition(|n| n.node_type.is_transfer());

        transfers.sort_by(|a, b| a.time.cmp(&b.time));
        departures.sort_by(|a, b| a.time.cmp(&b.time));

        link_adjacent_transfers(graph, &transfers);
        link_transfers_to_departures(graph, &transfers, departures);
    }
}

fn link_adjacent_transfers(graph: &mut Graph<GtfsId>, transfers: &Vec<GtfsId>) {
    for adj_transfers in transfers.windows(2) {
        let from = adj_transfers[0].clone();
        let to = adj_transfers[1].clone();
        let edge_weight = to.time - from.time;
            graph.add_edge(edge_id(&from, &to),
                            from,
                            to,
                            edge_weight);
    }

}

fn link_transfers_to_departures(graph: &mut Graph<GtfsId>,
                                transfers: &Vec<GtfsId>,
                                departures: Vec<GtfsId>) {

    for departure in departures {
        if let Some(transfer) = transfers.iter()
                                            .filter(|t| t.time <= departure.time)
                                            .max_by_key(|t| t.time) {
        let edge_weight = departure.time - transfer.time;
        graph.add_edge(edge_id(&transfer, &departure),
                        transfer.clone(),
                        departure,
                        edge_weight);
        }
    }
}

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

fn runs_on_day(day: &str, row: &ScheduleRow) -> bool {
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

pub fn time_to_seconds_after_midnight(t_str: &String) -> Option<i64> {
    match strptime(t_str, "%T") {
        Ok(t) => {
            Some((t.tm_sec + 60 * t.tm_min + 60 * 60 * t.tm_hour) as i64)
        }
        Err(_) => None
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::collections::HashSet;
    use test_helpers::to_node_id;
    use super::{ GtfsId,
                 TripId,
                 Location,
                 NodeType,
                 service_on_day,
                 trips_for_services,
                 stops_data,
                 time_to_seconds_after_midnight,
                 build_graph_from_gtfs
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

    #[test]
    fn parse_times_to_seconds() {
        let t = "08:00:00".to_string();
        let invalid = "notatime".to_string();

        assert_eq!(time_to_seconds_after_midnight(&t), Some(8 * 60 * 60));
        assert_eq!(time_to_seconds_after_midnight(&invalid), None);
    }

    #[test]
    fn build_transit_graph_with_valid_nodes() {
        let nodes = vec![("A", "06:00:00", NodeType::Arrival, Some("r1")),
                         ("A", "06:00:00", NodeType::Departure, Some("r1")),
                         ("A", "06:05:00", NodeType::Transfer, None),
                         ("A", "07:00:00", NodeType::Arrival, Some("r2")),
                         ("A", "07:00:00", NodeType::Departure, Some("r2")),
                         ("A", "07:05:00", NodeType::Transfer, None),
                         ("A", "08:00:00", NodeType::Arrival, Some("r3")),
                         ("A", "08:00:00", NodeType::Departure, Some("r3")),
                         ("A", "08:05:00", NodeType::Transfer, None),
                         ("B", "06:25:00", NodeType::Arrival, Some("r1")),
                         ("B", "06:25:00", NodeType::Departure, Some("r1")),
                         ("B", "06:30:00", NodeType::Transfer, None),
                         ("B", "07:25:00", NodeType::Arrival, Some("r2")),
                         ("B", "07:25:00", NodeType::Departure, Some("r2")),
                         ("B", "07:30:00", NodeType::Transfer, None),
                         ("B", "08:25:00", NodeType::Arrival, Some("r3")),
                         ("B", "08:25:00", NodeType::Departure, Some("r3")),
                         ("B", "08:30:00", NodeType::Transfer, None),
                         ("E", "06:50:00", NodeType::Arrival, Some("r1")),
                         ("E", "06:50:00", NodeType::Departure, Some("r1")),
                         ("E", "06:55:00", NodeType::Transfer, None),
                         ("E", "07:50:00", NodeType::Arrival, Some("r2")),
                         ("E", "07:50:00", NodeType::Departure, Some("r2")),
                         ("E", "07:55:00", NodeType::Transfer, None),
                         ("E", "08:50:00", NodeType::Arrival, Some("r3")),
                         ("E", "08:50:00", NodeType::Departure, Some("r3")),
                         ("E", "08:55:00", NodeType::Transfer, None),
                         ("A", "06:15:00", NodeType::Arrival, Some("g1")),
                         ("A", "06:15:00", NodeType::Departure, Some("g1")),
                         ("A", "06:20:00", NodeType::Transfer, None),
                         ("A", "06:45:00", NodeType::Arrival, Some("g2")),
                         ("A", "06:45:00", NodeType::Departure, Some("g2")),
                         ("A", "06:50:00", NodeType::Transfer, None),
                         ("A", "07:15:00", NodeType::Arrival, Some("g3")),
                         ("A", "07:15:00", NodeType::Departure, Some("g3")),
                         ("A", "07:20:00", NodeType::Transfer, None),
                         ("A", "07:45:00", NodeType::Arrival, Some("g4")),
                         ("A", "07:45:00", NodeType::Departure, Some("g4")),
                         ("A", "07:50:00", NodeType::Transfer, None),
                         ("A", "08:15:00", NodeType::Arrival, Some("g5")),
                         ("A", "08:15:00", NodeType::Departure, Some("g5")),
                         ("A", "08:20:00", NodeType::Transfer, None),
                         ("C", "06:45:00", NodeType::Arrival, Some("g1")),
                         ("C", "06:45:00", NodeType::Departure, Some("g1")),
                         ("C", "06:50:00", NodeType::Transfer, None),
                         ("C", "07:15:00", NodeType::Arrival, Some("g2")),
                         ("C", "07:15:00", NodeType::Departure, Some("g2")),
                         ("C", "07:20:00", NodeType::Transfer, None),
                         ("C", "07:45:00", NodeType::Arrival, Some("g3")),
                         ("C", "07:45:00", NodeType::Departure, Some("g3")),
                         ("C", "07:50:00", NodeType::Transfer, None),
                         ("C", "08:15:00", NodeType::Arrival, Some("g4")),
                         ("C", "08:15:00", NodeType::Departure, Some("g4")),
                         ("C", "08:20:00", NodeType::Transfer, None),
                         ("C", "08:45:00", NodeType::Arrival, Some("g5")),
                         ("C", "08:45:00", NodeType::Departure, Some("g5")),
                         ("C", "08:50:00", NodeType::Transfer, None),
                         ("D", "07:00:00", NodeType::Arrival, Some("g1")),
                         ("D", "07:00:00", NodeType::Departure, Some("g1")),
                         ("D", "07:05:00", NodeType::Transfer, None),
                         ("D", "07:30:00", NodeType::Arrival, Some("g2")),
                         ("D", "07:30:00", NodeType::Departure, Some("g2")),
                         ("D", "07:35:00", NodeType::Transfer, None),
                         ("D", "08:00:00", NodeType::Arrival, Some("g3")),
                         ("D", "08:00:00", NodeType::Departure, Some("g3")),
                         ("D", "08:05:00", NodeType::Transfer, None),
                         ("D", "08:30:00", NodeType::Arrival, Some("g4")),
                         ("D", "08:30:00", NodeType::Departure, Some("g4")),
                         ("D", "08:35:00", NodeType::Transfer, None),
                         ("D", "09:00:00", NodeType::Arrival, Some("g5")),
                         ("D", "09:00:00", NodeType::Departure, Some("g5")),
                         ("D", "09:05:00", NodeType::Transfer, None),
                         ("E", "07:30:00", NodeType::Arrival, Some("g1")),
                         ("E", "07:30:00", NodeType::Departure, Some("g1")),
                         ("E", "07:35:00", NodeType::Transfer, None),
                         ("E", "08:00:00", NodeType::Arrival, Some("g2")),
                         ("E", "08:00:00", NodeType::Departure, Some("g2")),
                         ("E", "08:05:00", NodeType::Transfer, None),
                         ("E", "08:30:00", NodeType::Arrival, Some("g3")),
                         ("E", "08:30:00", NodeType::Departure, Some("g3")),
                         ("E", "08:35:00", NodeType::Transfer, None),
                         ("E", "09:00:00", NodeType::Arrival, Some("g4")),
                         ("E", "09:00:00", NodeType::Departure, Some("g4")),
                         ("E", "09:05:00", NodeType::Transfer, None),
                         ("E", "09:30:00", NodeType::Arrival, Some("g5")),
                         ("E", "09:30:00", NodeType::Departure, Some("g5")),
                         ("E", "09:35:00", NodeType::Transfer, None),
                         ("F", "07:40:00", NodeType::Arrival, Some("g1")),
                         ("F", "07:40:00", NodeType::Departure, Some("g1")),
                         ("F", "07:45:00", NodeType::Transfer, None),
                         ("F", "08:10:00", NodeType::Arrival, Some("g2")),
                         ("F", "08:10:00", NodeType::Departure, Some("g2")),
                         ("F", "08:15:00", NodeType::Transfer, None),
                         ("F", "08:40:00", NodeType::Arrival, Some("g3")),
                         ("F", "08:40:00", NodeType::Departure, Some("g3")),
                         ("F", "08:45:00", NodeType::Transfer, None),
                         ("F", "09:10:00", NodeType::Arrival, Some("g4")),
                         ("F", "09:10:00", NodeType::Departure, Some("g4")),
                         ("F", "09:15:00", NodeType::Transfer, None),
                         ("F", "09:40:00", NodeType::Arrival, Some("g5")),
                         ("F", "09:40:00", NodeType::Departure, Some("g5")),
                         ("F", "09:45:00", NodeType::Transfer, None)
                    ];

        let expected_node_ids = nodes.into_iter()
                                     .map(|data| to_node_id(data))
                                     .collect::<HashSet<GtfsId>>();

        let graph = build_graph_from_gtfs("data/gtfs_example/", "wednesday");

        let actual_nodes = graph.all_nodes()
                                .iter()
                                .map(|&node| node.id.clone())
                                .collect::<HashSet<GtfsId>>();

        assert_eq!(actual_nodes, expected_node_ids);
    }

    #[test]
    fn build_transit_graph_with_edges_within_trip() {
        let edges = vec![
            (("A", "06:15:00", NodeType::Departure, Some("g1")),
             ("C", "06:45:00", NodeType::Arrival, Some("g1")),
             30),
            (("C", "06:45:00", NodeType::Arrival, Some("g1")),
             ("C", "06:45:00", NodeType::Departure, Some("g1")),
             0),
            (("C", "06:45:00", NodeType::Arrival, Some("g1")),
             ("C", "06:50:00", NodeType::Transfer, None),
             5),
            (("C", "06:45:00", NodeType::Departure, Some("g1")),
             ("D", "07:00:00", NodeType::Arrival, Some("g1")),
             15),
            (("D", "07:00:00", NodeType::Arrival, Some("g1")),
             ("D", "07:00:00", NodeType::Departure, Some("g1")),
             0),
            (("D", "07:00:00", NodeType::Arrival, Some("g1")),
             ("D", "07:05:00", NodeType::Transfer, None),
             5),
            (("D", "07:00:00", NodeType::Departure, Some("g1")),
             ("E", "07:30:00", NodeType::Arrival, Some("g1")),
             30),
            (("E", "07:30:00", NodeType::Arrival, Some("g1")),
             ("E", "07:30:00", NodeType::Departure, Some("g1")),
             0),
            (("E", "07:30:00", NodeType::Arrival, Some("g1")),
             ("E", "07:35:00", NodeType::Transfer, None),
             5),
            (("E", "07:30:00", NodeType::Departure, Some("g1")),
             ("F", "07:40:00", NodeType::Arrival, Some("g1")),
             10),
            (("F", "07:40:00", NodeType::Arrival, Some("g1")),
             ("F", "07:40:00", NodeType::Departure, Some("g1")),
             0),
            (("F", "07:40:00", NodeType::Arrival, Some("g1")),
             ("F", "07:45:00", NodeType::Transfer, None),
             5)];

        let mut graph = build_graph_from_gtfs("data/gtfs_example/", "wednesday");

        for edge in edges {
            let from = to_node_id(edge.0);
            let to = to_node_id(edge.1);
            let cost = edge.2;

            let actual_edge = graph.get_mut_edge(&from, &to);
            assert!(actual_edge.is_some());
            assert_eq!(actual_edge.map(|e| e.weight), Some(cost * 60));
        }
    }

    #[test]
    fn attaches_transfer_nodes() {
        let transfer_edges = vec![
            // arrival -> transfer
            (("E", "06:50:00", NodeType::Arrival, Some("r1")),
             ("E", "06:55:00", NodeType::Transfer, None),
             5),
            (("E", "07:50:00", NodeType::Arrival, Some("r2")),
             ("E", "07:55:00", NodeType::Transfer, None),
             5),
            (("E", "08:50:00", NodeType::Arrival, Some("r3")),
             ("E", "08:55:00", NodeType::Transfer, None),
             5),
            (("E", "07:30:00", NodeType::Arrival, Some("g1")),
             ("E", "07:35:00", NodeType::Transfer, None),
             5),
            (("E", "08:00:00", NodeType::Arrival, Some("g2")),
             ("E", "08:05:00", NodeType::Transfer, None),
             5),
            (("E", "08:30:00", NodeType::Arrival, Some("g3")),
             ("E", "08:35:00", NodeType::Transfer, None),
             5),
            (("E", "09:00:00", NodeType::Arrival, Some("g4")),
             ("E", "09:05:00", NodeType::Transfer, None),
             5),
            (("E", "09:30:00", NodeType::Arrival, Some("g5")),
             ("E", "09:35:00", NodeType::Transfer, None),
             5),
            // transfer -> transfer
            (("E", "06:55:00", NodeType::Transfer, None),
             ("E", "07:35:00", NodeType::Transfer, None),
             40),
            (("E", "07:35:00", NodeType::Transfer, None),
             ("E", "07:55:00", NodeType::Transfer, None),
             20),
            (("E", "07:55:00", NodeType::Transfer, None),
             ("E", "08:05:00", NodeType::Transfer, None),
             10),
            (("E", "08:05:00", NodeType::Transfer, None),
             ("E", "08:35:00", NodeType::Transfer, None),
             30),
            (("E", "08:35:00", NodeType::Transfer, None),
             ("E", "08:55:00", NodeType::Transfer, None),
             20),
            (("E", "08:55:00", NodeType::Transfer, None),
             ("E", "09:05:00", NodeType::Transfer, None),
             10),
            (("E", "09:05:00", NodeType::Transfer, None),
             ("E", "09:35:00", NodeType::Transfer, None),
             30),
            // transfer -> departure
            (("E", "06:55:00", NodeType::Transfer, None),
             ("E", "07:30:00", NodeType::Departure, Some("g1")),
             35),
            (("E", "07:35:00", NodeType::Transfer, None),
             ("E", "07:50:00", NodeType::Departure, Some("r2")),
             15),
            (("E", "07:55:00", NodeType::Transfer, None),
             ("E", "08:00:00", NodeType::Departure, Some("g2")),
             5),
            (("E", "08:05:00", NodeType::Transfer, None),
             ("E", "08:30:00", NodeType::Departure, Some("g3")),
             25),
            (("E", "08:35:00", NodeType::Transfer, None),
             ("E", "08:50:00", NodeType::Departure, Some("r3")),
             15),
            (("E", "08:55:00", NodeType::Transfer, None),
             ("E", "09:00:00", NodeType::Departure, Some("g4")),
             5),
        ];

        let mut graph = build_graph_from_gtfs("data/gtfs_example/", "wednesday");

        for edge in transfer_edges {
            let from = to_node_id(edge.0);
            let to = to_node_id(edge.1);
            let cost = edge.2;

            let actual_edge = graph.get_mut_edge(&from, &to);
            assert!(actual_edge.is_some());
            assert_eq!(actual_edge.map(|e| e.weight), Some(cost * 60));
        }
    }
}
