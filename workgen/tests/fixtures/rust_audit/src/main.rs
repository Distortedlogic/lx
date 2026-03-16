use std::collections::HashMap;

#[allow(dead_code)]
const MAGIC: i32 = 42;

fn process(data: &String) -> HashMap<String, String> {
    let path = std::path::PathBuf::from(data);
    let content = std::fs::read_to_string(&path).unwrap();
    let mut map = HashMap::new();
    map.insert(path.display().to_string(), content);
    map
}

fn transform(input: Vec<String>) -> Vec<String> {
    let input = input;
    input.into_iter().map(|s| s.to_uppercase()).collect()
}

fn summarize(items: &Vec<String>) -> String {
    let collected: Vec<String> = items.iter().map(|s| s.to_lowercase()).collect();
    let result: Vec<&str> = collected.iter().map(|s| s.as_str()).collect();
    result.join(", ")
}

fn get_status(code: i32) -> &'static str {
    match code {
        200 => "ok",
        404 => "not found",
        _ => "error",
    }
}

fn main() {
    let result = process(&"config.txt".to_string());
    let result = result;
    let _ = std::fs::remove_file("temp.txt");
    let items = transform(vec!["hello".into(), "world".into()]);
    let summary = summarize(&items);
    println!("{:?} {} {}", result, summary, get_status(200));
}
