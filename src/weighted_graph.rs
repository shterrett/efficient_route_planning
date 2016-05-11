use std::collections::HashMap;

#[derive(Debug)]
pub struct Graph {
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<NodeId, Vec<Edge>>
}

pub type NodeId = String;

#[derive(PartialEq, Debug, Default)]
pub struct Node {
    pub id: NodeId,
    pub x: f64,
    pub y: f64
}

#[derive(PartialEq, Debug)]
pub struct Edge {
    pub id: String,
    pub from_id: NodeId,
    pub to_id: NodeId,
    pub weight: i64
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            edges: HashMap::new(),
            nodes: HashMap::new()
        }
    }

    pub fn add_node(&mut self, id: String, x: f64, y: f64) {
        let node = Node { id: id.clone(), x: x, y: y, ..Default::default() };
        self.nodes.insert(id, node);
    }

    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn all_nodes(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }

    pub fn add_edge(&mut self, id: String, from_id: NodeId, to_id: NodeId, weight: i64) {
        let edge = self.build_edge(&id, &from_id, &to_id, weight);
        match edge {
            Some(e) => {
                let mut edges = self.edges.entry(from_id).or_insert(Vec::new());
                edges.push(e);
            }
            None => {}
        }
    }

    fn build_edge(&self, id: &str, from_id: &NodeId, to_id: &NodeId, weight: i64) -> Option<Edge> {
        let from = self.get_node(&from_id);
        let to = self.get_node(&to_id);
            if from.is_some() && to.is_some() {
                Some(Edge { id: id.to_string(),
                            from_id: from_id.to_string(),
                            to_id: to_id.to_string(),
                            weight: weight
                          })
            } else {
                None
            }
    }

    pub fn get_edges(&self, node_id: &str) -> Option<&Vec<Edge>> {
        self.edges.get(node_id)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use super::{ Graph, Edge };
    use test_helpers::floats_nearly_eq;

    #[test]
    fn build_graph() {
        let mut graph = Graph::new();

        graph.add_node("1".to_string(), 1.0, 1.0);
        graph.add_node("2".to_string(), 3.0, 5.0);

        let node_1 = graph.get_node("1");
        assert!(node_1.is_some());
        match node_1 {
            Some(node) => {
                assert_eq!(node.id, "1".to_string());
                assert!(floats_nearly_eq(node.x, 1.0));
                assert!(floats_nearly_eq(node.y, 1.0));
            }
            None => {}
        }

        let node_2 = graph.get_node("2");
        assert!(node_2.is_some());
        match node_2 {
            Some(node) => {
                assert_eq!(node.id, "2".to_string());
                assert!(floats_nearly_eq(node.x, 3.0));
                assert!(floats_nearly_eq(node.y, 5.0));
            }
            None => {}
        }

        let still_present = graph.get_node("1");
        assert!(still_present.is_some());
    }

    #[test]
    fn adding_edges() {
        let mut graph = Graph::new();

        graph.add_node("n1".to_string(), 0.0, 12.0);
        graph.add_node("n2".to_string(), 5.0, 0.0);
        graph.add_node("n3".to_string(), 2.0, 4.0);

        graph.add_edge("e1".to_string(), "n2".to_string(), "n1".to_string(), 13);
        graph.add_edge("e2".to_string(), "n3".to_string(), "n2".to_string(), 5);
        graph.add_edge("e3".to_string(), "n2".to_string(), "n3".to_string(), 5);

        let edges_n1 = graph.get_edges("n1");
        let edges_n2 = graph.get_edges("n2");
        let edges_n3 = graph.get_edges("n3");

        assert_eq!(edges_n1, None);
        assert_eq!(edges_n2, Some(&vec![Edge { id: "e1".to_string(),
                                               from_id: "n2".to_string(),
                                               to_id: "n1".to_string(),
                                               weight: 13
                                             },
                                        Edge { id: "e3".to_string(),
                                               from_id: "n2".to_string(),
                                               to_id: "n3".to_string(),
                                               weight: 5
                                             }]));
        assert_eq!(edges_n3, Some(&vec![Edge { id: "e2".to_string(),
                                               from_id: "n3".to_string(),
                                               to_id: "n2".to_string(),
                                               weight: 5
                                             }]));
    }

    #[test]
    fn returns_all_nodes() {
        let mut graph = Graph::new();

        graph.add_node("n1".to_string(), 0.0, 12.0);
        graph.add_node("n2".to_string(), 5.0, 0.0);
        graph.add_node("n3".to_string(), 2.0, 4.0);

        let n1 = "n1".to_string();
        let n2 = "n2".to_string();
        let n3 = "n3".to_string();

        let mut expected_node_ids = HashSet::new();
        expected_node_ids.insert(&n1);
        expected_node_ids.insert(&n2);
        expected_node_ids.insert(&n3);

        let nodes = graph.all_nodes()
                         .iter()
                         .map(|n| &n.id)
                         .collect::<HashSet<&String>>();

        assert_eq!(nodes, expected_node_ids);
    }
}
