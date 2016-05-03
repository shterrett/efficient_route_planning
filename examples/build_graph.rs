extern crate efficient_route_planning;
extern crate time;

use efficient_route_planning::graph_from_xml::build_graph_from_xml;

fn main() {
    {
        println!("Starting saarland.osm");
        let t1 = time::now();
        build_graph_from_xml("data/saarland.osm");
        println!("Done! in {}", time::now() - t1);
    }
    {
        println!("Starting baden-wuerttemberg.osm");
        let t1 = time::now();
        build_graph_from_xml("data/baden-wuerttemberg.osm");
        println!("Done! in {}", time::now() - t1);
    }
}
