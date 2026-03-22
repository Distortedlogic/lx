use crate::sym::Sym;

use super::exhaust_types::{CtorId, LitPat, Pat};

pub fn find_witnesses(matrix: &[Vec<Pat>], pattern: &[Pat], type_variants: Option<&[(Sym, usize)]>) -> Vec<Pat> {
  if pattern.is_empty() {
    return if matrix.is_empty() { vec![Pat::Wildcard] } else { Vec::new() };
  }

  let first = &pattern[0];
  let rest = &pattern[1..];

  match first {
    Pat::Wildcard => {
      let ctors = collect_constructors(matrix);
      if is_complete(&ctors, type_variants) {
        find_witnesses_complete(&ctors, matrix, rest, type_variants)
      } else if let Some(variants) = type_variants {
        let missing_ctors = find_missing_ctors(&ctors, variants);
        if missing_ctors.is_empty() {
          find_witnesses_default(matrix, rest, type_variants)
        } else {
          find_witnesses_missing(&missing_ctors, matrix, rest, type_variants)
        }
      } else {
        find_witnesses_default(matrix, rest, type_variants)
      }
    },
    Pat::Constructor { name, arity, args } => {
      let specialized = specialize_ctor(matrix, &CtorId::Named(*name), *arity);
      let mut sub_pat = args.clone();
      sub_pat.extend_from_slice(rest);
      let witnesses = find_witnesses(&specialized, &sub_pat, type_variants);
      let ctor = CtorId::Named(*name);
      witnesses.into_iter().map(|w| ctor.reconstruct(*arity, &w)).collect()
    },
    Pat::Literal(lit) => {
      let specialized = specialize_ctor(matrix, &CtorId::Literal(lit.clone()), 0);
      find_witnesses(&specialized, rest, type_variants)
    },
    Pat::Tuple(elems) => {
      let arity = elems.len();
      let specialized = specialize_ctor(matrix, &CtorId::Tuple(arity), arity);
      let mut sub_pat = elems.clone();
      sub_pat.extend_from_slice(rest);
      let witnesses = find_witnesses(&specialized, &sub_pat, type_variants);
      let ctor = CtorId::Tuple(arity);
      witnesses.into_iter().map(|w| ctor.reconstruct(arity, &w)).collect()
    },
  }
}

fn find_witnesses_complete(ctors: &[(CtorId, usize)], matrix: &[Vec<Pat>], rest: &[Pat], type_variants: Option<&[(Sym, usize)]>) -> Vec<Pat> {
  let mut all = Vec::new();
  for (ctor_id, arity) in ctors {
    let specialized = specialize_ctor(matrix, ctor_id, *arity);
    let mut sub_pat = vec![Pat::Wildcard; *arity];
    sub_pat.extend_from_slice(rest);
    let witnesses = find_witnesses(&specialized, &sub_pat, type_variants);
    for w in witnesses {
      all.push(ctor_id.reconstruct(*arity, &w));
    }
  }
  all
}

fn find_witnesses_default(matrix: &[Vec<Pat>], rest: &[Pat], type_variants: Option<&[(Sym, usize)]>) -> Vec<Pat> {
  let default = default_matrix(matrix);
  find_witnesses(&default, rest, type_variants)
}

fn find_witnesses_missing(missing_ctors: &[(Sym, usize)], matrix: &[Vec<Pat>], rest: &[Pat], type_variants: Option<&[(Sym, usize)]>) -> Vec<Pat> {
  let mut all = Vec::new();
  for (name, arity) in missing_ctors {
    let specialized = specialize_ctor(matrix, &CtorId::Named(*name), *arity);
    let mut sub_pat = vec![Pat::Wildcard; *arity];
    sub_pat.extend_from_slice(rest);
    let witnesses = find_witnesses(&specialized, &sub_pat, type_variants);
    for w in witnesses {
      let ctor = CtorId::Named(*name);
      all.push(ctor.reconstruct(*arity, &w));
    }
  }
  all
}

fn find_missing_ctors(seen: &[(CtorId, usize)], variants: &[(Sym, usize)]) -> Vec<(Sym, usize)> {
  variants.iter().filter(|(name, _)| !seen.iter().any(|(id, _)| matches!(id, CtorId::Named(n) if *n == *name))).copied().collect()
}

pub fn specialize_ctor(matrix: &[Vec<Pat>], ctor: &CtorId, arity: usize) -> Vec<Vec<Pat>> {
  let mut result = Vec::new();
  for row in matrix {
    if row.is_empty() {
      continue;
    }
    let first = &row[0];
    let rest = &row[1..];
    match first {
      Pat::Constructor { name, args, .. } => {
        if matches!(ctor, CtorId::Named(n) if *n == *name) {
          let mut new_row = args.clone();
          new_row.extend_from_slice(rest);
          result.push(new_row);
        }
      },
      Pat::Literal(lit) => {
        if matches!(ctor, CtorId::Literal(l) if *l == *lit) {
          result.push(rest.to_vec());
        }
      },
      Pat::Tuple(elems) => {
        if matches!(ctor, CtorId::Tuple(n) if *n == elems.len()) {
          let mut new_row = elems.clone();
          new_row.extend_from_slice(rest);
          result.push(new_row);
        }
      },
      Pat::Wildcard => {
        let mut new_row = vec![Pat::Wildcard; arity];
        new_row.extend_from_slice(rest);
        result.push(new_row);
      },
    }
  }
  result
}

pub fn default_matrix(matrix: &[Vec<Pat>]) -> Vec<Vec<Pat>> {
  let mut result = Vec::new();
  for row in matrix {
    if row.is_empty() {
      continue;
    }
    if matches!(row[0], Pat::Wildcard) {
      result.push(row[1..].to_vec());
    }
  }
  result
}

pub fn collect_constructors(matrix: &[Vec<Pat>]) -> Vec<(CtorId, usize)> {
  let mut seen = Vec::new();
  for row in matrix {
    if row.is_empty() {
      continue;
    }
    match &row[0] {
      Pat::Constructor { name, arity, .. } => {
        let id = CtorId::Named(*name);
        if !seen.iter().any(|(c, _)| *c == id) {
          seen.push((id, *arity));
        }
      },
      Pat::Literal(lit) => {
        let id = CtorId::Literal(lit.clone());
        if !seen.iter().any(|(c, _)| *c == id) {
          seen.push((id, 0));
        }
      },
      Pat::Tuple(elems) => {
        let id = CtorId::Tuple(elems.len());
        if !seen.iter().any(|(c, _)| *c == id) {
          seen.push((id, elems.len()));
        }
      },
      Pat::Wildcard => {},
    }
  }
  seen
}

fn is_complete(ctors: &[(CtorId, usize)], type_variants: Option<&[(Sym, usize)]>) -> bool {
  let has_true = ctors.iter().any(|(c, _)| matches!(c, CtorId::Literal(LitPat::Bool(true))));
  let has_false = ctors.iter().any(|(c, _)| matches!(c, CtorId::Literal(LitPat::Bool(false))));
  if has_true && has_false {
    return true;
  }
  let has_unit = ctors.iter().any(|(c, _)| matches!(c, CtorId::Literal(LitPat::Unit)));
  if has_unit {
    return true;
  }
  if ctors.iter().any(|(c, _)| matches!(c, CtorId::Tuple(_))) {
    return true;
  }
  if let Some(variants) = type_variants
    && !variants.is_empty()
    && variants.iter().all(|(name, _)| ctors.iter().any(|(c, _)| matches!(c, CtorId::Named(n) if *n == *name)))
  {
    return true;
  }
  false
}
