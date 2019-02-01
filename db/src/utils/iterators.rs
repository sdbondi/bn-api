pub fn intersection<T: PartialEq + Clone>(one: &[T], other: &[T]) -> Vec<T> {
    let mut result = vec![];
    for o in one.iter() {
        if other.contains(&o) {
            result.push(o.clone());
        }
    }
    result
}

pub fn intersect_set<T: PartialEq + Clone>(sets: &[Vec<T>]) -> Vec<T> {
    if sets.len() == 0 {
        return vec![];
    }
    let mut result = sets[0].clone();
    for i in 1..sets.len() {
        result = intersection(&result, &sets[i]);
    }
    result
}
