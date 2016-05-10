use std::collections::BinaryHeap;

use weighted_graph::Graph;

#[cfg(test)]
mod test {
    use weighted_graph::Graph;

    fn build_graph() ->  Graph {
        let mut graph = Graph::new();
        graph.add_node("1".to_string(), 1.0, 1.0);
        graph.add_node("2".to_string(), 1.0, 2.0);
        graph.add_node("3".to_string(), 2.0, 1.0);
        graph.add_node("4".to_string(), 2.0, 2.0);
        graph.add_node("5".to_string(), 2.0, 3.0);
        graph.add_node("6".to_string(), 3.0, 1.0);

        let edges = vec![("a".to_string(), "1".to_string(), "4".to_string(), 1.0),
                         ("b".to_string(), "4".to_string(), "2".to_string(), 4.0),
                         ("c".to_string(), "2".to_string(), "5".to_string(), 3.0),
                         ("d".to_string(), "5".to_string(), "6".to_string(), 3.0),
                         ("e".to_string(), "6".to_string(), "3".to_string(), 1.0),
                         ("f".to_string(), "6".to_string(), "4".to_string(), 2.0)];

        let mut iter = edges.into_iter();

        while let Some((edge_id, node_id_1, node_id_2, cost)) = iter.next() {
            graph.add_edge(edge_id.clone(), node_id_1.clone(), node_id_2.clone(), cost);
            graph.add_edge(edge_id.clone(), node_id_2.clone(), node_id_1.clone(), cost);
        }

        graph
    }

    #[test]
    fn graph() {
        let graph = build_graph();
        println!("{:?}", graph);
        assert!(graph.get_node("3").is_some());
        assert!(graph.get_edges("3").is_some());
    }
}
