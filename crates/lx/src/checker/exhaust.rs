use crate::ast::{AstArena, FieldPattern, Literal, MatchArm, Pattern, PatternConstructor, PatternId, PatternList, PatternRecord, StrPart};
use crate::sym::{self, Sym};

use super::exhaust_core::find_witnesses;
use super::exhaust_types::{LitPat, Pat};

pub fn check_exhaustiveness(_type_name: Sym, variants: &[(Sym, usize)], arms: &[MatchArm], arena: &AstArena) -> Vec<String> {
  let matrix: Vec<Vec<Pat>> = arms.iter().filter(|arm| arm.guard.is_none()).map(|arm| vec![ast_pattern_to_pat(arm.pattern, arena)]).collect();

  let wildcard_row = vec![Pat::Wildcard];
  let witnesses = find_witnesses(&matrix, &wildcard_row, Some(variants));

  witnesses.iter().map(|w| format!("{w}")).collect()
}

pub fn check_exhaustiveness_no_variants(type_name: Sym, arms: &[MatchArm], arena: &AstArena) -> Vec<String> {
  let matrix: Vec<Vec<Pat>> = arms.iter().filter(|arm| arm.guard.is_none()).map(|arm| vec![ast_pattern_to_pat(arm.pattern, arena)]).collect();

  let wildcard_row = vec![Pat::Wildcard];
  let witnesses = find_witnesses(&matrix, &wildcard_row, None);

  witnesses
    .iter()
    .map(|w| {
      let w_str = format!("{w}");
      if w_str == "_" { format!("pattern for {type_name}") } else { w_str }
    })
    .collect()
}

fn ast_pattern_to_pat(pid: PatternId, arena: &AstArena) -> Pat {
  match arena.pattern(pid) {
    Pattern::Wildcard | Pattern::Bind(_) => Pat::Wildcard,
    Pattern::Constructor(PatternConstructor { name, args }) => {
      let converted_args: Vec<Pat> = args.iter().map(|a| ast_pattern_to_pat(*a, arena)).collect();
      Pat::Constructor { name: *name, arity: converted_args.len(), args: converted_args }
    },
    Pattern::Tuple(pats) => {
      let elems: Vec<Pat> = pats.iter().map(|p| ast_pattern_to_pat(*p, arena)).collect();
      Pat::Tuple(elems)
    },
    Pattern::Literal(lit) => Pat::Literal(literal_to_litpat(lit)),
    Pattern::List(list) => list_pattern_to_pat(list, arena),
    Pattern::Record(rec) => record_pattern_to_pat(rec, arena),
  }
}

fn list_pattern_to_pat(list: &PatternList, arena: &AstArena) -> Pat {
  let nil = sym::intern("Nil");
  let cons = sym::intern("Cons");
  if list.elems.is_empty() {
    if list.rest.is_some() {
      return Pat::Wildcard;
    }
    return Pat::Constructor { name: nil, arity: 0, args: Vec::new() };
  }
  let mut result = if list.rest.is_some() { Pat::Wildcard } else { Pat::Constructor { name: nil, arity: 0, args: Vec::new() } };
  for elem in list.elems.iter().rev() {
    let head = ast_pattern_to_pat(*elem, arena);
    result = Pat::Constructor { name: cons, arity: 2, args: vec![head, result] };
  }
  result
}

fn record_pattern_to_pat(rec: &PatternRecord, arena: &AstArena) -> Pat {
  if rec.rest.is_some() {
    return Pat::Wildcard;
  }
  let mut fields: Vec<&FieldPattern> = rec.fields.iter().collect();
  fields.sort_by_key(|f| f.name);
  let elems: Vec<Pat> = fields
    .iter()
    .map(|f| match f.pattern {
      Some(pid) => ast_pattern_to_pat(pid, arena),
      None => Pat::Wildcard,
    })
    .collect();
  Pat::Tuple(elems)
}

fn literal_to_litpat(lit: &Literal) -> LitPat {
  match lit {
    Literal::Int(n) => LitPat::Int(n.clone()),
    Literal::Bool(b) => LitPat::Bool(*b),
    Literal::Unit => LitPat::Unit,
    Literal::Str(parts) => {
      let s: String = parts
        .iter()
        .map(|p| match p {
          StrPart::Text(t) => t.clone(),
          StrPart::Interp(_) => "_".into(),
        })
        .collect();
      LitPat::Str(s)
    },
    Literal::RawStr(s) => LitPat::Str(s.clone()),
    Literal::Float(v) => LitPat::Float(*v),
  }
}
