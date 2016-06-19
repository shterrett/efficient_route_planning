use std::collections::HashMap;
use weighted_graph::{ GraphKey, Graph, Node };
use dijkstra::shortest_path as dijkstra;
use pathfinder::{ CurrentBest, Pathfinder, EdgeIterator };

pub fn shortest_path<'a, T>(graph: &'a Graph<T>,
                        source: &T,
                        destination: Option<&T>
                       ) -> (i64, HashMap<T, CurrentBest<T>>)
    where T: GraphKey {
    let identity = |_: Option<&Node<T>>, _ :Option<&Node<T>>| 0;
    let edge_iterator = |g: &'a Graph<T>, node_id: &T| ->
                        EdgeIterator<'a, T> {
        Box::new(g.get_edges(node_id).iter().filter(|edge| edge.arc_flag))
    };
    let terminator = |_: &CurrentBest<T>, _: &HashMap<T, CurrentBest<T>>| false;
    let pathfinder = Pathfinder::new(Box::new(identity),
                                     Box::new(edge_iterator),
                                     Box::new(terminator)
                                    );
    pathfinder.shortest_path(graph, source, destination)
}

pub struct Rect {
    x_max: f64,
    x_min: f64,
    y_max: f64,
    y_min: f64
}

impl Rect {
    pub fn contains<T>(&self, node: &Node<T>) -> bool
           where T: GraphKey {
        node.x <= self.x_max &&
            node.x >= self.x_min &&
            node.y <= self.y_max &&
            node.y >= self.y_min
    }
}

pub fn assign_arc_flags<T>(graph: &mut Graph<T>, region: Rect)
       where T: GraphKey {
    let internal = &internal_nodes(graph, &region)[..];
    let results = inbound_paths(graph, internal, &region);
    for result in results {
        graph.get_mut_edge(&result.id, &result.predecessor)
             .map(|edge| edge.arc_flag = true);
    }

    for from_id in internal {
        for to_id in internal {
            graph.get_mut_edge(&from_id, &to_id).map(|edge| edge.arc_flag = true);
        }
    }
}

fn inbound_paths<T>(graph: &Graph<T>, node_ids: &[T], region: &Rect) -> Vec<CurrentBest<T>>
   where T: GraphKey {
    node_ids.iter()
            .filter(|node_id| boundary_node(graph, region, *node_id))
            .flat_map(|node_id|
                dijkstra(graph, &node_id, None).1.into_iter()
                    .map(|(_, v)| v)
                ).collect()
}

fn internal_nodes<T>(graph: &Graph<T>, rect: &Rect) -> Vec<T>
   where T: GraphKey {
    graph.all_nodes()
         .into_iter()
         .filter(|node_ref| rect.contains(node_ref))
         .map(|node_ref| node_ref.id.clone())
         .collect::<Vec<T>>()
}

fn boundary_node<T>(graph: &Graph<T>, rect: &Rect, node_id: &T) -> bool
   where T: GraphKey {
    match graph.get_node(node_id) {
        Some(node) => {
            rect.contains(node) &&
            graph.get_edges(&node.id).iter().any(|edge|
                graph.get_node(&edge.to_id).map(|node|
                    !rect.contains(node)
                ).unwrap_or(false)
            )
        }
        None => false
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use weighted_graph::{ Graph, Node };
    use super::{ Rect,
                 boundary_node,
                 assign_arc_flags,
                 shortest_path
               };

    fn build_graph() -> Graph<&'static str> {
        let mut graph = Graph::new();
        graph.add_node("1", 1.0, 2.0);
        graph.add_node("2", 2.0, 1.0);
        graph.add_node("3", 2.0, 2.0);
        graph.add_node("4", 3.0, 3.0);
        graph.add_node("5", 3.0, 4.0);
        graph.add_node("6", 4.0, 3.0);

        let edges = vec![("af", "1", "5", 1),
                         ("ar", "5", "1", 1),
                         ("bf", "5", "6", 1),
                         ("br", "6", "5", 1),
                         ("cf", "2", "6", 4),
                         ("cr", "6", "2", 4),
                         ("df", "2", "4", 1),
                         ("dr", "4", "2", 1),
                         ("ef", "3", "4", 1),
                         ("er", "4", "3", 1)];

        for (edge_id, node_id_1, node_id_2, cost) in edges {
            graph.add_edge(edge_id.clone(), node_id_1.clone(), node_id_2.clone(), cost);
        }

        graph
    }

    #[test]
    fn node_contains_rectangle() {
        let rect = Rect { x_min: 0.0, x_max: 5.0, y_min: 0.0, y_max: 5.0 };
        let contains = Node { id: "contains", x: 1.0, y: 1.0 };
        let outside = Node { id: "outside", x: 10.0, y: 10.0 };
        let border = Node { id: "border", x: 0.0, y: 3.0 };

        assert!(rect.contains(&contains));
        assert!(!rect.contains(&outside));
        assert!(rect.contains(&border));
    }

    #[test]
    fn identify_boundary_node() {
        let graph = build_graph();
        let rect = Rect { x_min: 1.5,
                          x_max: 3.5,
                          y_min: 1.5,
                          y_max: 3.5
                        };

        assert!(boundary_node(&graph, &rect, &"4"));
        assert!(!boundary_node(&graph, &rect, &"3"));
        assert!(!boundary_node(&graph, &rect, &"1"));
    }

    #[test]
    fn arc_flag_assignments() {
        let mut graph = build_graph();
        let region = Rect { x_min: 1.5,
                            x_max: 3.5,
                            y_min: 1.5,
                            y_max: 3.5
                          };

        assign_arc_flags(&mut graph, region);

        let flagged_arcs: HashSet<&str> = vec!["af",
                                               "bf",
                                               "cr",
                                               "df",
                                               "ef",
                                               "er"].into_iter().collect();

        for node in graph.all_nodes() {
            for edge in graph.get_edges(&node.id) {
                if flagged_arcs.contains(&edge.id) {
                    assert!(edge.arc_flag);
                } else {
                    assert!(!edge.arc_flag);
                }
            }
        }
    }

    #[test]
    fn shortest_path_uses_arc_flags() {
        let mut graph = build_graph();

        let region = Rect { x_min: 1.5,
                            x_max: 3.5,
                            y_min: 1.5,
                            y_max: 3.5
                          };

        assign_arc_flags(&mut graph, region);

        let (cost, results) = shortest_path(&graph, &"6", Some(&"4"));

        assert!(!results.values().any(|r| r.id == "5"));
        assert_eq!(cost, 5)
    }
}
