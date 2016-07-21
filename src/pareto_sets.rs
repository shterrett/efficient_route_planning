/*
 * Prove that if a pareto set of two-element vectors is ordered by increasing x
 * then every y in the ordering will be strictly decreasing.
 *
 * Proof by induction.
 * Base case, element 0 and element 1:
 * Because the vectors must be incomparable, x0 < x1 => y0 > y1. Otherwise,
 * e0 <= e1
 * Induction:
 * Given e(i), e(i+1) such that x(i) < x(i+1) and y(i) > y(i+1)
 * e(i+2) must not be comparable to e(i+1). Therefore x(i+1) < x(i+2) by hypothesis
 * and y(i+2) < y(i+1) by incomparability requirement.
 * Therefore, for all elements, x is strictly increasing, and y is strictly decreasing
 *
 * Also: I hate writing code to "optimize the number of comparisons". This could
 * be much cleaner if that was not a requirement. In order to return a new (ie copied,
 * not mutated) vector, it is required to iterate through the entire vector anyway.
 * The actual efficency savings is very small. The math bit is interesting though.
 */
use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq)]
pub enum PartialOrdering {
    Less,
    Equal,
    Greater,
    Incomparable
}

pub type Cost = (i64, i64);

pub fn insert_element(ordered_pareto: &Vec<Cost>, new_element: &Cost) -> Vec<Cost> {
    let length = ordered_pareto.len();
    let largest_x = ordered_pareto[length - 1].0;
    let largest_y = ordered_pareto[0].1;
    if new_element.0 >= largest_x && new_element.1 >= largest_y {
        return ordered_pareto.clone()
    }

    let mut new_pareto = vec![];
    let mut compare = true;
    for elem in ordered_pareto {
        match compare {
            true => {
                if new_element.0 <= elem.0 && new_element.1 >= elem.1 {
                    new_pareto.push(new_element.clone());
                    new_pareto.push(elem.clone());
                    compare = false;
                } else if partial_cmp(new_element, &elem) == PartialOrdering::Less {
                    new_pareto.push(new_element.clone());
                    compare = false;
                } else if partial_cmp(new_element, &elem) == PartialOrdering::Greater {
                    new_pareto.push(elem.clone());
                    compare = false;
                } else {
                    new_pareto.push(elem.clone());
                }
            }
            false => {
                new_pareto.push(elem.clone());
            }
        }
    }

    if compare == true {
        new_pareto.push(new_element.clone());
    }

    return new_pareto
}

pub fn partial_cmp(&(x1, y1): &Cost, &(x2, y2): &Cost) -> PartialOrdering {
    match (x1.cmp(&x2), y1.cmp(&y2)) {
        (Ordering::Equal, Ordering::Equal) => PartialOrdering::Equal,
        (Ordering::Less, Ordering::Equal) => PartialOrdering::Less,
        (Ordering::Equal, Ordering::Less) => PartialOrdering::Less,
        (Ordering::Less, Ordering::Less) => PartialOrdering::Less,
        (Ordering::Greater, Ordering::Equal) => PartialOrdering::Greater,
        (Ordering::Equal, Ordering::Greater) => PartialOrdering::Greater,
        (Ordering::Greater, Ordering::Greater) => PartialOrdering::Greater,
        (Ordering::Less, Ordering::Greater) => PartialOrdering::Incomparable,
        (Ordering::Greater, Ordering::Less) => PartialOrdering::Incomparable
    }
}

#[cfg(test)]
mod test {
    use super::{ PartialOrdering,
                 partial_cmp,
                 insert_element
               };

    #[test]
    fn comparing_costs() {
        let cost = (4, 7);
        let less = (3, 5);
        let greater = (5, 9);
        let equal = (4, 7);
        let one_elem_less = (4, 6);
        let one_elem_greater = (5, 7);
        let incomparable = (3, 8);
        let other_incomparable = (5, 6);

        assert_eq!(partial_cmp(&less, &cost), PartialOrdering::Less);
        assert_eq!(partial_cmp(&greater, &cost), PartialOrdering::Greater);
        assert_eq!(partial_cmp(&equal, &cost), PartialOrdering::Equal);
        assert_eq!(partial_cmp(&one_elem_less, &cost), PartialOrdering::Less);
        assert_eq!(partial_cmp(&one_elem_greater, &cost), PartialOrdering::Greater);
        assert_eq!(partial_cmp(&incomparable, &cost), PartialOrdering::Incomparable);
        assert_eq!(partial_cmp(&other_incomparable, &cost), PartialOrdering::Incomparable);
    }

    #[test]
    fn new_element_greater_than_all_existing() {
        let ordered_pareto = vec![(1, 5), (2, 4), (4, 3), (7, 1)];
        let new_element = (8, 6);

        let new_pareto = insert_element(&ordered_pareto, &new_element);

        assert_eq!(new_pareto, ordered_pareto);
    }

    #[test]
    fn new_element_greater_than_one_but_not_all() {
        let ordered_pareto = vec![(1, 5), (2, 4), (4, 3), (7, 1)];
        let new_element = (4, 4);

        let new_pareto = insert_element(&ordered_pareto, &new_element);

        assert_eq!(new_pareto, ordered_pareto);
    }

    #[test]
    fn new_element_incomparable_to_all_existing() {
        let ordered_pareto = vec![(1, 5), (2, 4), (4, 3), (7, 1)];
        let new_element = (5, 2);

        let new_pareto = insert_element(&ordered_pareto, &new_element);

        assert_eq!(new_pareto, vec![(1, 5), (2, 4), (4, 3), (5, 2), (7, 1)]);
    }

    #[test]
    fn new_element_incomparable_to_all_existing_at_end() {
        let ordered_pareto = vec![(1, 5), (2, 4), (4, 3), (7, 1)];
        let new_element = (8, 0);

        let new_pareto = insert_element(&ordered_pareto, &new_element);

        assert_eq!(new_pareto, vec![(1, 5), (2, 4), (4, 3), (7, 1), (8, 0)]);
    }

    #[test]
    fn new_element_incomparable_to_all_existing_at_beginning() {
        let ordered_pareto = vec![(1, 5), (2, 4), (4, 3), (7, 1)];
        let new_element = (0, 6);

        let new_pareto = insert_element(&ordered_pareto, &new_element);

        assert_eq!(new_pareto, vec![(0, 6), (1, 5), (2, 4), (4, 3), (7, 1)]);
    }

    #[test]
    fn new_element_less_than_an_existing_element() {
        let ordered_pareto = vec![(1, 5), (2, 4), (4, 3), (7, 1)];
        let new_element = (3, 2);

        let new_pareto = insert_element(&ordered_pareto, &new_element);

        assert_eq!(new_pareto, vec![(1, 5), (2, 4), (3, 2), (7, 1)]);
    }
}
