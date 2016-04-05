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

struct Queue<T>(VecDeque<T>);
struct Stack<T>(VecDeque<T>);

pub trait Seq<T> {
    fn push(&mut self, val: T);
    fn pop(&mut self) -> Option<T>;
    fn new() -> Self;
}

impl<T> Seq<T> for  Stack<T> {
    fn new() -> Self {
        Stack(VecDeque::new())
    }

    fn push(&mut self, val: T) {
        self.0.push_back(val)
    }

    fn pop(&mut self) -> Option<T> {
        self.0.pop_back()
    }
}

impl<T> Seq<T> for Queue<T> {
    fn new() -> Self {
        Queue(VecDeque::new())
    }

    fn push(&mut self, val: T) {
        self.0.push_back(val)
    }

    fn pop(&mut self) -> Option<T> {
        self.0.pop_front()
    }
}

fn search<'a, F, S: Seq<&'a str>>(graph: &'a Graph, root: &'a str, f: &mut F, seq: S)
    where F: FnMut(&str) {

    let root_ref = &root.to_string();
    let mut seen = seq;
    let mut marked = HashSet::new();
    seen.push(root);
    marked.insert(root_ref);
    while let Some(current_node) = seen.pop() {
        f(current_node);

        match graph.adjacent_nodes(&current_node.to_string()) {
            Some(adjacent_nodes) => {
                let mut iter = adjacent_nodes.iter();
                while let Some(node) = iter.next() {
                    if !marked.contains(&node) {
                        marked.insert(node);
                        seen.push(node);
                    }
                }
            }
            None => {}
        }
    }
}

pub fn breadth_first_search<F>(graph: &Graph, root: &str, f: &mut F)
        where F: FnMut(&str) {
            search(graph, root, f, <Queue<&str> as Seq<&str>>::new());
}

pub fn depth_first_search<F>(graph: &Graph, root: &str, f: &mut F)
        where F: FnMut(&str) {
            search(graph, root, f, <Stack<&str> as Seq<&str>>::new());
}

#[cfg(test)]
mod test {
    use std::collections::{ HashSet, VecDeque };
    use super::{ Graph,
                 breadth_first_search,
                 depth_first_search,
                 Stack,
                 Queue,
                 Seq
               };

    #[test]
    fn stack() {
        let a = "A".to_string();
        let b = "B".to_string();
        let c = "C".to_string();

        let mut stack = Stack::new();
        stack.push(&a);
        stack.push(&b);
        stack.push(&c);

        assert_eq!(stack.pop(), Some(&c));
        assert_eq!(stack.pop(), Some(&b));
        assert_eq!(stack.pop(), Some(&a));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn queue() {
        let a = "A".to_string();
        let b = "B".to_string();
        let c = "C".to_string();

        let mut queue = Queue::new();
        queue.push(&a);
        queue.push(&b);
        queue.push(&c);

        assert_eq!(queue.pop(), Some(&a));
        assert_eq!(queue.pop(), Some(&b));
        assert_eq!(queue.pop(), Some(&c));
        assert_eq!(queue.pop(), None);
    }

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
        let root = "A";

        let mut visited = VecDeque::new();

        {
            breadth_first_search(
                &graph,
                &root,
                &mut (|node: &str| visited.push_back(node.to_string()))
            );
        }

        let inner_node = |node|
            node == Some("B".to_string()) || node == Some("C".to_string());

        assert_eq!(visited.pop_front(), Some("A".to_string()));
        assert_eq!(visited.pop_back(), Some("D".to_string()));
        assert_eq!(visited.len(), 2);
        assert!(inner_node(visited.pop_front()));
        assert!(inner_node(visited.pop_front()));

        let mut a_adj = HashSet::new();
        a_adj.insert("B".to_string());
        a_adj.insert("C".to_string());
        assert_eq!(graph.adjacent_nodes(&"A".to_string()), Some(&a_adj));
    }

    #[test]
    fn dfs() {
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
            depth_first_search(
                &graph,
                &"A".to_string(),
                &mut (|node: &str| visited.push_back(node.to_string()))
            );
        }

        let inner_node = |node|
            node == Some("B".to_string()) || node == Some("C".to_string());

        assert_eq!(visited.pop_front(), Some("A".to_string()));
        assert!(inner_node(visited.pop_front()));
        assert_eq!(visited.pop_front(), Some("D".to_string()));
        assert!(inner_node(visited.pop_front()));

        let mut a_adj = HashSet::new();
        a_adj.insert("B".to_string());
        a_adj.insert("C".to_string());
        assert_eq!(graph.adjacent_nodes(&"A".to_string()), Some(&a_adj));
    }
}
