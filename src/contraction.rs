use std::collections::{ BinaryHeap, HashMap };
use std::cmp::Ordering;

use weighted_graph::{ GraphKey, Graph, Node };
use pathfinder::{ CurrentBest, Pathfinder, EdgeIterator };

pub fn preprocess_contraction<T>(graph: &mut Graph<T>)
       where T: GraphKey {
    let node_order = preorder_nodes(graph);
    contract_graph(graph, node_order);
    set_increasing_arc_flags(graph);
}

fn contract_graph<T>(graph: &mut Graph<T>,
                     mut order: BinaryHeap<EdgeDifference<T>>)
       where T: GraphKey {
    let mut contraction_order = 0;

    while let Some(next_node) = order.pop() {
        let contracted = graph.get_node(&next_node.node_id)
                              .and_then(|n| n.contraction_order)
                              .is_some();
        if !contracted {
            let edge_difference = contract_node(graph, &next_node.node_id, true);

            if edge_difference <= next_node.edge_difference {
                contraction_order += 1;
                graph.get_mut_node(&next_node.node_id).map(|n| n.contraction_order = Some(contraction_order));
                contract_node(graph, &next_node.node_id, false);
            } else {
                order.push(EdgeDifference { node_id: next_node.node_id,
                                            edge_difference: edge_difference
                                          });
            }
        }
    }
}

fn set_increasing_arc_flags<T>(graph: &mut Graph<T>)
   where T: GraphKey {
    let node_ids: Vec<T> = graph.all_nodes()
                                .iter()
                                .map(|node| node.id.clone())
                                .collect();
    for id in node_ids {
        let current_order = graph.get_node(&id).and_then(|n| n.contraction_order).unwrap();
        let connected_node_ids: Vec<T> = graph.get_edges(&id)
                                                .iter()
                                                .map(|e| e.to_id.clone())
                                                .collect();
        for cid in connected_node_ids {
            if graph.get_node(&cid).and_then(|n| n.contraction_order).unwrap() > current_order {
                graph.get_mut_edge(&id, &cid).map(|e| e.arc_flag = true);
            }
        }
    }
}

fn local_shortest_path<'a, T>(graph: &'a Graph<T>,
                              source: &T,
                              destination: &T,
                              max_nodes: usize,
                              max_cost: i64
                             ) -> (i64, HashMap<T, CurrentBest<T>>)
    where T: GraphKey {
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

fn contract_node<T>(graph: &mut Graph<T>, node_id: &T, count_only: bool) -> i64
   where T: GraphKey {
    let adjacent_nodes = find_adjacent_nodes(graph, node_id);
    // assuming the graph is symmetric and directed
    // edges = 2 * adjacent nodes
    let mut ed: i64 = adjacent_nodes.len() as i64 * 2 * -1;

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
                ed += 1;
                if !count_only {
                    add_shortcut(graph, from_node, to_node, weight_across);
                }
            }
        }
    }
    if count_only {
        for adjacent in &adjacent_nodes {
            unremove_from_graph(graph, adjacent, node_id);
        }
    }
    ed
}

fn find_adjacent_nodes<T>(graph: &Graph<T>, node_id: &T) -> Vec<T>
    // assuming the graph is symmetric and directed
    // adjacent nodes <=> nodes on outgoing edges
   where T: GraphKey {
    graph.get_edges(node_id)
         .iter()
         .filter(|edge| edge.arc_flag)
         .map(|edge| edge.to_id.clone())
         .collect()
}

fn remove_from_graph<T>(graph: &mut Graph<T>, adjacent_id: &T, node_id: &T)
   where T: GraphKey {
    change_arc_flag(graph, adjacent_id, node_id, false);
}

fn unremove_from_graph<T>(graph: &mut Graph<T>, adjacent_id: &T, node_id: &T)
   where T: GraphKey {
    change_arc_flag(graph, adjacent_id, node_id, true);
}

fn change_arc_flag<T>(graph: &mut Graph<T>, adjacent_id: &T, node_id: &T, flag: bool)
   where T: GraphKey {
    graph.get_mut_edge(node_id, adjacent_id)
        .map(|edge| edge.arc_flag = flag);
    graph.get_mut_edge(adjacent_id, node_id)
            .map(|edge| edge.arc_flag = flag);
}

fn weight_across_node<T>(graph: &Graph<T>,
                         from_node: &T,
                         to_node: &T,
                         cur_node: &T) -> i64
   where T: GraphKey {
    edge_weight(graph, from_node, cur_node) + edge_weight(graph, cur_node, to_node)
}

fn edge_weight<T>(graph: &Graph<T>, from_node: &T, to_node: &T) -> i64
   where T: GraphKey {
    graph.get_edges(from_node)
          .iter()
          .find(|edge| edge.to_id == *to_node)
          .map(|edge| edge.weight)
          .unwrap_or(0)
}

fn add_shortcut<T>(graph: &mut Graph<T>, from_node: &T, to_node: &T, weight: i64)
   where T: GraphKey {
    graph.add_edge(from_node.clone(),
                   from_node.clone(),
                   to_node.clone(),
                   weight);
    graph.get_mut_edge(from_node, to_node).map(|edge| edge.arc_flag = true);
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct EdgeDifference<T: GraphKey> {
    node_id: T,
    edge_difference: i64
}

impl<T> Ord for EdgeDifference<T>
        where T: GraphKey {
    // flip order so min-heap instead of max-heap
    fn cmp(&self, other: &EdgeDifference<T>) -> Ordering {
        other.edge_difference.cmp(&self.edge_difference)
    }
}

impl<T> PartialOrd for EdgeDifference<T>
        where T: GraphKey {
    fn partial_cmp(&self, other: &EdgeDifference<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn preorder_nodes<T>(graph: &mut Graph<T>) -> BinaryHeap<EdgeDifference<T>>
   where T: GraphKey {
       let mut preorder = BinaryHeap::new();
       let node_ids: Vec<T> = graph.all_nodes()
                                   .iter()
                                   .map(|node| node.id.clone())
                                   .collect();
       for node_id in node_ids {
           let edge_difference = contract_node(graph, &node_id, true);
           preorder.push(EdgeDifference { node_id: node_id,
                                          edge_difference: edge_difference
                                        });
       }

       preorder
}

#[cfg(test)]
mod test {
    use weighted_graph::{ Graph };
    use arc_flags::shortest_path as arc_flags_shortest_path;
    use super::{ local_shortest_path,
                 contract_node,
                 contract_graph,
                 preorder_nodes,
                 set_increasing_arc_flags,
                 preprocess_contraction
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

        contract_node(&mut graph, &"b", false);

        let added_ac = graph.get_edges(&"a")
                            .iter()
                            .find(|edge| edge.to_id == "c")
                            .unwrap();
        let added_ca = graph.get_edges(&"c")
                            .iter()
                            .find(|edge| edge.to_id == "a")
                            .unwrap();
        assert!(added_ac.arc_flag);
        assert_eq!(added_ac.weight, 2);
        assert!(added_ca.arc_flag);
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
    fn calculate_edge_difference_in_shortest_path() {
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

        let ed = contract_node(&mut graph, &"b", true);
        assert_eq!(ed, 2 - 4);

        for edge in graph.get_edges(&"b") {
            assert!(edge.arc_flag);
        }
        for edge in graph.get_edges(&"a") {
            assert!(edge.arc_flag);
        }
        for edge in graph.get_edges(&"c") {
            assert!(edge.arc_flag);
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

        contract_node(&mut graph, &"b", false);

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
    fn calculate_edge_difference_not_in_shortest_path() {
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

        let ed = contract_node(&mut graph, &"b", true);
        assert_eq!(ed, 0 - 4);

        for edge in graph.get_edges(&"b") {
            assert!(edge.arc_flag);
        }
        for edge in graph.get_edges(&"a") {
            assert!(edge.arc_flag);
        }
        for edge in graph.get_edges(&"c") {
            assert!(edge.arc_flag);
        }
    }

    fn build_full_graph() -> (Vec<(&'static str, f64, f64)>, // nodes
                              Vec<(&'static str, &'static str, i64)>, // edges
                              Graph<&'static str>) {
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

        (nodes, edges, graph)
    }

    #[test]
    fn order_nodes_by_edge_difference() {
        let (_, _, mut graph) = build_full_graph();

        let mut node_order = preorder_nodes(&mut graph);
        let mut current_edge_difference = i64::min_value();

        while let Some(next_node) = node_order.pop() {
            assert!(current_edge_difference <= next_node.edge_difference);
            current_edge_difference = next_node.edge_difference;
        }
        assert_eq!(current_edge_difference, 0);
    }

    #[test]
    fn contract_all_nodes() {
        let (nodes, edges, mut graph) = build_full_graph();

        let node_order = preorder_nodes(&mut graph);
        contract_graph(&mut graph, node_order);

        for &(id, _, _) in &nodes {
            assert!(graph.get_edges(&id).iter().all(|edge| !edge.arc_flag));
            assert!(graph.get_node(&id)
                         .map(|node|
                              node.contraction_order.is_some())
                         .unwrap_or(false))
        }
        let edge_count = nodes.iter()
                              .map(|&(id, _, _)| graph.get_edges(&id).len())
                              .fold(0, |sum, l| sum + l);
        assert!(edge_count >= edges.len() * 2);
    }

    #[test]
    fn mark_edges_where_contraction_order_increases() {
        let (_, _, mut graph) = build_full_graph();

        let node_order = preorder_nodes(&mut graph);
        contract_graph(&mut graph, node_order);

        set_increasing_arc_flags(&mut graph);

        for node in graph.all_nodes() {
            for edge in graph.get_edges(&node.id) {
                if graph.get_node(&edge.to_id)
                        .and_then(|n| n.contraction_order)
                        .unwrap() >
                   node.contraction_order.unwrap() {
                    assert!(edge.arc_flag);
                } else {
                    assert!(!edge.arc_flag);
                }
            }
        }
    }

    #[test]
    fn full_preprocessing_returns_walkable_graph() {
        let (nodes, _, mut graph) = build_full_graph();

        preprocess_contraction(&mut graph);


        for (id, _, _) in nodes {
            let (_, results) = arc_flags_shortest_path(&graph,
                                                       &id,
                                                       None);
            let start_node_contraction = graph.get_node(&id)
                                              .unwrap()
                                              .contraction_order
                                              .unwrap();
            let result_contractions: Vec<i64> = results.keys()
                                                       .map(|id| graph.get_node(id)
                                                                       .unwrap()
                                                                       .contraction_order
                                                                       .unwrap())
                                                       .collect();
            assert!(result_contractions.iter().all(|&co| co >= start_node_contraction));
        }
    }
}
