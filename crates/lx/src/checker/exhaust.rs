use crate::ast::{MatchArm, Pattern};

pub fn check_exhaustiveness(
    _type_name: &str,
    variants: &[String],
    arms: &[MatchArm],
) -> Vec<String> {
    let mut covered = std::collections::HashSet::new();
    for arm in arms {
        match &arm.pattern.node {
            Pattern::Constructor { name, .. } => {
                covered.insert(name.as_str());
            }
            Pattern::Wildcard | Pattern::Bind(_) => {
                return Vec::new();
            }
            _ => {}
        }
    }
    variants
        .iter()
        .filter(|v| !covered.contains(v.as_str()))
        .cloned()
        .collect()
}
