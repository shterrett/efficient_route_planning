extern crate xml;

use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use self::xml::reader::{ EventReader, XmlEvent };

use prep::weighted_graph::{ Graph, weight };

pub fn build_graph_from_xml(path: &str) -> Graph {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let mut parser = EventReader::new(reader);
    let mut graph = Graph::new();
    let mut eof = false;

    while !eof {
        match parser.next() {
            Ok(e) => match e {
                XmlEvent::StartElement { ref name, ref attributes, .. } => {
                    println!("{:?}", name.local_name);
                    println!("{:?}", attributes);
                    parse_elem(&e, &mut parser, &mut graph);
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

fn parse_elem(e: &XmlEvent, parser: &mut EventReader<BufReader<File>>, graph: &mut Graph) {
    match e {
        &XmlEvent::StartElement { ref name, ref attributes, .. } => {
            let mut map = HashMap::new();
            let mut atrb = attributes.iter().fold(&mut map, |m, attribute| {
                              m.insert(attribute.name.local_name.clone(),
                                       attribute.value.clone());
                              m
                          }
            );
            if name.local_name == "point" {
                graph.add_node(atrb.remove("id").unwrap(),
                               atrb.remove("x").unwrap().parse::<f64>().unwrap(),
                               atrb.remove("y").unwrap().parse::<f64>().unwrap()
                )
            }
        }
        _ => {}
    }
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
