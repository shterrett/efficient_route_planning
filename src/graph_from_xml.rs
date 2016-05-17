extern crate xml;

use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use self::xml::attribute::OwnedAttribute;
use self::xml::reader::{ EventReader, XmlEvent };

use weighted_graph::Graph;
use road_weights::road_weight;

pub fn build_graph_from_xml(path: &str) -> Graph<String> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let mut parser = EventReader::new(reader);
    let mut graph = Graph::new();
    let mut current_edge_id = "".to_string();
    let mut current_edge_type = "".to_string();
    let mut edge_nodes = vec![];
    let mut eof = false;

    while !eof {
        match parser.next() {
            Ok(e) => match e {
                XmlEvent::StartElement { ref name, ref attributes, .. } => {
                    match name.local_name.as_str() {
                        "node" => {
                            add_node(&mut graph, &attributes);
                        }
                        "way" => {
                            current_edge_id = get_attribute(&attributes, "id").unwrap_or("".to_string());
                        }
                        "nd" => {
                            edge_nodes.push(get_attribute(&attributes, "ref").unwrap_or("".to_string()));
                        }
                        "tag" => {
                            get_attribute(&attributes, "k").map(|key|
                                if key == "highway" {
                                    current_edge_type = get_attribute(&attributes, "v").unwrap();
                                }
                            );
                        }
                        _ => {}
                    }
                }
                XmlEvent::EndElement { ref name, .. } => {
                    match name.local_name.as_str() {
                        "way" => {
                            add_edge(&mut graph, &current_edge_id, &current_edge_type, &edge_nodes);
                            current_edge_id = "".to_string();
                            current_edge_type = "".to_string();
                            edge_nodes.clear();
                        }
                        _ => {}
                    }
                }
                XmlEvent::EndDocument => {
                    eof = true;
                }
                _ => {}
            },
            Err(e) => println!("Error parsing XML document: {}", e)
        }
    }

    graph
}

fn add_node(graph: &mut Graph<String>, attributes: &Vec<OwnedAttribute>) {
    let mut map = HashMap::new();
    let mut atrb = attributes.iter().fold(&mut map, |m, attribute| {
                    m.insert(attribute.name.local_name.clone(),
                            attribute.value.clone());
                    m
                }
    );
    graph.add_node(atrb.remove("id").unwrap(),
                   atrb.remove("lon").unwrap().parse::<f64>().unwrap(),
                   atrb.remove("lat").unwrap().parse::<f64>().unwrap()
    )
}

fn add_edge(graph: &mut Graph<String>, edge_id: &String, edge_type: &str, nodes: &Vec<String>) {
    let mut pairs = nodes.windows(2);
    while let Some(pair) = pairs.next() {
        match road_weight(graph.get_node(&pair[0]).unwrap(),
                          graph.get_node(&pair[1]).unwrap(),
                          edge_type) {
            Some(weight) => {
                graph.add_edge(edge_id.clone(),
                               pair[0].clone(),
                               pair[1].clone(),
                               weight);
                graph.add_edge(edge_id.clone(),
                               pair[1].clone(),
                               pair[0].clone(),
                               weight);
            }
            None => {}
        };
    }
}

fn get_attribute(attributes: &Vec<OwnedAttribute>, attribute_name: &str) -> Option<String> {
    let mut matches = attributes.iter().filter_map(|attribute|
                         if attribute.name.local_name == attribute_name {
                              Some(attribute.value.clone())
                         } else {
                             None
                         }
                     );
    matches.next()
}

#[cfg(test)]
mod test {
    use super::{ build_graph_from_xml };
    use weighted_graph:: { Graph, Node };
    use road_weights::road_weight;

    fn has_node_ids(graph: &Graph<String>) -> bool {
        vec!["292403538", "298884289", "261728686", "298884272"].iter().all(|id|
            graph.get_node(&id.to_string()).is_some()
        )
    }

    fn node_spot_check(graph: &Graph<String>) -> bool {
        match graph.get_node(&"292403538".to_string()) {
            Some(node) => {
                node == &Node { id: "292403538".to_string(),
                                x: 12.2482632,
                                y: 54.0901746
                              }
            }
            None => false
        }
    }

    fn has_edges_for_nodes(graph: &Graph<String>) -> bool {
        vec![("292403538".to_string(), 2),
             ("298884289".to_string(), 2),
             ("261728686".to_string(), 2),
             ("298884272".to_string(), 2)]
            .iter().all(|t|
                graph.get_edges(&t.0).is_some() &&
                  graph.get_edges(&t.0).unwrap().len() == t.1
            )
    }

    fn edge_spot_check(graph: &Graph<String>) -> bool {
        match graph.get_edges(&"298884289".to_string()) {
            Some(edges) => {
                edges.len() == 2 &&
                edges.iter().any(|edge|
                    edge.from_id == "298884289" &&
                    edge.to_id == "292403538" &&
                        (edge.weight ==
                            road_weight(graph.get_node(&"298884289".to_string()).unwrap(),
                                        graph.get_node(&"292403538".to_string()).unwrap(),
                                        "unclassified").unwrap())
                ) &&
                edges.iter().any(|edge|
                    edge.from_id == "298884289" &&
                    edge.to_id == "261728686" &&
                        (edge.weight ==
                            road_weight(graph.get_node(&"298884289".to_string()).unwrap(),
                                        graph.get_node(&"261728686".to_string()).unwrap(),
                                        "unclassified").unwrap())
                )
            }
            None => false
        }
    }

    #[test]
    fn populate_graph() {
        let graph = build_graph_from_xml("data/example.osm");

        assert!(has_node_ids(&graph));
        assert!(node_spot_check(&graph));
        assert!(has_edges_for_nodes(&graph));
        assert!(edge_spot_check(&graph));
    }
}
