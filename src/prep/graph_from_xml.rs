extern crate xml;

use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use self::xml::attribute::OwnedAttribute;
use self::xml::reader::{ EventReader, XmlEvent };

use prep::weighted_graph::{ Graph, weight };

pub fn build_graph_from_xml(path: &str) -> Graph {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let mut parser = EventReader::new(reader);
    let mut graph = Graph::new();
    let mut current_edge_id = "".to_string();
    let mut edge_nodes = vec![];
    let mut eof = false;

    while !eof {
        match parser.next() {
            Ok(e) => match e {
                XmlEvent::StartElement { ref name, ref attributes, .. } => {
                    match name.local_name.as_str() {
                        "point" => {
                            add_node(&mut graph, &attributes);
                        }
                        "edge" => {
                            current_edge_id = get_attribute(&attributes, "id").unwrap_or("".to_string());
                        }
                        "p" => {
                            edge_nodes.push(get_attribute(&attributes, "ref").unwrap_or("".to_string()));
                        }
                        _ => {}
                    }
                }
                XmlEvent::EndElement { ref name, .. } => {
                    match name.local_name.as_str() {
                        "edge" => {
                            add_edge(&mut graph, &current_edge_id, &edge_nodes);
                            current_edge_id = "".to_string();
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

fn add_node(graph: &mut Graph, attributes: &Vec<OwnedAttribute>) {
    let mut map = HashMap::new();
    let mut atrb = attributes.iter().fold(&mut map, |m, attribute| {
                    m.insert(attribute.name.local_name.clone(),
                            attribute.value.clone());
                    m
                }
    );
    graph.add_node(atrb.remove("id").unwrap(),
                atrb.remove("x").unwrap().parse::<f64>().unwrap(),
                atrb.remove("y").unwrap().parse::<f64>().unwrap()
    )
}

fn add_edge(graph: &mut Graph, edge_id: &String, nodes: &Vec<String>) {
    let mut pairs = nodes.windows(2);
    while let Some(pair) = pairs.next() {
        graph.add_edge(edge_id.clone(),
                       pair[0].clone(),
                       pair[1].clone());
        graph.add_edge(edge_id.clone(),
                       pair[1].clone(),
                       pair[0].clone());
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
    use prep::weighted_graph:: { Graph, Node };

    fn has_node_ids(graph: &Graph) -> bool {
        vec!["0", "1", "2", "3", "4", "5", "6"].iter().all(|id|
            graph.get_node(id).is_some()
        )
    }

    fn node_spot_check(graph: &Graph) -> bool {
        match graph.get_node("2") {
            Some(node) => {
                node == &Node { id: "2".to_string(), x: 3.0, y: -1.0 }
            }
            None => false
        }
    }

    fn has_edge_ids(graph: &Graph) -> bool {
        vec![("0", 3), ("1", 5), ("2", 3), ("3", 1), ("4", 3), ("5", 2), ("6", 3)]
            .iter().all(|t|
                graph.get_edges(t.0).is_some() &&
                  graph.get_edges(t.0).unwrap().len() == t.1
            )
    }

    #[test]
    fn populate_graph() {
        let graph = build_graph_from_xml("data/basic_graph.xml");

        assert!(has_node_ids(&graph));
        assert!(node_spot_check(&graph));
        assert!(has_edge_ids(&graph));
    }
}
