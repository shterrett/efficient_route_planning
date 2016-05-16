use std::collections::{ BinaryHeap, HashMap };
use std::hash::Hash;
use std::cmp::Ordering;

use weighted_graph::Graph;

pub fn shortest_path<T>(graph: &Graph<T>,
                     source: &T,
                     destination: Option<&T>
                    ) -> (i64, HashMap<T, CurrentBest<T>>)
   where T: Clone + Hash + Eq {

    let mut min_heap = BinaryHeap::new();
    let mut results = HashMap::new();

    let initial = CurrentBest { id: source.clone(), cost: 0, predecessor: source.clone() };
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
                        let hnode = CurrentBest { id: node.id.clone(),
                                                  cost: current.cost + edge.weight,
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
    use weighted_graph::Graph;
    use super::{ shortest_path, CurrentBest };

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
    fn orderable_node_ref() {
        let less = CurrentBest { id: "less", cost: 1, predecessor: "" };
        let more = CurrentBest { id: "more", cost: 5, predecessor: "" };

        assert!(less > more);
        assert!(more < less);
    }

    #[test]
    fn graph() {
        let graph = build_graph();
        assert!(graph.get_node("3").is_some());
        assert!(graph.get_edges("3").is_some());
    }

    #[test]
    fn find_shortest_path() {
        let graph = build_graph();

        let (cost, _) = shortest_path(&graph, "1", Some("5"));

        assert_eq!(cost, 6);
    }

    #[test]
    fn find_all_shortest_paths() {
        let graph = build_graph();
        let mut expected = HashMap::new();
        expected.insert("1", CurrentBest { id: "1",
                                           cost: 0,
                                           predecessor: ""
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

        let (_, results) = shortest_path(&graph, "1", None);

        assert_eq!(results, expected);
    }
}
