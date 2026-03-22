use crate::ast::{MatchArm, Pattern};
use crate::sym::Sym;

pub fn check_exhaustiveness(_type_name: Sym, variants: &[Sym], arms: &[MatchArm]) -> Vec<Sym> {
  let mut covered = std::collections::HashSet::new();
  for arm in arms {
    match &arm.pattern.node {
      Pattern::Constructor { name, .. } => {
        covered.insert(*name);
      },
      Pattern::Wildcard | Pattern::Bind(_) => {
        return Vec::new();
      },
      _ => {},
    }
  }
  variants.iter().filter(|v| !covered.contains(v)).copied().collect()
}
