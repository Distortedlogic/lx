use std::collections::HashMap;

pub fn helper_process(input: String) -> String {
    let trimmed = input.trim().to_string();
    let upper = trimmed.to_uppercase();
    let result = format!("processed: {}", upper);
    result
}

pub fn helper_validate(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }
    if input.len() > 500 {
        return false;
    }
    true
}

pub fn helper_format(items: Vec<String>) -> String {
    let mut output = String::new();
    for item in items {
        output.push_str(&item);
        output.push('\n');
    }
    output
}

pub fn helper_parse_int(s: &str) -> i32 {
    s.parse().unwrap_or(0)
}
