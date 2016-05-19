use std::collections::HashMap;
use std::hash::Hash;

use weighted_graph::{ Graph, Node };
use road_weights::road_weight;

pub fn crow_files<T>() -> Box<Fn(Option<&Node<T>>, Option<&Node<T>>) -> i64>
       where T: Clone + Hash + Eq {
    Box::new(|current: Option<&Node<T>>, target: Option<&Node<T>>| {
        match (current, target) {
            (Some(cnode), Some(tnode)) => {
                road_weight(cnode, tnode, "motorway").unwrap_or(0)
            }
            _ => 0
        }
    })
}

// pub fn landmarks<T>(landmark_distances: HashMap<(T, T), i64>) -> F
//        where F: Fn(Option<Node<T>>, Option<Node<T>>) -> i64 {
// }
// 
// fn build_landmark_distances<T>(graph: &Graph<T>, landmarks: Vec<&T>) {
// }
// 
// fn select_landmarks<T>(graph: &Graph<T>, n: i32) {
// }

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use weighted_graph::{ Graph, Node };
    use road_weights::road_weight;
    use super::crow_files;

    #[test]
    fn calculate_crow_flying_distance() {
        let node_1 = Node { id: "1", x: 0.0, y: 0.0 };
        let node_2 = Node { id: "2", x: 1.0, y: 1.0 };

        let heuristic = crow_files();

        let expected = road_weight(&node_1, &node_2, "motorway").unwrap();
        let actual = heuristic(Some(&node_1), Some(&node_2));

        assert_eq!(actual, expected);
    }

    #[test]
    fn crow_flies_0_if_none() {
        let node_1 = Node { id: "1", x: 0.0, y: 0.0 };

        let heuristic = crow_files();

        let actual = heuristic(Some(&node_1), None);

        assert_eq!(actual, 0);
    }
}
