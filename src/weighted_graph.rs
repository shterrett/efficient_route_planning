use std::fmt::Debug;
use std::collections::HashMap;
use std::hash::Hash;
use std::borrow::Borrow;

pub trait GraphKey : Clone + Hash + Eq + Debug {}
impl GraphKey for String {}
impl GraphKey for &'static str {}

#[derive(Debug)]
pub struct Graph<T: GraphKey> {
    nodes: HashMap<T, Node<T>>,
    edges: HashMap<T, Vec<Edge<T>>>
}

#[derive(PartialEq, Debug)]
pub struct Node<T: GraphKey> {
    pub id: T,
    pub x: f64,
    pub y: f64,
    pub contraction_order: Option<i64>
}

#[derive(PartialEq, Debug)]
pub struct Edge<T: GraphKey> {
    pub id: T,
    pub from_id: T,
    pub to_id: T,
    pub weight: i64,
    pub arc_flag: bool
}

impl<T: GraphKey> Graph<T> {
    pub fn new() -> Self {
        Graph {
            edges: HashMap::new(),
            nodes: HashMap::new()
        }
    }

    pub fn add_node(&mut self, id: T, x: f64, y: f64) {
        let node = Node { id: id.clone(),
                          x: x,
                          y: y,
                          contraction_order: None
                        };
        self.nodes.insert(id, node);
    }

    pub fn get_node<S>(&self, id: &S) -> Option<&Node<T>>
           where T: Borrow<S>,
                 S: Hash + Eq {
        self.nodes.get(id)
    }

    pub fn get_mut_node<S>(&mut self, id: &S) -> Option<&mut Node<T>>
           where T: Borrow<S>,
                 S: Hash + Eq {
        self.nodes.get_mut(id)
    }

    pub fn all_nodes(&self) -> Vec<&Node<T>> {
        self.nodes.values().collect()
    }

    pub fn add_edge(&mut self, id: T, from_id: T, to_id: T, weight: i64)
           where T: GraphKey {
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
       where T: GraphKey {
        let from = self.get_node(&from_id);
        let to = self.get_node(&to_id);
            if from.is_some() && to.is_some() {
                Some(Edge { id: id.clone(),
                            from_id: from_id.clone(),
                            to_id: to_id.clone(),
                            weight: weight,
                            arc_flag: false
                          })
            } else {
                None
            }
    }

    pub fn get_edges<'a, S>(&'a self, node_id: &S) -> &[Edge<T>]
           where T: Borrow<S>,
                 S: Hash + Eq {
        self.edges.get(node_id).map(Vec::borrow).unwrap_or(&[])
    }

    pub fn get_mut_edge(&mut self, from_node_id: &T, to_node_id: &T) -> Option<&mut Edge<T>>
       where T: GraphKey {
        self.edges.get_mut(from_node_id).and_then(|edges|
            edges.iter_mut().find(|edge| edge.to_id == *to_node_id)
        )
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

        let node_1 = graph.get_node(&"1");
        assert!(node_1.is_some());
        match node_1 {
            Some(node) => {
                assert_eq!(node.id, "1");
                assert!(floats_nearly_eq(node.x, 1.0));
                assert!(floats_nearly_eq(node.y, 1.0));
            }
            None => {}
        }

        let node_2 = graph.get_node(&"2");
        assert!(node_2.is_some());
        match node_2 {
            Some(node) => {
                assert_eq!(node.id, "2");
                assert!(floats_nearly_eq(node.x, 3.0));
                assert!(floats_nearly_eq(node.y, 5.0));
            }
            None => {}
        }

        let still_present = graph.get_node(&"1");
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

        let edges_n1 = graph.get_edges(&"n1");
        let edges_n2 = graph.get_edges(&"n2");
        let edges_n3 = graph.get_edges(&"n3");

        assert_eq!(edges_n1, &[]);
        assert_eq!(edges_n2, &[Edge { id: "e1",
                                      from_id: "n2",
                                      to_id: "n1",
                                      weight: 13,
                                      arc_flag: false
                                    },
                               Edge { id: "e3",
                                      from_id: "n2",
                                      to_id: "n3",
                                      weight: 5,
                                      arc_flag: false
                                    }]);
        assert_eq!(edges_n3, &[Edge { id: "e2",
                                      from_id: "n3",
                                      to_id: "n2",
                                      weight: 5,
                                      arc_flag: false
                                    }]);
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
                         .map(|n| n.id)
                         .collect::<HashSet<&str>>();

        assert_eq!(nodes, expected_node_ids);
    }

    #[test]
    fn edit_edge() {
        let mut graph = Graph::new();

        graph.add_node("n1", 0.0, 12.0);
        graph.add_node("n2", 5.0, 0.0);
        graph.add_node("n3", 2.0, 4.0);

        graph.add_edge("e1", "n2", "n1", 13);
        graph.add_edge("e2", "n3", "n2", 5);
        graph.add_edge("e3", "n2", "n3", 5);

        if let Some(mut edge) = graph.get_mut_edge(&"n2", &"n3") {
            edge.arc_flag = true;
        }

        for edge in graph.get_edges(&"n2") {
            if edge.to_id == "n3" {
                assert!(edge.arc_flag);
            } else {
                assert!(!edge.arc_flag);
            }
        }
    }

    #[test]
    fn edit_node() {
        let mut graph = Graph::new();
        graph.add_node("n", 0.0, 0.0);

        assert_eq!(graph.get_node(&"n").and_then(|n| n.contraction_order), None);

        graph.get_mut_node(&"n").map(|n| n.contraction_order = Some(1));

        assert_eq!(graph.get_node(&"n").and_then(|n| n.contraction_order), Some(1));
    }
}
