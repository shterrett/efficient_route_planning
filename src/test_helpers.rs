#[cfg(test)]
use graph_from_gtfs::{ NodeType, GtfsId, time_to_seconds_after_midnight };

#[cfg(test)]
pub fn floats_nearly_eq(float_1: f64, float_2: f64) -> bool {
    (float_1 - float_2).abs() < 0.0001
}

#[cfg(test)]
pub fn to_node_id(data: (&'static str, &'static str, NodeType, Option<&str>)) -> GtfsId {
    let (id, t, stop_type, trip) = data;

    GtfsId { stop_id: id.to_string(),
                time: time_to_seconds_after_midnight(&t.to_string()).unwrap(),
                node_type: stop_type,
                trip_id: trip.map(|n| n.to_string())
            }
}

