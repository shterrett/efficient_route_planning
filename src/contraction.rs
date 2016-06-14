use std::hash::Hash;
use std::collections::HashMap;

use weighted_graph::{ Graph, Node };
use pathfinder::{ CurrentBest, Pathfinder, EdgeIterator };

pub fn local_shortest_path<'a, T>(graph: &'a Graph<T>,
                                  source: &T,
                                  destination: &T,
                                  max_nodes: usize,
                                  max_cost: i64
                                 ) -> (i64, HashMap<T, CurrentBest<T>>)
    where T: Clone + Hash + Eq {
    let identity = |_: Option<&Node<T>>, _ :Option<&Node<T>>| 0;
    let edge_iterator = |g: &'a Graph<T>, node_id: &T| ->
                        EdgeIterator<'a, T> {
        Box::new(g.get_edges(node_id).iter().filter(|edge| edge.arc_flag))
    };
    let terminator = move |r: &CurrentBest<T>, rs: &HashMap<T, CurrentBest<T>>| {
        rs.len() >= max_nodes || r.cost >= max_cost
    };
    let pathfinder = Pathfinder::new(Box::new(identity),
                                     Box::new(edge_iterator),
                                     Box::new(terminator)
                                    );
    pathfinder.shortest_path(graph, source, Some(destination))
}

#[cfg(test)]
mod test {
    use weighted_graph::{ Graph };
    use super::local_shortest_path;

    #[test]
    fn local_shortest_path_terminates_early_by_cost() {
        let mut graph = Graph::new();
        graph.add_node("a", 0.0, 0.0);
        graph.add_node("b", 1.0, 1.0);
        graph.add_node("c", 2.0, 2.0);
        graph.add_node("d", 3.0, 3.0);
        graph.add_edge("ab", "a", "b", 2);
        graph.add_edge("bc", "b", "c", 3);
        graph.add_edge("cd", "c", "d", 4);

        for (from, to) in  vec![("a", "b"), ("b", "c"), ("c", "d")] {
            graph.get_mut_edge(&from, &to).map(|edge| edge.arc_flag = true);
        }

        let (cost, _) = local_shortest_path(&graph, &"a", &"d", 10, 4);
        assert_eq!(cost, 5);
    }

    #[test]
    fn local_shortest_path_terminates_early_by_neighborhood() {
        let mut graph = Graph::new();
        graph.add_node("a", 0.0, 0.0);
        graph.add_node("b", 1.0, 1.0);
        graph.add_node("c", 2.0, 2.0);
        graph.add_node("d", 3.0, 3.0);
        graph.add_edge("ab", "a", "b", 2);
        graph.add_edge("bc", "b", "c", 3);
        graph.add_edge("cd", "c", "d", 4);

        for (from, to) in  vec![("a", "b"), ("b", "c"), ("c", "d")] {
            graph.get_mut_edge(&from, &to).map(|edge| edge.arc_flag = true);
        }

        let (_, results) = local_shortest_path(&graph, &"a", &"d", 2, 10);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn local_shortest_path_ignores_arc_flags_false() {
        let mut graph = Graph::new();
        graph.add_node("a", 0.0, 0.0);
        graph.add_node("b", 1.0, 1.0);
        graph.add_node("c", 2.0, 2.0);
        graph.add_node("d", 3.0, 3.0);
        graph.add_edge("ab", "a", "b", 2);
        graph.add_edge("bc", "b", "c", 3);
        graph.add_edge("cd", "c", "d", 4);

        for (from, to) in  vec![("a", "b"), ("b", "c"), ("c", "d")] {
            graph.get_mut_edge(&from, &to).map(|edge| edge.arc_flag = true);
        }

        graph.get_mut_edge(&"c", &"d").map(|edge| edge.arc_flag = false);

        let(_, results) = local_shortest_path(&graph, &"a", &"d", 10, 10);
        assert_eq!(results.len(), 3)
    }

    #[test]
    fn contract_node_in_shortest_path() {
    }

    #[test]
    fn contract_node_not_in_shortest_path() {
    }
}
