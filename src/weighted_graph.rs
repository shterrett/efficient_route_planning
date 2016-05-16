use std::collections::HashMap;
use std::hash::Hash;
use std::borrow::Borrow;

#[derive(Debug)]
pub struct Graph<T: Clone + Hash + Eq> {
    nodes: HashMap<T, Node<T>>,
    edges: HashMap<T, Vec<Edge<T>>>
}

#[derive(PartialEq, Debug)]
pub struct Node<T: Clone + Hash + Eq> {
    pub id: T,
    pub x: f64,
    pub y: f64
}

#[derive(PartialEq, Debug)]
pub struct Edge<T: Clone + Hash + Eq> {
    pub id: T,
    pub from_id: T,
    pub to_id: T,
    pub weight: i64
}

impl<T: Clone + Hash + Eq> Graph<T> {
    pub fn new() -> Self {
        Graph {
            edges: HashMap::new(),
            nodes: HashMap::new()
        }
    }

    pub fn add_node(&mut self, id: T, x: f64, y: f64) {
        let node = Node { id: id.clone(), x: x, y: y };
        self.nodes.insert(id, node);
    }

    pub fn get_node<S>(&self, id: &S) -> Option<&Node<T>>
           where T: Borrow<S>,
                 S: Hash + Eq {
        self.nodes.get(id)
    }

    pub fn all_nodes(&self) -> Vec<&Node<T>> {
        self.nodes.values().collect()
    }

    pub fn add_edge(&mut self, id: T, from_id: T, to_id: T, weight: i64)
           where T: Clone + Hash + Eq {
        let edge = self.build_edge(&id, &from_id, &to_id, weight);
        match edge {
            Some(e) => {
                let mut edges = self.edges.entry(from_id).or_insert(Vec::new());
                edges.push(e);
            }
            None => {}
        }
    }

    fn build_edge(&self, id: &T, from_id: &T, to_id: &T, weight: i64) -> Option<Edge<T>>
       where T: Clone + Hash + Eq {
        let from = self.get_node(&from_id);
        let to = self.get_node(&to_id);
            if from.is_some() && to.is_some() {
                Some(Edge { id: id.clone(),
                            from_id: from_id.clone(),
                            to_id: to_id.clone(),
                            weight: weight
                          })
            } else {
                None
            }
    }

    pub fn get_edges<S>(&self, node_id: &S) -> Option<&Vec<Edge<T>>>
           where T: Borrow<S>,
                 S: Hash + Eq {
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

        graph.add_node("1", 1.0, 1.0);
        graph.add_node("2", 3.0, 5.0);

        let node_1 = graph.get_node("1");
        assert!(node_1.is_some());
        match node_1 {
            Some(node) => {
                assert_eq!(node.id, "1");
                assert!(floats_nearly_eq(node.x, 1.0));
                assert!(floats_nearly_eq(node.y, 1.0));
            }
            None => {}
        }

        let node_2 = graph.get_node("2");
        assert!(node_2.is_some());
        match node_2 {
            Some(node) => {
                assert_eq!(node.id, "2");
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

        graph.add_node("n1", 0.0, 12.0);
        graph.add_node("n2", 5.0, 0.0);
        graph.add_node("n3", 2.0, 4.0);

        graph.add_edge("e1", "n2", "n1", 13);
        graph.add_edge("e2", "n3", "n2", 5);
        graph.add_edge("e3", "n2", "n3", 5);

        let edges_n1 = graph.get_edges("n1");
        let edges_n2 = graph.get_edges("n2");
        let edges_n3 = graph.get_edges("n3");

        assert_eq!(edges_n1, None);
        assert_eq!(edges_n2, Some(&vec![Edge { id: "e1",
                                               from_id: "n2",
                                               to_id: "n1",
                                               weight: 13
                                             },
                                        Edge { id: "e3",
                                               from_id: "n2",
                                               to_id: "n3",
                                               weight: 5
                                             }]));
        assert_eq!(edges_n3, Some(&vec![Edge { id: "e2",
                                               from_id: "n3",
                                               to_id: "n2",
                                               weight: 5
                                             }]));
    }

    #[test]
    fn returns_all_nodes() {
        let mut graph = Graph::new();

        graph.add_node("n1", 0.0, 12.0);
        graph.add_node("n2", 5.0, 0.0);
        graph.add_node("n3", 2.0, 4.0);

        let mut expected_node_ids = HashSet::new();
        expected_node_ids.insert("n1");
        expected_node_ids.insert("n2");
        expected_node_ids.insert("n3");

        let nodes = graph.all_nodes()
                         .iter()
                         .map(|n| &n.id)
                         .collect::<HashSet<&String>>();

        assert_eq!(nodes, expected_node_ids);
    }
}
