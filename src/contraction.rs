use std::hash::Hash;
use std::collections::HashMap;

use weighted_graph::{ Graph, Node };
use pathfinder::{ CurrentBest, Pathfinder, EdgeIterator };

pub fn contract_graph<T>(graph: &mut Graph<T>,
                         order: &Vec<T>)
       where T: Clone + Eq + Hash {
    for node in order {
        contract_node(graph, &node);
    }
}

fn local_shortest_path<'a, T>(graph: &'a Graph<T>,
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
        rs.len() >= max_nodes || r.cost > max_cost
    };
    let pathfinder = Pathfinder::new(Box::new(identity),
                                     Box::new(edge_iterator),
                                     Box::new(terminator)
                                    );
    pathfinder.shortest_path(graph, source, Some(destination))
}

fn contract_node<T>(graph: &mut Graph<T>, node_id: &T)
   where T: Clone + Eq + Hash {
    let adjacent_nodes = find_adjacent_nodes(graph, node_id);

    for adjacent in &adjacent_nodes {
        remove_from_graph(graph, adjacent, node_id);
    }

    for from_node in &adjacent_nodes {
        for to_node in &adjacent_nodes {
            let weight_across = weight_across_node(graph,
                                                   from_node,
                                                   to_node,
                                                   node_id
                                                  );
            let (min_weight, _) = local_shortest_path(graph,
                                                      from_node,
                                                      to_node,
                                                      20,
                                                      weight_across);

            if min_weight > weight_across {
                add_shortcut(graph, from_node, to_node, weight_across);
            }
        }
    }
}

fn find_adjacent_nodes<T>(graph: &Graph<T>, node_id: &T) -> Vec<T>
    // assuming the graph is symmetric and directed
    // adjacent nodes <=> nodes on outgoing edges
   where T: Clone + Hash + Eq {
    graph.get_edges(node_id)
         .iter()
         .filter(|edge| edge.arc_flag)
         .map(|edge| edge.to_id.clone())
         .collect()
}

fn remove_from_graph<T>(graph: &mut Graph<T>, adjacent_id: &T, node_id: &T)
   where T: Clone + Hash + Eq {
    graph.get_mut_edge(node_id, adjacent_id)
        .map(|edge| edge.arc_flag = false);
    graph.get_mut_edge(adjacent_id, node_id)
            .map(|edge| edge.arc_flag = false);
}

fn weight_across_node<T>(graph: &Graph<T>,
                         from_node: &T,
                         to_node: &T,
                         cur_node: &T) -> i64
   where T: Clone + Hash + Eq {
    edge_weight(graph, from_node, cur_node) + edge_weight(graph, cur_node, to_node)
}

fn edge_weight<T>(graph: &Graph<T>, from_node: &T, to_node: &T) -> i64
   where T: Clone + Hash + Eq {
    graph.get_edges(from_node)
          .iter()
          .find(|edge| edge.to_id == *to_node)
          .map(|edge| edge.weight)
          .unwrap_or(0)
}

fn add_shortcut<T>(graph: &mut Graph<T>, from_node: &T, to_node: &T, weight: i64)
   where T: Clone + Hash + Eq {
    graph.add_edge(from_node.clone(),
                   from_node.clone(),
                   to_node.clone(),
                   weight);
}

#[cfg(test)]
mod test {
    use weighted_graph::{ Graph };
    use super::{ local_shortest_path,
                 contract_node,
                 contract_graph
               };

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
        let mut graph = Graph::new();
        graph.add_node("a", 0.0, 1.0);
        graph.add_node("b", 1.0, 0.0);
        graph.add_node("c", 2.0, 1.0);
        graph.add_node("d", 1.0, 1.0);
        let edges = vec![("a", "b", 1),
                         ("b", "c", 1),
                         ("c", "d", 3),
                         ("d", "a", 3)];
        for (n1, n2, w) in edges {
            graph.add_edge(n1, n1, n2, w);
            graph.add_edge(n2, n2, n1, w);
            graph.get_mut_edge(&n1, &n2).map(|edge| edge.arc_flag = true);
            graph.get_mut_edge(&n2, &n1).map(|edge| edge.arc_flag = true);
        }

        contract_node(&mut graph, &"b");

        let added_ac = graph.get_edges(&"a")
                            .iter()
                            .find(|edge| edge.to_id == "c")
                            .unwrap();
        let added_ca = graph.get_edges(&"c")
                            .iter()
                            .find(|edge| edge.to_id == "a")
                            .unwrap();
        assert!(!added_ac.arc_flag);
        assert_eq!(added_ac.weight, 2);
        assert!(!added_ca.arc_flag);
        assert_eq!(added_ac.weight, 2);

        for edge in graph.get_edges(&"b") {
            assert!(!edge.arc_flag);
        }
        for edge in graph.get_edges(&"a")
                         .iter()
                         .filter(|edge| edge.to_id == "b") {
            assert!(!edge.arc_flag);
        }
        for edge in graph.get_edges(&"c")
                         .iter()
                         .filter(|edge| edge.to_id == "b") {
            assert!(!edge.arc_flag);
        }
    }

    #[test]
    fn contract_node_not_in_shortest_path() {
        let mut graph = Graph::new();
        graph.add_node("a", 0.0, 1.0);
        graph.add_node("b", 1.0, 0.0);
        graph.add_node("c", 2.0, 1.0);
        graph.add_node("d", 1.0, 1.0);
        let edges = vec![("a", "b", 2),
                         ("b", "c", 2),
                         ("c", "d", 1),
                         ("d", "a", 1)];
        for (n1, n2, w) in edges {
            graph.add_edge(n1, n1, n2, w);
            graph.add_edge(n2, n2, n1, w);
            graph.get_mut_edge(&n1, &n2).map(|edge| edge.arc_flag = true);
            graph.get_mut_edge(&n2, &n1).map(|edge| edge.arc_flag = true);
        }

        contract_node(&mut graph, &"b");

        let added_ac = graph.get_edges(&"a")
                            .iter()
                            .find(|edge| edge.to_id == "c");
        let added_ca = graph.get_edges(&"c")
                            .iter()
                            .find(|edge| edge.to_id == "a");
        assert_eq!(added_ac, None);
        assert_eq!(added_ca, None);

        for edge in graph.get_edges(&"b") {
            assert!(!edge.arc_flag);
        }
        for edge in graph.get_edges(&"a")
                         .iter()
                         .filter(|edge| edge.to_id == "b") {
            assert!(!edge.arc_flag);
        }
        for edge in graph.get_edges(&"c")
                         .iter()
                         .filter(|edge| edge.to_id == "b") {
            assert!(!edge.arc_flag);
        }
    }

    #[test]
    fn contract_all_nodes() {
        let mut graph = Graph::new();
        let nodes = vec![("a", 0.0, 3.0),
                         ("b", 0.0, 1.0),
                         ("c", 0.0, 0.0),
                         ("d", 1.0, 3.0),
                         ("e", 1.0, 2.0),
                         ("f", 1.0, 0.0),
                         ("g", 2.0, 3.0),
                         ("h", 2.0, 1.0),
                         ("i", 2.0, 0.0)];
        for &(id, x, y) in &nodes {
            graph.add_node(id, x, y);
        }

        let edges = vec![("a", "b", 3),
                         ("a", "d", 2),
                         ("b", "c", 1),
                         ("b", "e", 1),
                         ("c", "f", 2),
                         ("d", "e", 1),
                         ("d", "g", 2),
                         ("e", "f", 3),
                         ("e", "h", 1),
                         ("f", "i", 2),
                         ("g", "h", 4),
                         ("h", "i", 2),
                        ];
        for &(n1, n2, w) in &edges {
            graph.add_edge(n1, n1, n2, w);
            graph.add_edge(n2, n2, n1, w);
            graph.get_mut_edge(&n1, &n2).map(|edge| edge.arc_flag = true);
            graph.get_mut_edge(&n2, &n1).map(|edge| edge.arc_flag = true);
        }

        let shortcuts = vec![("c", "e", 2), ("e", "g", 3)];

        let node_ids = nodes.iter()
                            .map(|&(id, _, _)| id)
                            .collect::<Vec<&str>>();
        contract_graph(&mut graph, &node_ids);

        for &(id, _, _) in &nodes {
            assert!(graph.get_edges(&id).iter().all(|edge| !edge.arc_flag));
        }
        for &(n1, n2, w) in &shortcuts {
            assert!(graph.get_edges(&n1)
                         .iter()
                         .find(|edge| edge.to_id == n2)
                         .map(|edge| edge.weight == w).unwrap_or(false));
            assert!(graph.get_edges(&n2)
                         .iter()
                         .find(|edge| edge.to_id == n1)
                         .map(|edge| edge.weight == w).unwrap_or(false));
        }

        let edge_count = nodes.iter()
                              .map(|&(id, _, _)| graph.get_edges(&id).len())
                              .fold(0, |sum, l| sum + l);
        assert_eq!(edge_count, shortcuts.len() * 2 + edges.len() * 2);
    }
}
