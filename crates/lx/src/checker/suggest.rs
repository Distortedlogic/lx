use similar::TextDiff;

pub fn closest_matches(target: &str, candidates: &[&str], max: usize) -> Vec<String> {
  if target.is_empty() || candidates.is_empty() {
    return Vec::new();
  }
  let mut scored: Vec<(f32, &str)> = candidates.iter().map(|&c| (TextDiff::from_chars(target, c).ratio(), c)).filter(|(ratio, _)| *ratio > 0.6).collect();
  scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
  scored.into_iter().take(max).map(|(_, name)| name.to_string()).collect()
}

pub fn format_suggestions(suggestions: &[String]) -> Option<String> {
  match suggestions.len() {
    0 => None,
    1 => Some(format!("did you mean '{}'?", suggestions[0])),
    _ => Some(format!("did you mean one of: {}?", suggestions.join(", "))),
  }
}
