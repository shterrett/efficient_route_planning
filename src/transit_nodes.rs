use std::collections::{ HashSet, HashMap };

use weighted_graph::{ Graph, GraphKey };
use pathfinder::CurrentBest;
use contraction::preprocess_contraction;
use arc_flags::shortest_path as arc_flags_shortest_path;
use dijkstra::shortest_path as dijkstra_shortest_path;
use contraction::shortest_path as contraction_shortest_path;

pub fn shortest_path<T>(source_distances: &HashMap<T, i64>,
                        destination_distances: &HashMap<T, i64>,
                        inter_transit_node_distances: &HashMap<(T, T), i64>
                       ) -> Option<(i64, (T, T))>
    where T: GraphKey {
        inter_transit_node_distances.iter()
            .filter_map(|(&(ref fm, ref to), &inter_cost)|
                            path_cost_through_transits(fm,
                                                    to,
                                                    inter_cost,
                                                    source_distances,
                                                    destination_distances).map(|cost|
                                                        (cost, (fm, to)))
                       )
            .min_by_key(|&(cost, _)| cost)
            .map(|(cost, (source_transit, dest_transit))|
                 (cost, (source_transit.clone(), dest_transit.clone()))
                )
}

fn path_cost_through_transits<T>(from: &T,
                                 to: &T,
                                 inter_cost: i64,
                                 source_distances: &HashMap<T, i64>,
                                 destination_distances: &HashMap<T, i64>) -> Option<i64>
   where T: GraphKey {
    source_distances.get(from)
                    .and_then(|sd| destination_distances.get(to).map(|dd| dd + sd))
                    .map(|dist| dist + inter_cost)
}

pub fn transit_nodes_contraction<T>(graph: &mut Graph<T>) -> HashSet<T>
       where T: GraphKey {
    let number_transit_nodes = (graph.all_nodes().len() as f64).sqrt().floor() as usize;
    preprocess_contraction(graph);

    let mut nodes = graph.all_nodes()
                     .iter()
                     .map(|node| (node.contraction_order.unwrap().clone(), node.id.clone()))
                     .collect::<Vec<(i64, T)>>();

    nodes.as_mut_slice()
         .sort_by(|a, b| b.0.cmp(&a.0));

    nodes.iter()
         .map(|node| node.1.clone())
         .take(number_transit_nodes)
         .collect()
}

pub fn neighboring_transit_nodes<T>(graph: &Graph<T>,
                                    transit_nodes: &HashSet<T>,
                                    origin: &T)
                                   -> HashMap<T, i64>
   where T: GraphKey {
    let (_, results) = arc_flags_shortest_path(graph, origin, None);

    results.iter()
           .filter_map(|(node_id, _)|
                first_transit_node(node_id, &results, transit_nodes))
           .filter_map(|transit_node|
                contraction_shortest_path(graph,
                                          origin,
                                          &transit_node
                                         ).map(|(cost, _)| (transit_node, cost)))
           .collect()
}

fn first_transit_node<T>(node_id: &T,
                         results: &HashMap<T, CurrentBest<T>>,
                         transit_nodes: &HashSet<T>)
                        -> Option<T>
   where T: GraphKey {
    if transit_nodes.contains(node_id) {
        first_transit_node_helper(node_id.clone(),
                                  Some(node_id.clone()),
                                  results,
                                  transit_nodes).map(|id| id.clone())
    } else {
        first_transit_node_helper(node_id.clone(),
                                  None,
                                  results,
                                  transit_nodes).map(|id| id.clone())
    }
}

fn first_transit_node_helper<T>(node_id: T,
                                current_transit_node: Option<T>,
                                results: &HashMap<T, CurrentBest<T>>,
                                transit_nodes: &HashSet<T>)
                               -> Option<T>
   where T: GraphKey {
    match results.get(&node_id).and_then(|result| result.predecessor.clone()) {
        Some(predecessor_id) => {
            if transit_nodes.contains(&predecessor_id) {
                 first_transit_node_helper(predecessor_id.clone(),
                                           Some(predecessor_id),
                                           results,
                                           transit_nodes)
                } else {
                 first_transit_node_helper(predecessor_id,
                                           current_transit_node,
                                           results,
                                           transit_nodes)
                }
        }
        None => current_transit_node
    }
}

pub fn pairwise_transit_node_distances<T>(graph: &Graph<T>,
                                          transit_nodes: &HashSet<T>
                                         ) -> HashMap<(T, T), i64>
   where T: GraphKey {
    let mut pairs = vec![];
    for from in transit_nodes {
        for to in transit_nodes {
            pairs.push((from.clone(), to.clone()));
        }
    }

    pairs.iter().map(|&(ref from, ref to)|
            match dijkstra_shortest_path(graph, from, Some(to)) {
                (cost, _) => ((from.clone(), to.clone()), cost)
            }
         ).collect()
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use weighted_graph::{ Graph };
    use dijkstra::shortest_path as dijkstra;
    use super::{ transit_nodes_contraction,
                 neighboring_transit_nodes,
                 pairwise_transit_node_distances,
                 shortest_path
               };

    fn build_full_graph() -> (Vec<(&'static str, f64, f64)>, // nodes
                              Vec<(&'static str, &'static str, i64)>, // edges
                              Graph<&'static str>) {
        let mut graph = Graph::new();
        let nodes = vec![("a", 0.0, 3.0),
                         ("b", 0.0, 1.0),
                         ("c", 0.0, 0.0),
                         ("d", 1.0, 3.0),
                         ("e", 1.0, 2.0),
                         ("f", 1.0, 0.0),
                         ("g", 2.0, 3.0),
                         ("h", 2.0, 1.0),
                         ("i", 2.0, 0.0)];
        for &(id, x, y) in &nodes {
            graph.add_node(id, x, y);
        }

        let edges = vec![("a", "b", 3),
                         ("a", "d", 2),
                         ("b", "c", 1),
                         ("b", "e", 1),
                         ("c", "f", 2),
                         ("d", "e", 1),
                         ("d", "g", 2),
                         ("e", "f", 3),
                         ("e", "h", 1),
                         ("f", "i", 2),
                         ("g", "h", 4),
                         ("h", "i", 2),
                        ];
        for &(n1, n2, w) in &edges {
            graph.add_edge(n1, n1, n2, w);
            graph.add_edge(n2, n2, n1, w);
            graph.get_mut_edge(&n1, &n2).map(|edge| edge.arc_flag = true);
            graph.get_mut_edge(&n2, &n1).map(|edge| edge.arc_flag = true);
        }

        (nodes, edges, graph)
    }

    #[test]
    fn compute_set_of_transit_nodes() {
        let (_, _, mut graph) = build_full_graph();

        let transit_nodes = transit_nodes_contraction(&mut graph);

        let mut expected = HashSet::new();
        expected.insert(7);
        expected.insert(8);
        expected.insert(9);

        assert_eq!(transit_nodes.len(), 3);
        assert_eq!(transit_nodes.iter()
                                .filter_map(|tn|
                                     graph.get_node(tn)
                                          .map(|node| node.contraction_order.unwrap()))
                                .collect::<HashSet<i64>>(),
                   expected);
    }

    #[test]
    fn transit_node_distances_from_node() {
        let (_, _, mut graph) = build_full_graph();
        let transit_nodes = transit_nodes_contraction(&mut graph);

        let first_contracted = graph.all_nodes()
                                    .iter()
                                    .min_by_key(|node| node.contraction_order)
                                    .map(|node| node.id)
                                    .unwrap();

        let transit_node_distances = neighboring_transit_nodes(&graph,
                                                               &transit_nodes,
                                                               &first_contracted);

        assert!(transit_node_distances.len() > 0);
        assert!(transit_node_distances.keys().all(|node_id| transit_nodes.contains(node_id)));
        println!("GRAPH {:?}", graph);
        for (node_id, _) in &transit_node_distances {
            let (cost, _) = dijkstra(&graph, &first_contracted, Some(&node_id));
            println!("FROM {:?} TO {:?}", first_contracted, node_id);
            assert_eq!(*transit_node_distances.get(node_id).unwrap(), cost);
        }
    }

    #[test]
    fn inter_transit_node_distances() {
        let (_, _, mut graph) = build_full_graph();
        let transit_nodes = transit_nodes_contraction(&mut graph);

        let transit_node_distances = pairwise_transit_node_distances(&graph,
                                                                     &transit_nodes);

        for &from in &transit_nodes {
            for &to in &transit_nodes {
                let (cost, _) = dijkstra(&graph, &from, Some(&to));
                assert_eq!(cost, *transit_node_distances.get(&(from, to)).unwrap());
            }
        }
    }

    #[test]
    fn find_shortest_path() {
        let (_, _, mut graph) = build_full_graph();
        let source = "c";
        let destination = "g";

        let transit_nodes = transit_nodes_contraction(&mut graph);
        let source_distances = neighboring_transit_nodes(&graph,
                                                         &transit_nodes,
                                                         &source);
        let destination_distances = neighboring_transit_nodes(&graph,
                                                              &transit_nodes,
                                                              &destination);
        let inter_transit_node_distances = pairwise_transit_node_distances(&graph,
                                                                           &transit_nodes);
        match shortest_path(&source_distances,
                            &destination_distances,
                            &inter_transit_node_distances) {
            Some((cost, (fm_transit, to_transit))) => {
                assert_eq!(cost, 5);
                assert!(transit_nodes.contains(&fm_transit));
                assert!(transit_nodes.contains(&to_transit));
            }
            None => assert!(false)
        }
    }
}
