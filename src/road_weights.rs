use std::f64;
use std::collections::HashMap;
use weighted_graph::Node;

lazy_static! {
    pub static ref ROAD_TYPE_SPEED: HashMap<&'static str, i32> = {
        let mut m = HashMap::new();
            m.insert("motorway", 110);
            m.insert("trunk", 110);
            m.insert("primary", 70);
            m.insert("secondary", 60);
            m.insert("tertiary", 50);
            m.insert("motorway_link", 50);
            m.insert("trunk_link", 50);
            m.insert("primary_link", 50);
            m.insert("secondary_link", 50);
            m.insert("road", 40);
            m.insert("unclassified", 40);
            m.insert("residential", 30);
            m.insert("unsurfaced", 30);
            m.insert("living_street", 10);
            m.insert("service", 5);
            m
    };
    static ref RADIUS_EARTH_METERS: f64 = 6371000.0;
}

pub fn road_weight(from: &Node, to: &Node, road_type: &str) -> Option<i64> {
    ROAD_TYPE_SPEED.get(road_type).map(|speed|
       ((haversine(from.x, from.y, to.x, to.y) / *speed as f64) * 3600.0) as i64
    )
}

fn degrees_to_radians(degrees: f64) -> f64 {
    (degrees / 180.0) * f64::consts::PI
}

fn haversine(from_lng: f64, from_lat: f64, to_lng: f64, to_lat: f64) -> f64 {
    let lat1 = degrees_to_radians(from_lat);
    let lat2 = degrees_to_radians(to_lat);
    let dlat = degrees_to_radians(to_lat - from_lat);
    let dlng = degrees_to_radians(to_lng - from_lng);

    let a = (dlat / 2.0).sin().powi(2) +
            lat1.cos() * lat2.cos() * (dlng / 2.0).sin().powi(2);
    let c = 2.0 * (a.sqrt().atan2((1.0 - a).sqrt()));

    *RADIUS_EARTH_METERS * c / 1000.0 // to km
}

#[cfg(test)]
mod test {
    use super::{ ROAD_TYPE_SPEED, road_weight, haversine };
    use weighted_graph::Node;
    use test_helpers::floats_nearly_eq;

    #[test]
    fn test_road_type_speed_construction() {
        assert_eq!(ROAD_TYPE_SPEED.get("motorway"), Some(&110));
    }

    #[test]
    fn test_haversine() {
        let distance_1 = haversine(-71.085743, 42.343212, -71.087792, 42.347249);
        assert!(floats_nearly_eq(distance_1, 0.4794));

        let distance_2 = haversine(-71.085743, 42.343212, -73.982969, 40.773046);
        assert!(floats_nearly_eq(distance_2, 297.6200));
    }

    #[test]
    fn test_road_weight() {
        let node_1 = Node { id: "node-1".to_string(), x: -71.085743, y: 42.343212,..Default::default() };
        let node_2 = Node { id: "node-2".to_string(), x: -71.087792, y: 42.347249, ..Default::default() };

        let motorway_weight = road_weight(&node_1, &node_2, "motorway");
        let road_type_weight = road_weight(&node_1, &node_2, "road");
        let service_weight = road_weight(&node_1, &node_2, "service");
        let not_a_road_weight = road_weight(&node_1, &node_2, "notaroad");

        assert_eq!(motorway_weight.unwrap(), 15);
        assert_eq!(road_type_weight.unwrap(), 43);
        assert_eq!(service_weight.unwrap(), 345);
        assert_eq!(not_a_road_weight, None);
    }
}
