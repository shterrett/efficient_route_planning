use std::collections::{ HashMap, HashSet };

use pathfinder::CurrentBest;
use weighted_graph::{ Graph };
use graph_from_gtfs::{ GtfsId,
                       StopId
                     };
use set_dijkstra::shortest_path as set_dijkstra;

fn partition_station_nodes<'a>(graph: &'a Graph<GtfsId>) -> HashMap<&'a StopId, HashSet<&'a GtfsId>> {
    let mut partition = HashMap::new();
    for node in graph.all_nodes() {
        let mut nodes = partition.entry(&node.id.stop_id).or_insert(HashSet::new());
        nodes.insert(&node.id);
    }

    partition
}

fn full_dijkstra_from_station<'a>(graph: &'a Graph<GtfsId>,
                              partition: &'a HashMap<&'a StopId, HashSet<&'a GtfsId>>,
                              station: &StopId
                             ) -> HashMap<GtfsId, CurrentBest<GtfsId>> {
    let sources = partition.get(station).unwrap().into_iter().map(|&e| e).collect::<Vec<&GtfsId>>();
    set_dijkstra(graph, &sources, None).1
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use test_helpers::to_node_id;
    use weighted_graph::Graph;
    use graph_from_gtfs::{
        GtfsId,
        build_graph_from_gtfs,
        NodeType
    };
    use super::{
        partition_station_nodes,
        full_dijkstra_from_station
    };

    fn graph() -> Graph<GtfsId> {
        build_graph_from_gtfs("data/gtfs_example/", "wednesday")
    }

    #[test]
    fn assoc_nodes_with_stations() {
        let graph = graph();

        let partition = partition_station_nodes(&graph);

        let station_a = vec![("A", "06:00:00", NodeType::Arrival, Some("r1")),
                             ("A", "06:00:00", NodeType::Departure, Some("r1")),
                             ("A", "06:05:00", NodeType::Transfer, None),
                             ("A", "06:15:00", NodeType::Arrival, Some("g1")),
                             ("A", "06:15:00", NodeType::Departure, Some("g1")),
                             ("A", "06:20:00", NodeType::Transfer, None),
                             ("A", "06:45:00", NodeType::Arrival, Some("g2")),
                             ("A", "06:45:00", NodeType::Departure, Some("g2")),
                             ("A", "06:50:00", NodeType::Transfer, None),
                             ("A", "07:00:00", NodeType::Arrival, Some("r2")),
                             ("A", "07:00:00", NodeType::Departure, Some("r2")),
                             ("A", "07:05:00", NodeType::Transfer, None),
                             ("A", "07:15:00", NodeType::Arrival, Some("g3")),
                             ("A", "07:15:00", NodeType::Departure, Some("g3")),
                             ("A", "07:20:00", NodeType::Transfer, None),
                             ("A", "07:45:00", NodeType::Arrival, Some("g4")),
                             ("A", "07:45:00", NodeType::Departure, Some("g4")),
                             ("A", "07:50:00", NodeType::Transfer, None),
                             ("A", "08:00:00", NodeType::Arrival, Some("r3")),
                             ("A", "08:00:00", NodeType::Departure, Some("r3")),
                             ("A", "08:05:00", NodeType::Transfer, None),
                             ("A", "08:15:00", NodeType::Arrival, Some("g5")),
                             ("A", "08:15:00", NodeType::Departure, Some("g5")),
                             ("A", "08:20:00", NodeType::Transfer, None)];
        let station_b = vec![("B", "06:25:00", NodeType::Arrival, Some("r1")),
                             ("B", "06:25:00", NodeType::Departure, Some("r1")),
                             ("B", "06:30:00", NodeType::Transfer, None),
                             ("B", "07:25:00", NodeType::Arrival, Some("r2")),
                             ("B", "07:25:00", NodeType::Departure, Some("r2")),
                             ("B", "07:30:00", NodeType::Transfer, None),
                             ("B", "08:25:00", NodeType::Arrival, Some("r3")),
                             ("B", "08:25:00", NodeType::Departure, Some("r3")),
                             ("B", "08:30:00", NodeType::Transfer, None)];
        let station_c = vec![("C", "06:45:00", NodeType::Arrival, Some("g1")),
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
                             ("C", "08:50:00", NodeType::Transfer, None)];
        let station_d = vec![("D", "07:00:00", NodeType::Arrival, Some("g1")),
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
                             ("D", "09:05:00", NodeType::Transfer, None)];
        let station_e = vec![("E", "06:50:00", NodeType::Arrival, Some("r1")),
                             ("E", "06:50:00", NodeType::Departure, Some("r1")),
                             ("E", "06:55:00", NodeType::Transfer, None),
                             ("E", "07:30:00", NodeType::Arrival, Some("g1")),
                             ("E", "07:30:00", NodeType::Departure, Some("g1")),
                             ("E", "07:35:00", NodeType::Transfer, None),
                             ("E", "07:50:00", NodeType::Arrival, Some("r2")),
                             ("E", "07:50:00", NodeType::Departure, Some("r2")),
                             ("E", "07:55:00", NodeType::Transfer, None),
                             ("E", "08:00:00", NodeType::Arrival, Some("g2")),
                             ("E", "08:00:00", NodeType::Departure, Some("g2")),
                             ("E", "08:05:00", NodeType::Transfer, None),
                             ("E", "08:30:00", NodeType::Arrival, Some("g3")),
                             ("E", "08:30:00", NodeType::Departure, Some("g3")),
                             ("E", "08:35:00", NodeType::Transfer, None),
                             ("E", "08:50:00", NodeType::Arrival, Some("r3")),
                             ("E", "08:50:00", NodeType::Departure, Some("r3")),
                             ("E", "08:55:00", NodeType::Transfer, None),
                             ("E", "09:00:00", NodeType::Arrival, Some("g4")),
                             ("E", "09:00:00", NodeType::Departure, Some("g4")),
                             ("E", "09:05:00", NodeType::Transfer, None),
                             ("E", "09:30:00", NodeType::Arrival, Some("g5")),
                             ("E", "09:30:00", NodeType::Departure, Some("g5")),
                             ("E", "09:35:00", NodeType::Transfer, None)];
        let station_f = vec![("F", "07:40:00", NodeType::Arrival, Some("g1")),
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
                             ("F", "09:45:00", NodeType::Transfer, None)];

        let station_a_nodes = station_a.into_iter()
                                       .map(|data| to_node_id(data))
                                       .collect::<HashSet<GtfsId>>();
        assert_eq!(*partition.get(&"A".to_string()).unwrap(),
                   station_a_nodes.iter().map(|n| n).collect::<HashSet<&GtfsId>>());

        let station_b_nodes = station_b.into_iter()
                                       .map(|data| to_node_id(data))
                                       .collect::<HashSet<GtfsId>>();
        assert_eq!(*partition.get(&"B".to_string()).unwrap(),
                   station_b_nodes.iter().map(|n| n).collect::<HashSet<&GtfsId>>());

        let station_c_nodes = station_c.into_iter()
                                       .map(|data| to_node_id(data))
                                       .collect::<HashSet<GtfsId>>();
        assert_eq!(*partition.get(&"C".to_string()).unwrap(),
                   station_c_nodes.iter().map(|n| n).collect::<HashSet<&GtfsId>>());

        let station_d_nodes = station_d.into_iter()
                                       .map(|data| to_node_id(data))
                                       .collect::<HashSet<GtfsId>>();
        assert_eq!(*partition.get(&"D".to_string()).unwrap(),
                   station_d_nodes.iter().map(|n| n).collect::<HashSet<&GtfsId>>());

        let station_e_nodes = station_e.into_iter()
                                       .map(|data| to_node_id(data))
                                       .collect::<HashSet<GtfsId>>();
        assert_eq!(*partition.get(&"E".to_string()).unwrap(),
                   station_e_nodes.iter().map(|n| n).collect::<HashSet<&GtfsId>>());

        let station_f_nodes = station_f.into_iter()
                                       .map(|data| to_node_id(data))
                                       .collect::<HashSet<GtfsId>>();
        assert_eq!(*partition.get(&"F".to_string()).unwrap(),
                   station_f_nodes.iter().map(|n| n).collect::<HashSet<&GtfsId>>());

    }

    #[test]
    fn find_all_shortest_paths_from_station() {
        let graph = graph();
        let partition = partition_station_nodes(&graph);

        let shortest_paths = full_dijkstra_from_station(&graph, &partition, &"A".to_string());

        // no transfers
        let spot_check_1 = to_node_id(("F", "09:40:00", NodeType::Arrival, Some("g5")));
        assert_eq!(shortest_paths.get(&spot_check_1).unwrap().cost, 85 * 60);

        // requires a transfer
        let spot_check_2 = to_node_id(("F", "09:10:00", NodeType::Arrival, Some("g4")));
        assert_eq!(shortest_paths.get(&spot_check_2).unwrap().cost, 70 * 60);
    }
}
