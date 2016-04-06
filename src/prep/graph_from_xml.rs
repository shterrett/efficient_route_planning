extern crate xml;

use std::fs::File;
use std::io::BufReader;
use self::xml::reader::{ EventReader, XmlEvent };

pub fn readxml(path: &str) {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let parser = EventReader::new(reader);
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                println!("{:?}", name.local_name);
                println!("{:?}", attributes);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod test {
    use super::{ readxml };

    #[test]
    fn can_read() {
        readxml("data/basic_graph.xml");
        assert_eq!(1, 2);
    }
}
