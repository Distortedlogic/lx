use std::collections::HashMap;
use std::sync::Mutex;

static CACHE: Mutex<HashMap<String, Vec<f64>>> = Mutex::new(HashMap::new());

fn aggregate(data: &[f64]) -> f64 {
    let collected: Vec<f64> = data.iter().copied().collect();
    collected.iter().sum()
}

fn build_index(items: &[String]) -> HashMap<String, usize> {
    let mut index = HashMap::new();
    for (i, item) in items.iter().enumerate() {
        index.insert(item.clone(), i);
    }
    index
}

fn process_batch(keys: &[String], values: &[f64]) {
    for (key, val) in keys.iter().zip(values) {
        let mut cache = CACHE.lock().unwrap();
        cache
            .entry(key.clone())
            .or_insert_with(Vec::new)
            .push(*val);
    }
}

fn format_results(items: &[String]) -> String {
    let mut result = String::new();
    for item in items {
        result.push_str(&format!("{}, ", item));
    }
    result
}

fn top_scores(scores: &mut Vec<f64>, n: usize) -> Vec<f64> {
    scores.sort_by(|a, b| b.partial_cmp(a).unwrap());
    scores[..n].to_vec()
}

fn lookup_all(map: &HashMap<String, f64>, keys: &[String]) -> Vec<f64> {
    let mut results = Vec::new();
    for key in keys {
        if map.get(key).is_some() {
            results.push(*map.get(key).unwrap());
        }
    }
    results
}

fn main() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let total = aggregate(&data);
    println!("total: {}", total);
}
