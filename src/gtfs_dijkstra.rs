use std::collections::HashMap;

use pathfinder::{ Pathfinder, CurrentBest, EdgeIterator };
use weighted_graph::{ Graph, Node };
use graph_from_gtfs::{ GtfsId,
                       StopId,
                     };

pub fn shortest_path<'a>(graph: &'a Graph<GtfsId>,
                         source: &GtfsId,
                         destination: StopId
                        ) -> (i64, HashMap<GtfsId, CurrentBest<GtfsId>>) {
    let identity = |_: Option<&Node<GtfsId>>, _ :Option<&Node<GtfsId>>| 0;
    let edge_iterator = |g: &'a Graph<GtfsId>, node_id: &GtfsId| ->
                        EdgeIterator<'a, GtfsId> {
        Box::new(g.get_edges(node_id).iter().filter(|_| true))
    };
    let terminator = move |current: &CurrentBest<GtfsId>, _: &HashMap<GtfsId, CurrentBest<GtfsId>>|  {
        destination == current.id.stop_id
    };
    let pathfinder = Pathfinder::new(Box::new(identity),
                                     Box::new(edge_iterator),
                                     Box::new(terminator)
                                    );
    pathfinder.shortest_path(graph, source, None)
}

#[cfg(test)]
mod test {
    use weighted_graph::Graph;
    use graph_from_gtfs::{ build_graph_from_gtfs,
                           time_to_seconds_after_midnight,
                           GtfsId,
                           NodeType
                         };
    use super::shortest_path;

    fn build_graph() -> Graph<GtfsId> {
        build_graph_from_gtfs("data/gtfs_example/", "wednesday")
    }

    #[test]
    fn direct_shortest_path() {
        let graph = build_graph();
        let start_time = time_to_seconds_after_midnight(&"06:15:00".to_string()).unwrap();
        let (cost, _) = shortest_path(&graph,
                                      &GtfsId { stop_id: "A".to_string(),
                                                time: start_time,
                                                node_type: NodeType::Arrival,
                                                trip_id: Some("g1".to_string())
                                              },
                                      "F".to_string());

        assert_eq!(cost, 85 * 60);
    }

    #[test]
    fn shortest_path_with_transfer() {
        let graph = build_graph();
        let start_time = time_to_seconds_after_midnight(&"07:00:00".to_string()).unwrap();
        let (cost, _) = shortest_path(&graph,
                                      &GtfsId { stop_id: "A".to_string(),
                                                time: start_time,
                                                node_type: NodeType::Arrival,
                                                trip_id: Some("r2".to_string())
                                              },
                                      "F".to_string());

        assert_eq!(cost, 70 * 60);
    }

    #[test]
    fn start_time_dependent_shortest_path() {
        let graph = build_graph();
        let made_red_line = time_to_seconds_after_midnight(&"07:00:00".to_string()).unwrap();
        let missed_red_line = time_to_seconds_after_midnight(&"07:15:00".to_string()).unwrap();

        let (cost_red, _) = shortest_path(&graph,
                                          &GtfsId { stop_id: "A".to_string(),
                                                    time: made_red_line,
                                                    node_type: NodeType::Arrival,
                                                    trip_id: Some("r2".to_string())
                                                   },
                                          "E".to_string());
        let (cost_green, _) = shortest_path(&graph,
                                            &GtfsId { stop_id: "A".to_string(),
                                                      time: missed_red_line,
                                                      node_type: NodeType::Arrival,
                                                      trip_id: Some("g3".to_string())
                                                     },
                                            "E".to_string());

        assert_eq!(cost_red, 50 * 60);
        assert_eq!(cost_green, 75 * 60);
    }
}
