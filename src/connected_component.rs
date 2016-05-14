use std::collections::{ HashSet };
use weighted_graph::{ Graph, NodeId };
use dijkstra::shortest_path;

pub fn reduce_to_largest_connected_component(graph: Graph) -> Graph {
    let untested_nodes = node_ids(&graph);
    reducer(graph, untested_nodes, vec![])
}

fn reducer(graph: Graph, untested_nodes: HashSet<NodeId>, mut results: Vec<HashSet<NodeId>>) -> Graph {
    match untested_nodes.iter().next() {
        None => {
            collapsed_graph(&graph, &results)
        }
        Some(root) => {
            let connected_nodes = explore_from(&root, &graph);
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

fn explore_from(root: &NodeId, graph: &Graph) -> HashSet<NodeId> {
    let (_, results) = shortest_path(graph, root, None);
    results.values()
           .map(|result| result.id.clone())
           .collect()
}

fn collapsed_graph(graph: &Graph, results: &Vec<HashSet<NodeId>>) -> Graph {
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

fn add_node(old_graph: &Graph, mut new_graph: &mut Graph, id: &NodeId) {
    if let Some(node) = old_graph.get_node(id) {
        new_graph.add_node(id.clone(),
                        node.x,
                        node.y);
    }
}

fn add_edges(old_graph: &Graph, mut new_graph: &mut Graph, id: &NodeId) {
    if let Some(edges) = old_graph.get_edges(id) {
        for edge in edges {
            new_graph.add_edge(edge.id.clone(),
                                id.clone(),
                                edge.to_id.clone(),
                                edge.weight);
        }
    }
}

fn node_ids(graph: &Graph) -> HashSet<NodeId> {
    graph.all_nodes()
        .iter()
        .map(|node| node.id.clone())
        .collect::<HashSet<NodeId>>()
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use weighted_graph::{ Graph, NodeId };
    use super::{ reduce_to_largest_connected_component,
                 node_ids,
                 explore_from,
                 add_node,
                 add_edges
               };

    fn build_graph() ->  Graph {
        let mut graph = Graph::new();
        graph.add_node("1".to_string(), 1.0, 1.0);
        graph.add_node("2".to_string(), 1.0, 2.0);
        graph.add_node("3".to_string(), 2.0, 1.0);
        graph.add_node("4".to_string(), 2.0, 2.0);
        graph.add_node("5".to_string(), 2.0, 3.0);
        graph.add_node("6".to_string(), 3.0, 1.0);
        graph.add_node("7".to_string(), 4.0, 2.0);
        graph.add_node("8".to_string(), 5.0, 3.0);
        graph.add_node("9".to_string(), 5.0, 2.0);

        let edges = vec![("a".to_string(), "1".to_string(), "4".to_string(), 1),
                         ("b".to_string(), "4".to_string(), "2".to_string(), 4),
                         ("c".to_string(), "2".to_string(), "5".to_string(), 3),
                         ("d".to_string(), "5".to_string(), "6".to_string(), 3),
                         ("e".to_string(), "6".to_string(), "3".to_string(), 1),
                         ("f".to_string(), "6".to_string(), "4".to_string(), 2),
                         ("g".to_string(), "7".to_string(), "8".to_string(), 1),
                         ("h".to_string(), "8".to_string(), "9".to_string(), 3),
                         ("i".to_string(), "9".to_string(), "7".to_string(), 2)];

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

        let expected: HashSet<NodeId> =  vec!["1".to_string(),
                                              "2".to_string(),
                                              "3".to_string(),
                                              "4".to_string(),
                                              "5".to_string(),
                                              "6".to_string(),
                                              "7".to_string(),
                                              "8".to_string(),
                                              "9".to_string()].into_iter().collect();

        let nodes = node_ids(&graph);

        assert_eq!(nodes, expected);
    }

    #[test]
    fn return_connected_nodes() {
        let graph = build_graph();

        let root = "9".to_string();

        let connected_nodes = explore_from(&root, &graph);
        let small_connection: HashSet<NodeId> = vec!["7".to_string(),
                                                     "8".to_string(),
                                                     root].into_iter()
                                                          .collect();

        assert_eq!(connected_nodes, small_connection);
    }

    #[test]
    fn test_add_node() {
        let graph = build_graph();
        let node_id = "1".to_string();
        let mut new_graph = Graph::new();

        add_node(&graph, &mut new_graph, &node_id);

        assert_eq!(new_graph.get_node(&node_id), graph.get_node(&node_id));
    }

    #[test]
    fn test_add_edges() {
        let graph = build_graph();
        let node_id = "7".to_string();
        let mut new_graph = Graph::new();

        add_node(&graph, &mut new_graph, &node_id);
        add_node(&graph, &mut new_graph, &"9".to_string());
        add_edges(&graph, &mut new_graph, &node_id);
        add_edges(&graph, &mut new_graph, &"9".to_string());

        let nodes_from_seven: Vec<&NodeId> = new_graph.get_edges(&node_id)
                                                      .map(|edges|
                                                          edges.iter()
                                                              .map(|edge| &edge.to_id)
                                                              .collect())
                                                      .unwrap();
        let nodes_from_nine: Vec<&NodeId> = new_graph.get_edges(&"9".to_string())
                                                     .map(|edges|
                                                          edges.iter()
                                                               .map(|edge| &edge.to_id)
                                                               .collect())
                                                     .unwrap();

        assert_eq!(nodes_from_seven, vec![&"9".to_string()]);
        assert_eq!(nodes_from_nine, vec![&"7".to_string()]);
    }

    #[test]
    fn find_connected_component() {
        let graph = build_graph();

        assert!(graph.get_node("7").is_some());
        assert!(graph.get_node("8").is_some());
        assert!(graph.get_node("9").is_some());

        let connected_graph = reduce_to_largest_connected_component(graph);

        assert!(connected_graph.get_node("7").is_none());
        assert!(connected_graph.get_node("8").is_none());
        assert!(connected_graph.get_node("9").is_none());
        assert!(connected_graph.get_node("1").is_some());
        assert!(connected_graph.get_node("2").is_some());
        assert!(connected_graph.get_node("3").is_some());
        assert!(connected_graph.get_node("4").is_some());
        assert!(connected_graph.get_node("5").is_some());
        assert!(connected_graph.get_node("6").is_some());
    }
}