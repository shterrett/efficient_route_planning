use std::collections::{ BinaryHeap, HashMap };
use std::hash::Hash;
use std::cmp::Ordering;

use weighted_graph::{ Graph, Node };

pub fn shortest_path<T, F>(graph: &Graph<T>,
                     source: &T,
                     destination: Option<&T>,
                     heuristic: F
                    ) -> (i64, HashMap<T, CurrentBest<T>>)
   where T: Clone + Hash + Eq,
         F: Fn(Option<&Node<T>>, Option<&Node<T>>) -> i64 {

    let mut min_heap = BinaryHeap::new();
    let mut results = HashMap::new();

    let initial = CurrentBest { id: source.clone(),
                                cost: heuristic(graph.get_node(source),
                                                destination.and_then(|id|
                                                    graph.get_node(id)
                                                )
                                               ),
                                predecessor: source.clone()
                              };
    results.insert(source.clone(), initial.clone());
    min_heap.push(initial.clone());

    while let Some(current) = min_heap.pop() {
        if let Some(target) = destination {
            if current.id == *target {
                return (current.cost, results)
            }
        }

        if let Some(edges) = graph.get_edges(&current.id) {
            for edge in edges.iter() {
                if let Some(node) = graph.get_node(&edge.to_id) {
                    let node_cost = results.get(&node.id)
                                           .map_or(i64::max_value(), |node| node.cost);
                    if current.cost + edge.weight < node_cost {
                        let cost = current.cost +
                                   edge.weight +
                                   heuristic(Some(&node),
                                             destination.and_then(|id| graph.get_node(id))
                                            );
                        let hnode = CurrentBest { id: node.id.clone(),
                                                  cost: cost,
                                                  predecessor: current.id.clone()
                                                };
                        min_heap.push(hnode.clone());
                        results.insert(node.id.clone(), hnode.clone());
                    }
                }
            }
        }
    }
    (0, results)
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct CurrentBest<T: Clone + Hash + Eq> {
    pub cost: i64,
    pub id: T,
    pub predecessor: T
}

impl<T> Ord for CurrentBest<T>
        where T: Clone + Hash + Eq {
    // flip order so min-heap instead of max-heap
    fn cmp(&self, other: &CurrentBest<T>) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl<T> PartialOrd for CurrentBest<T>
        where T: Clone + Hash + Eq {
    fn partial_cmp(&self, other: &CurrentBest<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use weighted_graph::{ Graph, Node };
    use super::{ shortest_path, CurrentBest };

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
    fn orderable_node_ref() {
        let less = CurrentBest { id: "less", cost: 1, predecessor: "" };
        let more = CurrentBest { id: "more", cost: 5, predecessor: "" };

        assert!(less > more);
        assert!(more < less);
    }

    #[test]
    fn reduction_to_dijkstra() {
        let graph = build_graph();

        let identity = |_: Option<&Node<&str>>, _: Option<&Node<&str>>| 0;

        let (cost, _) = shortest_path(&graph, &"1", Some(&"6"), identity);
        assert_eq!(cost, 7);
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

        let heuristic = |current: Option<&Node<&str>>, _: Option<&Node<&str>>| {
            *current.and_then(|node| h.get(&node.id)).unwrap()
        };

        let (_, naive) = shortest_path(&graph, &"1", Some(&"6"), identity);
        let(_, heuristified) = shortest_path(&graph, &"1", Some(&"6"), heuristic);

        println!("{:?}", heuristified);
        assert_eq!(naive.get(&"4").map(|b| b.cost), Some(5));
        assert_eq!(heuristified.get(&"4"), None);
        assert_eq!(naive.get(&"5").map(|b| b.cost), Some(6));
        assert_eq!(heuristified.get(&"5"), None);
    }
}
