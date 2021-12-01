#[allow(dead_code, unused_variables)]
pub fn fill(map: std::collections::HashMap<String, f64>, value: f64) -> std::collections::HashMap<String, f64> {
    let mut return_value = std::collections::HashMap::<String, f64>::new();
    for key in map.keys() {
        return_value.insert(key.to_string(), 1.0);
    }
    return_value
}

pub fn normalize(map: &mut std::collections::HashMap<String, f64>, length: f64) -> std::collections::HashMap<String, f64> {
    let mut return_value = std::collections::HashMap::<String, f64>::new();
    let mut sum = 0.0;
    for value in map.values() {
        sum += value;
    }
    for (key, value) in map {
        let x = ((*value) / sum) * length;
        return_value.insert(key.to_string(), x);
    }
    return_value
}


