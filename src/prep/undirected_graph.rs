use std::collections::{ HashMap, HashSet, VecDeque };

pub struct Graph(HashMap<String, HashSet<String>>);

impl<'a> Graph {
    pub fn new() -> Self {
        Graph(HashMap::new())
    }

    pub fn add_edge(&mut self, a: String, b: String) {
        {
            let a_adjacent = self.0.entry(a.clone()).or_insert(HashSet::new());
            a_adjacent.insert(b.clone());
        }
        {
            let b_adjacent = self.0.entry(b).or_insert(HashSet::new());
            b_adjacent.insert(a);
        }
    }

    pub fn adjacent_nodes(&'a self, node: &String) -> Option<&'a HashSet<String>> {
        self.0.get(node)
    }
}

pub fn breadth_first_search<F>(graph: Graph, root: &String, f: &mut F)
        where F: for<'a> FnMut(&String) {
    let mut queue: VecDeque<&String> = VecDeque::new();
    let mut marked = HashSet::new();
    queue.push_back(root);
    while let Some(current_node) = queue.pop_front() {
        marked.insert(current_node);
        f(current_node);

        let adjacent_nodes = graph.adjacent_nodes(current_node);
        adjacent_nodes.map(|nodes|
            queue = nodes.iter().fold(queue.clone(), |mut acc, adj|
                if !marked.contains(adj) {
                    acc.push_back(adj);
                    marked.insert(adj);
                    acc
                } else {
                    acc
                }
            )
        );
    }
}

#[cfg(test)]
mod test {
    use std::collections::{ HashMap, HashSet, VecDeque };
    use super::{ Graph, breadth_first_search };

    #[test]
    fn build_graph() {
        let mut graph = Graph::new();

        // A -- B
        // |    |
        // C -- D

        graph.add_edge("A".to_string(), "B".to_string());
        graph.add_edge("A".to_string(), "C".to_string());
        graph.add_edge("B".to_string(), "D".to_string());
        graph.add_edge("C".to_string(), "D".to_string());

        let mut a_adj = HashSet::new();
        a_adj.insert("B".to_string());
        a_adj.insert("C".to_string());
        assert_eq!(graph.adjacent_nodes(&"A".to_string()), Some(&a_adj));

        let mut b_adj = HashSet::new();
        b_adj.insert("A".to_string());
        b_adj.insert("D".to_string());
        assert_eq!(graph.adjacent_nodes(&"B".to_string()), Some(&b_adj));

        let mut c_adj = HashSet::new();
        c_adj.insert("A".to_string());
        c_adj.insert("D".to_string());
        assert_eq!(graph.adjacent_nodes(&"C".to_string()), Some(&c_adj));

        let mut d_adj = HashSet::new();
        d_adj.insert("B".to_string());
        d_adj.insert("C".to_string());
        assert_eq!(graph.adjacent_nodes(&"D".to_string()), Some(&d_adj));
    }

    #[test]
    fn bfs() {
        let mut graph = Graph::new();

        // A -- B
        // |    |
        // C -- D

        graph.add_edge("A".to_string(), "B".to_string());
        graph.add_edge("A".to_string(), "C".to_string());
        graph.add_edge("B".to_string(), "D".to_string());
        graph.add_edge("C".to_string(), "D".to_string());

        let mut visited = VecDeque::new();

        {
            breadth_first_search(
                graph,
                &"A".to_string(),
                &mut (|node: &String| visited.push_back(node.clone()))
            );
        }

        let inner_node = |node|
            node == Some("B".to_string()) || node == Some("C".to_string());

        assert_eq!(visited.pop_front(), Some("A".to_string()));
        assert_eq!(visited.pop_back(), Some("D".to_string()));
        assert_eq!(visited.len(), 2);
        assert!(inner_node(visited.pop_front()));
        assert!(inner_node(visited.pop_front()));
    }
}
