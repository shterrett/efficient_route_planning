use std::collections::{ HashMap, HashSet };

use pathfinder::CurrentBest;
use weighted_graph::{ Graph };
use graph_from_gtfs::{ GtfsId,
                       StopId,
                       NodeType
                     };
use set_dijkstra::shortest_path as set_dijkstra;

pub fn transfer_patterns_for_all_stations(graph: &Graph<GtfsId>
                                         ) -> HashMap<(StopId, StopId), HashSet<Vec<StopId>>> {
    let partition = partition_station_nodes(&graph);

    let pairs = station_pairs(partition.keys().collect::<Vec<&&StopId>>());
    pairs.iter().fold(HashMap::new(), |mut transfers, station_pair| {
        let dijkstra_results = full_dijkstra_from_station(&graph,
                                                          &partition,
                                                          &station_pair.0);
        let partitioned_dijkstra = partition_dijkstra_results(&dijkstra_results);
        if let Some(destination_node) = partitioned_dijkstra.get(&station_pair.1) {
            let smoothed = smooth_results(destination_node);
            transfers.insert((station_pair.0.clone(), station_pair.1.clone()),
                            transfer_patterns_for_station_pair(&dijkstra_results, &smoothed));
        }
        transfers
    })
}

fn station_pairs<'a>(stations: Vec<&&'a StopId>) -> Vec<(&'a StopId, &'a StopId)> {
    let mut pairs = vec![];
    for &s1 in &stations {
        for &s2 in &stations {
            pairs.push((*s1, *s2));
        }
    }
    pairs
}

fn partition_station_nodes<'a>(graph: &'a Graph<GtfsId>) -> HashMap<&'a StopId, HashSet<&'a GtfsId>> {
    graph.all_nodes().iter().fold(HashMap::new(), |mut partition, node| {
        partition.entry(&node.id.stop_id).or_insert(HashSet::new()).insert(&node.id);
        partition
    })
}

fn full_dijkstra_from_station<'a>(graph: &'a Graph<GtfsId>,
                              partition: &'a HashMap<&'a StopId, HashSet<&'a GtfsId>>,
                              station: &StopId
                             ) -> HashMap<GtfsId, CurrentBest<GtfsId>> {
    let sources = partition.get(station).unwrap().into_iter().map(|&e| e).collect::<Vec<&GtfsId>>();
    set_dijkstra(graph, &sources, None).1
}

fn partition_dijkstra_results<'a>(results: &'a HashMap<GtfsId, CurrentBest<GtfsId>>)
                              -> HashMap<&'a StopId, Vec<&'a CurrentBest<GtfsId>>> {
    let mut partition = results.iter()
                               .filter(|&(node_id, _)| node_id.node_type == NodeType::Arrival)
                               .fold(HashMap::new(), |mut p, (node_id, node_result)| {
                                   p.entry(&node_id.stop_id).or_insert(vec![]).push(node_result);
                                   p
                               });
    for mut nodes in partition.values_mut() {
        nodes.sort_by(|&a, &b| a.id.time.cmp(&b.id.time));
    }
    partition
}

fn smooth_results(results: &Vec<&CurrentBest<GtfsId>>) -> Vec<CurrentBest<GtfsId>> {
    results.windows(2).fold(vec![results[0].clone()], |mut smoothed, nodes| {
        let prev = nodes[0];
        let curr = nodes[1];
        let wait_cost = prev.cost + (curr.id.time - prev.id.time);
        if curr.cost > wait_cost {
            smoothed.push(CurrentBest { id: curr.id.clone(),
                                        cost: wait_cost,
                                        predecessor: Some(prev.id.clone())
                                      });
        } else {
            smoothed.push(curr.clone());
        }
        smoothed
    })
}

fn transfer_patterns_for_station_pair(dijkstra_results: &HashMap<GtfsId, CurrentBest<GtfsId>>,
                                      smoothed: &Vec<CurrentBest<GtfsId>>
                                     )
                                     -> HashSet<Vec<StopId>> {
    smoothed.iter().fold(HashSet::new(), |mut patterns, node| {
        patterns.insert(collect_transfer_points(dijkstra_results, node));
        patterns
    })
}

fn collect_transfer_points(dijkstra_results: &HashMap<GtfsId, CurrentBest<GtfsId>>,
                           final_node: &CurrentBest<GtfsId>,
                          )
                           -> Vec<StopId> {
    let path = backtrack(dijkstra_results, final_node);
    let mut transfers = path.iter().fold(vec![], |mut points, next_node| {
        if points.last().is_none() || next_node.node_type.is_transfer() {
            points.push(next_node);
        }
        points
    });
    if transfers.last().map(|node| node.stop_id != final_node.id.stop_id).unwrap_or(true) {
        transfers.push(&final_node.id);
    }

    transfers.iter().map(|node| node.stop_id.clone()).collect()

}

fn backtrack(dijkstra_results: &HashMap<GtfsId, CurrentBest<GtfsId>>,
             current: &CurrentBest<GtfsId>
            ) -> Vec<GtfsId> {
    match current.predecessor {
        Some(ref predecessor) => {
            let mut path = dijkstra_results.get(&predecessor)
                                           .map(|cb| backtrack(dijkstra_results, cb))
                                           .unwrap_or(vec![]);
            path.push(current.id.clone());
            path
        }
        None => {
            vec![current.id.clone()]
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::{ HashSet, HashMap };
    use test_helpers::to_node_id;
    use weighted_graph::Graph;
    use pathfinder::CurrentBest;
    use graph_from_gtfs::{
        GtfsId,
        build_graph_from_gtfs,
        NodeType
    };
    use super::{
        partition_station_nodes,
        full_dijkstra_from_station,
        partition_dijkstra_results,
        smooth_results,
        transfer_patterns_for_station_pair,
        transfer_patterns_for_all_stations
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

    #[test]
    fn results_partition_by_station_and_filtered_to_arrivals() {
        let first_a_arrival = CurrentBest {
                                id: to_node_id(("A", "09:40:00", NodeType::Arrival, Some("g5"))),
                                cost: 5,
                                predecessor: None
                               };
        let second_a_arrival = CurrentBest {
                                id: to_node_id(("A", "10:40:00", NodeType::Arrival, Some("g5"))),
                                cost: 5,
                                predecessor: None
                               };
        let first_b_arrival = CurrentBest {
                                id: to_node_id(("B", "09:40:00", NodeType::Arrival, Some("g5"))),
                                cost: 5,
                                predecessor: None
                              };

        let result_data = vec![CurrentBest {
                                id: to_node_id(("A", "10:40:00", NodeType::Arrival, Some("g5"))),
                                cost: 5,
                                predecessor: None
                               },
                               CurrentBest {
                                id: to_node_id(("A", "09:40:00", NodeType::Departure, Some("g5"))),
                                cost: 5,
                                predecessor: None
                               },
                               CurrentBest {
                                id: to_node_id(("A", "09:40:00", NodeType::Transfer, Some("g5"))),
                                cost: 5,
                                predecessor: None
                               },
                               CurrentBest {
                                id: to_node_id(("A", "09:40:00", NodeType::Arrival, Some("g5"))),
                                cost: 5,
                                predecessor: None
                               },
                               CurrentBest {
                                id: to_node_id(("B", "09:40:00", NodeType::Arrival, Some("g5"))),
                                cost: 5,
                                predecessor: None
                               }];

        let results = &result_data.iter()
                                 .map(|result| (result.id.clone(), result.clone()))
                                 .collect::<HashMap<GtfsId, CurrentBest<GtfsId>>>();

        let partition = partition_dijkstra_results(&results);

        let stop_a = &"A".to_string();
        let stop_b = &"B".to_string();
        let mut expected_partition = HashMap::new();
        expected_partition.insert(stop_a, vec![]);
        expected_partition.insert(stop_b, vec![]);
        expected_partition.get_mut(&stop_a).map(|mut rs| rs.push(&first_a_arrival));
        expected_partition.get_mut(&stop_a).map(|mut rs| rs.push(&second_a_arrival));
        expected_partition.get_mut(&stop_b).map(|mut rs| rs.push(&first_b_arrival));

        assert_eq!(partition, expected_partition);
    }

    #[test]
    fn modify_arrival_times_and_paths() {
        let result_1 = CurrentBest { id: to_node_id(("E", "09:40:00", NodeType::Arrival, Some("g5"))),
                                    cost: 3000,
                                    predecessor: Some(to_node_id(("D", "09:30:00", NodeType::Departure, Some("g5"))))
                                   };
        let result_2 = CurrentBest { id: to_node_id(("E", "10:00:00", NodeType::Arrival, Some("g6"))),
                                    cost: 3000,
                                    predecessor: Some(to_node_id(("D", "09:50:00", NodeType::Departure, Some("g6"))))
                                   };
        let result_3 = CurrentBest { id: to_node_id(("E", "10:20:00", NodeType::Arrival, Some("r3"))),
                                     cost: 7000,
                                     predecessor: Some(to_node_id(("C", "10:00:00", NodeType::Departure, Some("r3"))))
                                   };

        let results = vec![&result_1, &result_2, &result_3];

        let cleaned = smooth_results(&results);

        let smooth_results = cleaned.iter()
                                    .map(|cb| (cb.cost, cb.predecessor.clone().unwrap()))
                                    .collect::<Vec<(i64, GtfsId)>>();
        let expected = vec![(3000, result_1.clone().predecessor.unwrap()),
                            (3000, result_2.clone().predecessor.unwrap()),
                            (3000 + 20 * 60, result_2.clone().id)];

        assert_eq!(smooth_results, expected);
    }

    #[test]
    fn find_transfer_patterns_for_single_station_pair() {
        let origin_station = "A".to_string();
        let destination_station = "F".to_string();
        let graph = graph();

        let partition = partition_station_nodes(&graph);
        let dijkstra_results = full_dijkstra_from_station(&graph,
                                                          &partition,
                                                          &origin_station);
        let partitioned_dijkstra = partition_dijkstra_results(&dijkstra_results);
        let smoothed = smooth_results(partitioned_dijkstra.get(&destination_station).unwrap());

        let transfer_patterns = transfer_patterns_for_station_pair(&dijkstra_results,
                                                                   &smoothed);

        let mut expected = HashSet::new();
        expected.insert(vec!["A".to_string(), "E".to_string(), "F".to_string()]);
        expected.insert(vec!["A".to_string(), "F".to_string()]);

        assert_eq!(transfer_patterns, expected);
    }

    #[test]
    fn find_all_transfer_patterns() {
        let graph = graph();
        let stations = vec!["A", "B", "C", "D", "E", "F"];
        let mut station_pairs = HashSet::new();
        for i in &stations {
            for j in &stations {
                station_pairs.insert((i.to_string(), j.to_string()));
            }
        }

        let all_transfer_patterns = transfer_patterns_for_all_stations(&graph);

        for key in &station_pairs {
            let partition = partition_station_nodes(&graph);
            let dijkstra_results = full_dijkstra_from_station(&graph,
                                                              &partition,
                                                              &key.0);
            let partitioned_dijkstra = partition_dijkstra_results(&dijkstra_results);
            if let Some(destination_node) = partitioned_dijkstra.get(&key.1) {
                let smoothed = smooth_results(destination_node);
                    assert_eq!(all_transfer_patterns.get(&key).unwrap(),
                            &transfer_patterns_for_station_pair(&dijkstra_results, &smoothed));
            }
        }
    }
}
