use crate::ast::{AstArena, FieldPattern, Pattern, PatternConstructor, PatternId, PatternList, PatternRecord};
use miette::SourceSpan;

use super::AstFolder;

pub fn fold_pattern<F: AstFolder + ?Sized>(f: &mut F, id: PatternId, arena: &mut AstArena) -> PatternId {
  let span = arena.pattern_span(id);
  let pattern = arena.pattern(id).clone();
  match pattern {
    Pattern::Literal(lit) => {
      let folded = f.fold_literal(lit.clone(), span, arena);
      if folded == lit {
        return id;
      }
      arena.alloc_pattern(Pattern::Literal(folded), span)
    },
    Pattern::Bind(_) | Pattern::Wildcard => id,
    Pattern::Tuple(elems) => {
      let folded: Vec<_> = elems.iter().map(|p| f.fold_pattern(*p, arena)).collect();
      if folded == elems {
        return id;
      }
      arena.alloc_pattern(Pattern::Tuple(folded), span)
    },
    Pattern::List(ref pl) => f.fold_pattern_list(id, pl, span, arena),
    Pattern::Record(ref pr) => f.fold_pattern_record(id, pr, span, arena),
    Pattern::Constructor(ref pc) => f.fold_pattern_constructor(id, pc, span, arena),
  }
}

pub fn fold_pattern_list<F: AstFolder + ?Sized>(f: &mut F, id: PatternId, pl: &PatternList, span: SourceSpan, arena: &mut AstArena) -> PatternId {
  let folded: Vec<_> = pl.elems.iter().map(|p| f.fold_pattern(*p, arena)).collect();
  if folded == pl.elems {
    return id;
  }
  arena.alloc_pattern(Pattern::List(PatternList { elems: folded, rest: pl.rest }), span)
}

pub fn fold_pattern_record<F: AstFolder + ?Sized>(f: &mut F, id: PatternId, pr: &PatternRecord, span: SourceSpan, arena: &mut AstArena) -> PatternId {
  let folded: Vec<_> = pr.fields.iter().map(|fp| FieldPattern { name: fp.name, pattern: fp.pattern.map(|p| f.fold_pattern(p, arena)) }).collect();
  let changed = folded.iter().zip(pr.fields.iter()).any(|(a, b)| a.pattern != b.pattern);
  if !changed {
    return id;
  }
  arena.alloc_pattern(Pattern::Record(PatternRecord { fields: folded, rest: pr.rest }), span)
}

pub fn fold_pattern_constructor<F: AstFolder + ?Sized>(f: &mut F, id: PatternId, pc: &PatternConstructor, span: SourceSpan, arena: &mut AstArena) -> PatternId {
  let folded: Vec<_> = pc.args.iter().map(|p| f.fold_pattern(*p, arena)).collect();
  if folded == pc.args {
    return id;
  }
  arena.alloc_pattern(Pattern::Constructor(PatternConstructor { name: pc.name, args: folded }), span)
}
