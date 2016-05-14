use std::collections::{ BinaryHeap, HashMap };
use std::cmp::Ordering;

use weighted_graph::{ Graph, NodeId };

pub fn shortest_path(graph: &Graph,
                     source: &str,
                     destination: Option<&str>
                    ) -> (i64, HashMap<NodeId, CurrentBest>) {

    let mut min_heap = BinaryHeap::new();
    let mut results = HashMap::new();

    let initial = CurrentBest { id: source.to_string(), cost: 0, predecessor: "".to_string() };
    results.insert(source.to_string(), initial.clone());
    min_heap.push(initial.clone());

    while let Some(current) = min_heap.pop() {
        if let Some(target) = destination {
            if current.id == target {
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
pub struct CurrentBest {
    pub cost: i64,
    pub id: NodeId,
    pub predecessor: NodeId
}

impl Ord for CurrentBest {
    // flip order so min-heap instead of max-heap
    fn cmp(&self, other: &CurrentBest) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for CurrentBest {
    fn partial_cmp(&self, other: &CurrentBest) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use weighted_graph::Graph;
    use super::{ shortest_path, CurrentBest };

    fn build_graph() ->  Graph {
        let mut graph = Graph::new();
        graph.add_node("1".to_string(), 1.0, 1.0);
        graph.add_node("2".to_string(), 1.0, 2.0);
        graph.add_node("3".to_string(), 2.0, 1.0);
        graph.add_node("4".to_string(), 2.0, 2.0);
        graph.add_node("5".to_string(), 2.0, 3.0);
        graph.add_node("6".to_string(), 3.0, 1.0);

        let edges = vec![("a".to_string(), "1".to_string(), "4".to_string(), 1),
                         ("b".to_string(), "4".to_string(), "2".to_string(), 4),
                         ("c".to_string(), "2".to_string(), "5".to_string(), 3),
                         ("d".to_string(), "5".to_string(), "6".to_string(), 3),
                         ("e".to_string(), "6".to_string(), "3".to_string(), 1),
                         ("f".to_string(), "6".to_string(), "4".to_string(), 2)];

        let mut iter = edges.into_iter();

        while let Some((edge_id, node_id_1, node_id_2, cost)) = iter.next() {
            graph.add_edge(edge_id.clone(), node_id_1.clone(), node_id_2.clone(), cost);
            graph.add_edge(edge_id.clone(), node_id_2.clone(), node_id_1.clone(), cost);
        }

        graph
    }

    #[test]
    fn orderable_node_ref() {
        let less = CurrentBest { id: "less".to_string(), cost: 1, predecessor: "".to_string() };
        let more = CurrentBest { id: "more".to_string(), cost: 5, predecessor: "".to_string() };

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
        expected.insert("1".to_string(), CurrentBest { id: "1".to_string(),
                                                       cost: 0,
                                                       predecessor: "".to_string()
                                                });
        expected.insert("2".to_string(), CurrentBest { id: "2".to_string(),
                                                       cost: 5,
                                                       predecessor: "4".to_string()
                                                });
        expected.insert("3".to_string(), CurrentBest { id: "3".to_string(),
                                                       cost: 4,
                                                       predecessor: "6".to_string()
                                                });
        expected.insert("4".to_string(), CurrentBest { id: "4".to_string(),
                                                       cost: 1,
                                                       predecessor: "1".to_string()
                                                });
        expected.insert("5".to_string(), CurrentBest { id: "5".to_string(),
                                                       cost: 6,
                                                       predecessor: "6".to_string()
                                                });
        expected.insert("6".to_string(), CurrentBest { id: "6".to_string(),
                                                       cost: 3,
                                                       predecessor: "4".to_string()
                                                });

        let (_, results) = shortest_path(&graph, "1", None);

        assert_eq!(results, expected);
    }
}
