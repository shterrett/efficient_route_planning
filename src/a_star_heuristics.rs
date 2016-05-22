use std::collections::HashMap;
use std::hash::Hash;
use rand::{thread_rng, Rng};

use weighted_graph::{ Graph, Node };
use road_weights::road_weight;
use dijkstra::shortest_path;

pub type HeuristicFn<T> = Box<Fn(Option<&Node<T>>, Option<&Node<T>>) -> i64>;

pub fn crow_files<T>() -> HeuristicFn<T>
       where T: Clone + Hash + Eq {
    Box::new(|current: Option<&Node<T>>, target: Option<&Node<T>>| {
        match (current, target) {
            (Some(cnode), Some(tnode)) => {
                road_weight(cnode, tnode, "motorway").unwrap_or(0)
            }
            _ => 0
        }
    })
}

pub fn build_landmark_heuristic<T>(graph: &Graph<T>, num_landmarks: usize) -> HeuristicFn<T>
    where T: 'static + Clone + Hash + Eq {
        landmarks(
            build_landmark_distances(
                graph,
                &select_landmarks(graph, num_landmarks)))
}

fn landmarks<T>(landmark_distances: Vec<HashMap<T, i64>>) -> HeuristicFn<T>
       where T: 'static + Clone + Hash + Eq {
    Box::new(move |current: Option<&Node<T>>, target: Option<&Node<T>>| {
        match (current, target) {
            (Some(c_node), Some(t_node)) => {
                landmark_distances.iter().filter_map(|distances|
                    distances.get(&c_node.id)
                             .and_then(|dist|
                                 distances.get(&t_node.id)
                                          .map(|t_dist|
                                               (dist - t_dist).abs()))
                ).max().unwrap_or(0)
            }
            _ => 0
        }
    })
}

fn build_landmark_distances<T>(graph: &Graph<T>, landmarks: &Vec<T>)
   -> Vec<HashMap<T, i64>>
   where T: Clone + Hash + Eq {
       landmarks.iter().map(|landmark_id|
           dijkstra_distances(graph, landmark_id)
       ).collect()
}

fn dijkstra_distances<T>(graph: &Graph<T>, source: &T) -> HashMap<T, i64>
   where T: Clone + Hash + Eq {
    let (_, results) = shortest_path(graph, source, None);
    results.iter().map(|(node_id, results)|
               (node_id.clone(), results.cost)
           ).collect()
}

fn select_landmarks<T>(graph: &Graph<T>, num_landmarks: usize) -> Vec<T>
   where T: Clone + Hash + Eq {
    let mut nodes = graph.all_nodes();
    let slice = nodes.as_mut_slice();

    thread_rng().shuffle(slice);
    slice.iter().take(num_landmarks).map(|node| node.id.clone()).collect()
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::hash::Hash;
    use weighted_graph::{ Graph, Node };
    use road_weights::road_weight;
    use super::{ crow_files,
                 select_landmarks,
                 build_landmark_distances,
                 landmarks
               };

    fn build_graph() -> Graph<&'static str> {
        let mut graph = Graph::new();
        graph.add_node("1", 1.0, 1.0);
        graph.add_node("2", 2.0, 4.0);
        graph.add_node("3", 3.0, 2.0);
        graph.add_node("4", 4.0, 1.0);
        graph.add_node("5", 5.0, 3.0);
        graph.add_node("6", 5.0, 5.0);

        let edges = vec![("a", "1", "2", 5),
                         ("-a", "2", "1", 5),
                         ("b", "2", "6", 2),
                         ("-b", "6", "2", 2),
                         ("c", "1", "3", 3),
                         ("-c", "3", "1", 3),
                         ("d", "3", "5", 3),
                         ("-d", "5", "3", 3),
                         ("e", "3", "4", 2),
                         ("-e", "4", "3", 2),
                         ("f", "4", "5", 3),
                         ("-f", "5", "4", 3),
                         ("g", "5", "6", 4),
                         ("-g", "6", "5", 4)];

        let mut iter = edges.into_iter();

        while let Some((edge_id, node_id_1, node_id_2, cost)) = iter.next() {
            graph.add_edge(edge_id.clone(), node_id_1.clone(), node_id_2.clone(), cost);
        }

        graph
    }

    #[test]
    fn calculate_crow_flying_distance() {
        let node_1 = Node { id: "1", x: 0.0, y: 0.0 };
        let node_2 = Node { id: "2", x: 1.0, y: 1.0 };

        let heuristic = crow_files();

        let expected = road_weight(&node_1, &node_2, "motorway").unwrap();
        let actual = heuristic(Some(&node_1), Some(&node_2));

        assert_eq!(actual, expected);
    }

    #[test]
    fn crow_flies_0_if_none() {
        let node_1 = Node { id: "1", x: 0.0, y: 0.0 };

        let heuristic = crow_files();

        let actual = heuristic(Some(&node_1), None);

        assert_eq!(actual, 0);
    }

    #[test]
    fn pick_landmarks_from_graph() {
        let graph = build_graph();

        for n in 1..6 {
            let landmarks = select_landmarks(&graph, n);
            assert_eq!(landmarks.len(), n);
            for landmark in landmarks {
                assert!(graph.get_node(&landmark).is_some());
            }
        }

    }

    #[test]
    fn build_distances_to_landmarks() {
        let graph = build_graph();

        let landmark_nodes = vec!["2", "3"];

        let distances = build_landmark_distances(&graph, &landmark_nodes);

        let mut results_2 = HashMap::new();
        let mut results_3 = HashMap::new();

        results_2.insert("1", 5);
        results_2.insert("2", 0);
        results_2.insert("3", 8);
        results_2.insert("4", 9);
        results_2.insert("5", 6);
        results_2.insert("6", 2);
        results_3.insert("1", 3);
        results_3.insert("2", 8);
        results_3.insert("3", 0);
        results_3.insert("4", 2);
        results_3.insert("5", 3);
        results_3.insert("6", 7);

        let expected = vec![results_2, results_3];

        assert_eq!(distances, expected);
    }

    #[test]
    fn landmark_heuristic_returns_max_difference_landmark_distance() {
        let graph = build_graph();
        let node_1 = graph.get_node(&"1");
        let node_6 = graph.get_node(&"6");
        let landmark_nodes = vec!["2", "3"];

        let heuristic = landmarks(
                            build_landmark_distances(&graph, &landmark_nodes)
                        );


        assert_eq!(heuristic(node_1, node_6), 4);
    }
}
