use std::collections::HashMap;
use std::hash::Hash;

use pathfinder::{ Pathfinder, CurrentBest, EdgeIterator };
use weighted_graph::{ Graph, Node };

pub fn shortest_path<'a, T>(graph: &'a Graph<T>,
                            source: &T,
                            destination: Option<&T>
                           ) -> (i64, HashMap<T, CurrentBest<T>>)
    where T: Clone + Hash + Eq {
    let identity = |_: Option<&Node<T>>, _ :Option<&Node<T>>| 0;
    let edge_iterator = |g: &'a Graph<T>, node_id: &T| ->
                        EdgeIterator<'a, T> {
        Box::new(g.get_edges(node_id).iter().filter(|_| true))
    };
    let pathfinder = Pathfinder::new(Box::new(identity),
                                     Box::new(edge_iterator)
                                    );
    pathfinder.shortest_path(graph, source, destination)
}

#[cfg(test)]
mod test {
    use super::shortest_path;
    use pathfinder::CurrentBest;
    use weighted_graph::Graph;
    use std::collections::HashMap;

    fn build_graph() ->  Graph<&'static str> {
        let mut graph = Graph::new();
        graph.add_node("1", 1.0, 1.0);
        graph.add_node("2", 1.0, 2.0);
        graph.add_node("3", 2.0, 1.0);
        graph.add_node("4", 2.0, 2.0);
        graph.add_node("5", 2.0, 3.0);
        graph.add_node("6", 3.0, 1.0);

        let edges = vec![("a", "1", "4", 1),
                         ("b", "4", "2", 4),
                         ("c", "2", "5", 3),
                         ("d", "5", "6", 3),
                         ("e", "6", "3", 1),
                         ("f", "6", "4", 2)];

        let mut iter = edges.into_iter();

        while let Some((edge_id, node_id_1, node_id_2, cost)) = iter.next() {
            graph.add_edge(edge_id.clone(), node_id_1.clone(), node_id_2.clone(), cost);
            graph.add_edge(edge_id.clone(), node_id_2.clone(), node_id_1.clone(), cost);
        }

        graph
    }

    #[test]
    fn find_shortest_path() {
        let graph = build_graph();

        let (cost, _) = shortest_path(&graph, &"1", Some(&"5"));

        assert_eq!(cost, 6);
    }

    #[test]
    fn find_all_shortest_paths() {
        let graph = build_graph();
        let mut expected = HashMap::new();
        expected.insert("1", CurrentBest { id: "1",
                                           cost: 0,
                                           predecessor: "1"
                                         });
        expected.insert("2", CurrentBest { id: "2",
                                           cost: 5,
                                           predecessor: "4"
                                         });
        expected.insert("3", CurrentBest { id: "3",
                                           cost: 4,
                                           predecessor: "6"
                                         });
        expected.insert("4", CurrentBest { id: "4",
                                           cost: 1,
                                           predecessor: "1"
                                         });
        expected.insert("5", CurrentBest { id: "5",
                                           cost: 6,
                                           predecessor: "6"
                                         });
        expected.insert("6", CurrentBest { id: "6",
                                           cost: 3,
                                           predecessor: "4"
                                         });

        let (_, results) = shortest_path(&graph, &"1", None);

        assert_eq!(results, expected);
    }
}
