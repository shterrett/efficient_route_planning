use std::collections::{ HashSet };
use std::hash::Hash;
use weighted_graph::Graph;
use dijkstra::shortest_path;

pub fn reduce_to_largest_connected_component<T>(graph: Graph<T>) -> Graph<T>
       where T: Clone + Hash + Eq {
    let untested_nodes = node_ids(&graph);
    reducer(graph, untested_nodes, vec![])
}

fn reducer<T>(graph: Graph<T>, untested_nodes: HashSet<T>, mut results: Vec<HashSet<T>>) -> Graph<T>
   where T: Clone + Hash + Eq {
    match untested_nodes.iter().next() {
        None => {
            collapsed_graph(&graph, &results)
        }
        Some(root) => {
            let connected_nodes = explore_from(root, &graph);
            let difference = untested_nodes.difference(&connected_nodes)
                                           .cloned()
                                           .collect();
            results.push(connected_nodes);
            reducer(graph,
                    difference,
                    results
                    )
        }
    }
}

fn explore_from<T: Clone + Hash + Eq>(root: &T, graph: &Graph<T>) -> HashSet<T> {
    let (_, results) = shortest_path(graph, root, None);
    results.values()
           .map(|result| result.id.clone())
           .collect()
}

fn collapsed_graph<T>(graph: &Graph<T>, results: &Vec<HashSet<T>>) -> Graph<T>
   where T: Clone + Hash + Eq {
    let mut new_graph = Graph::new();
    if let Some(nodes) = results.iter().max_by_key(|results| results.len()) {
        for node_id in nodes {
            add_node(graph, &mut new_graph, &node_id);
        }
        for node_id in nodes {
            add_edges(graph, &mut new_graph, &node_id);
        }
    }
    new_graph
}

fn add_node<T>(old_graph: &Graph<T>, mut new_graph: &mut Graph<T>, id: &T)
   where T: Clone + Hash + Eq {
    if let Some(node) = old_graph.get_node(id) {
        new_graph.add_node(id.clone(),
                           node.x,
                           node.y);
    }
}

fn add_edges<T>(old_graph: &Graph<T>, mut new_graph: &mut Graph<T>, id: &T)
   where T: Clone + Hash + Eq {
    for edge in old_graph.get_edges(id) {
        new_graph.add_edge(edge.id.clone(),
                            id.clone(),
                            edge.to_id.clone(),
                            edge.weight);
    }
}

fn node_ids<T>(graph: &Graph<T>) -> HashSet<T>
   where T: Clone + Hash + Eq {
    graph.all_nodes()
        .iter()
        .map(|node| node.id.clone())
        .collect::<HashSet<T>>()
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use weighted_graph::Graph;
    use super::{ reduce_to_largest_connected_component,
                 node_ids,
                 explore_from,
                 add_node,
                 add_edges
               };

    fn build_graph() ->  Graph<&'static str> {
        let mut graph = Graph::new();
        graph.add_node("1", 1.0, 1.0);
        graph.add_node("2", 1.0, 2.0);
        graph.add_node("3", 2.0, 1.0);
        graph.add_node("4", 2.0, 2.0);
        graph.add_node("5", 2.0, 3.0);
        graph.add_node("6", 3.0, 1.0);
        graph.add_node("7", 4.0, 2.0);
        graph.add_node("8", 5.0, 3.0);
        graph.add_node("9", 5.0, 2.0);

        let edges = vec![("a", "1", "4", 1),
                         ("b", "4", "2", 4),
                         ("c", "2", "5", 3),
                         ("d", "5", "6", 3),
                         ("e", "6", "3", 1),
                         ("f", "6", "4", 2),
                         ("g", "7", "8", 1),
                         ("h", "8", "9", 3),
                         ("i", "9", "7", 2)];

        let mut iter = edges.into_iter();

        while let Some((edge_id, node_id_1, node_id_2, cost)) = iter.next() {
            graph.add_edge(edge_id.clone(), node_id_1.clone(), node_id_2.clone(), cost);
            graph.add_edge(edge_id.clone(), node_id_2.clone(), node_id_1.clone(), cost);
        }

        graph
    }

    #[test]
    fn initial_node_ids() {
        let graph = build_graph();

        let expected: HashSet<&str> =  vec!["1",
                                            "2",
                                            "3",
                                            "4",
                                            "5",
                                            "6",
                                            "7",
                                            "8",
                                            "9"].into_iter().collect();

        let nodes = node_ids(&graph);

        assert_eq!(nodes, expected);
    }

    #[test]
    fn return_connected_nodes() {
        let graph = build_graph();

        let root = "9";

        let connected_nodes = explore_from(&root, &graph);
        let small_connection: HashSet<&str> = vec!["7",
                                                   "8",
                                                   root].into_iter()
                                                        .collect();

        assert_eq!(connected_nodes, small_connection);
    }

    #[test]
    fn test_add_node() {
        let graph = build_graph();
        let node_id = "1";
        let mut new_graph = Graph::new();

        add_node(&graph, &mut new_graph, &node_id);

        assert_eq!(new_graph.get_node(&node_id), graph.get_node(&node_id));
    }

    #[test]
    fn test_add_edges() {
        let graph = build_graph();
        let node_id = "7";
        let mut new_graph = Graph::new();

        add_node(&graph, &mut new_graph, &node_id);
        add_node(&graph, &mut new_graph, &"9");
        add_edges(&graph, &mut new_graph, &node_id);
        add_edges(&graph, &mut new_graph, &"9");

        let nodes_from_seven: Vec<&str> = new_graph.get_edges(&node_id)
                                                   .iter()
                                                   .map(|edge| edge.to_id)
                                                   .collect();
        let nodes_from_nine: Vec<&str> = new_graph.get_edges(&"9")
                                                  .iter()
                                                  .map(|edge| edge.to_id)
                                                  .collect();

        assert_eq!(nodes_from_seven, vec!["9"]);
        assert_eq!(nodes_from_nine, vec!["7"]);
    }

    #[test]
    fn find_connected_component() {
        let graph = build_graph();

        assert!(graph.get_node(&"7").is_some());
        assert!(graph.get_node(&"8").is_some());
        assert!(graph.get_node(&"9").is_some());

        let connected_graph = reduce_to_largest_connected_component(graph);

        assert!(connected_graph.get_node(&"7").is_none());
        assert!(connected_graph.get_node(&"8").is_none());
        assert!(connected_graph.get_node(&"9").is_none());
        assert!(connected_graph.get_node(&"1").is_some());
        assert!(connected_graph.get_node(&"2").is_some());
        assert!(connected_graph.get_node(&"3").is_some());
        assert!(connected_graph.get_node(&"4").is_some());
        assert!(connected_graph.get_node(&"5").is_some());
        assert!(connected_graph.get_node(&"6").is_some());
    }
}
