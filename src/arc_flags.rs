use std::hash::Hash;
use weighted_graph::{ Graph, Node };
use dijkstra::shortest_path;
use pathfinder::CurrentBest;

pub struct Rect {
    x_max: f64,
    x_min: f64,
    y_max: f64,
    y_min: f64
}

impl Rect {
    pub fn contains<T>(&self, node: &Node<T>) -> bool
           where T: Clone + Eq + Hash {
        node.x <= self.x_max &&
            node.x >= self.x_min &&
            node.y <= self.y_max &&
            node.y >= self.y_min
    }
}

pub fn assign_arc_flags<T>(graph: &mut Graph<T>, region: Rect)
       where T: Clone + Eq + Hash {
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
   where T: Clone + Eq + Hash {
    node_ids.iter()
            .filter(|node_id| boundary_node(graph, region, *node_id))
            .flat_map(|node_id|
                shortest_path(graph, &node_id, None).1.into_iter()
                    .map(|(_, v)| v)
                ).collect()
}

fn internal_nodes<T>(graph: &Graph<T>, rect: &Rect) -> Vec<T>
   where T: Clone + Eq + Hash {
    graph.all_nodes()
         .into_iter()
         .filter(|node_ref| rect.contains(node_ref))
         .map(|node_ref| node_ref.id.clone())
         .collect::<Vec<T>>()
}

fn boundary_node<T>(graph: &Graph<T>, rect: &Rect, node_id: &T) -> bool
   where T: Clone + Eq + Hash {
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
    use dijkstra::shortest_path;
    use super::{ Rect, boundary_node, assign_arc_flags };

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
                         ("cf", "2", "6", 1),
                         ("cr", "6", "2", 1),
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

        assign_arc_flags(&mut graph, Rect { x_min: 1.5,
                                            x_max: 3.5,
                                            y_min: 1.5,
                                            y_max: 3.5
                                          });

        let flagged_arcs: HashSet<&str> = vec!["af",
                                               "bf",
                                               "cr",
                                               "df",
                                               "ef",
                                               "er"].into_iter().collect();

        let results = shortest_path(&graph, &"4", None);
        let internal = graph.all_nodes()
                            .into_iter()
                            .filter(|node_ref| region.contains(node_ref))
                            .map(|node_ref| node_ref.id.clone())
                            .collect::<Vec<&str>>();
        println!("{:?}", results);
        println!("\n\n\n");
        println!("{:?}", internal);
        println!("\n\n\n");
        println!("{:?}", graph);
        println!("\n\n\n");
        for node in graph.all_nodes() {
            for edge in graph.get_edges(&node.id) {
                println!("{:?}", edge);
                if flagged_arcs.contains(&edge.id) {
                    assert!(edge.arc_flag);
                } else {
                    assert!(!edge.arc_flag);
                }
            }
        }
    }
}
