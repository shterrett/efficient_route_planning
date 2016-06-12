use std::collections::HashMap;
use std::hash::Hash;

use weighted_graph::Graph;
use pathfinder::{ Pathfinder, CurrentBest, HeuristicFn, EdgeIterator };

pub fn shortest_path<'a, T>(graph: &'a Graph<T>,
                            source: &T,
                            destination: Option<&T>,
                            heuristic: HeuristicFn<'a, T>
                           ) -> (i64, HashMap<T, CurrentBest<T>>)
   where T: Clone + Hash + Eq {
    let edge_iterator = |g: &'a Graph<T>, node_id: &T| ->
                        EdgeIterator<'a, T> {
        Box::new(g.get_edges(node_id).iter().filter(|_| true))
    };
    let terminator = |_: &CurrentBest<T>, _: &HashMap<T, CurrentBest<T>>| false;
    let pathfinder = Pathfinder::new(heuristic,
                                     Box::new(edge_iterator),
                                     Box::new(terminator)
                                    );
    pathfinder.shortest_path(graph, source, destination)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use weighted_graph::{ Graph, Node };
    use super::shortest_path;

    fn build_graph() ->  Graph<&'static str> {
        let mut graph = Graph::new();
        graph.add_node("1", 1.0, 1.0);
        graph.add_node("2", 2.0, 4.0);
        graph.add_node("3", 3.0, 2.0);
        graph.add_node("4", 4.0, 1.0);
        graph.add_node("5", 5.0, 3.0);
        graph.add_node("6", 5.0, 5.0);

        let edges = vec![("a", "1", "2", 5),
                         ("b", "2", "6", 2),
                         ("c", "1", "3", 3),
                         ("d", "3", "5", 3),
                         ("e", "3", "4", 2),
                         ("f", "4", "5", 3),
                         ("g", "5", "6", 4)];

        let mut iter = edges.into_iter();

        while let Some((edge_id, node_id_1, node_id_2, cost)) = iter.next() {
            graph.add_edge(edge_id.clone(), node_id_1.clone(), node_id_2.clone(), cost);
            graph.add_edge(edge_id.clone(), node_id_2.clone(), node_id_1.clone(), cost);
        }

        graph
    }

    #[test]
    fn uses_heuristic_short_circuit() {
        let graph = build_graph();
        let identity = |_: Option<&Node<&str>>, _: Option<&Node<&str>>| 0;
        let mut h = HashMap::new();
        h.insert("1", 6);
        h.insert("2", 1);
        h.insert("3", 6);
        h.insert("4", 7);
        h.insert("5", 3);
        h.insert("6", 0);

        let heuristic = move |current: Option<&Node<&str>>, _: Option<&Node<&str>>| {
            *current.and_then(|node| h.get(&node.id)).unwrap()
        };

        let (_, naive) = shortest_path(&graph, &"1", Some(&"6"), Box::new(identity));
        let(_, heuristified) = shortest_path(&graph, &"1", Some(&"6"), Box::new(heuristic));

        assert_eq!(naive.get(&"4").map(|b| b.cost), Some(5));
        assert_eq!(heuristified.get(&"4"), None);
        assert_eq!(naive.get(&"5").map(|b| b.cost), Some(6));
        assert_eq!(heuristified.get(&"5"), None);
    }
}
