use std::collections::HashMap;

pub fn handle_request(params: &HashMap<String, String>) -> String {
    let user_id = params.get("user_id").cloned().unwrap_or_default();
    if user_id.is_empty() {
        return "error".to_string();
    }

    let data = fetch_user(&user_id);
    match data {
        Some(user) => format!("found: {}", user),
        None => "not found".to_string(),
    }
}

fn fetch_user(id: &str) -> Option<String> {
    let users = load_users();
    users.get(id).cloned()
}

fn load_users() -> HashMap<String, String> {
    let content = std::fs::read_to_string("users.json").unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn validate_input(input: &str) -> Result<(), String> {
    if input.len() > 1000 {
        return Err("too long".into());
    }
    if input.contains('<') {
        return Err("html detected".into());
    }
    Ok(())
}

pub fn log_error(msg: &str) {
    let _ = std::fs::OpenOptions::new()
        .append(true)
        .open("errors.log")
        .map(|mut f| {
            use std::io::Write;
            let _ = writeln!(f, "{}", msg);
        });
}
