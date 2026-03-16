use std::fs;
use std::collections::HashMap;

fn load_config(path: &str) -> HashMap<String, String> {
    let content = fs::read_to_string(path).unwrap();
    let mut map = HashMap::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        let key = parts.get(0).unwrap().trim().to_string();
        let value = parts.get(1).unwrap().trim().to_string();
        map.insert(key, value);
    }
    map
}

fn get_port(config: &HashMap<String, String>) -> u16 {
    config.get("port").unwrap().parse().unwrap()
}

fn read_users(path: &str) -> Vec<String> {
    let data = fs::read_to_string(path).unwrap();
    serde_json::from_str(&data).unwrap()
}

fn main() {
    let config = load_config("config.txt");
    let port = get_port(&config);
    let users = read_users("users.json");
    println!("Starting on port {} with {} users", port, users.len());
}
