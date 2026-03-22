use crate::ast::{AstArena, FieldPattern, Pattern, PatternConstructor, PatternId, PatternList, PatternRecord};
use miette::SourceSpan;

use super::AstFolder;

pub fn fold_pattern<F: AstFolder + ?Sized>(f: &mut F, id: PatternId, arena: &mut AstArena) -> PatternId {
  let span = arena.pattern_span(id);
  let pattern = arena.pattern(id).clone();
  match pattern {
    Pattern::Literal(lit) => {
      let folded = f.fold_literal(lit, span, arena);
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
    Pattern::List(pl) => f.fold_pattern_list(pl, span, arena),
    Pattern::Record(pr) => f.fold_pattern_record(pr, span, arena),
    Pattern::Constructor(pc) => f.fold_pattern_constructor(pc, span, arena),
  }
}

pub fn fold_pattern_list<F: AstFolder + ?Sized>(f: &mut F, pl: PatternList, span: SourceSpan, arena: &mut AstArena) -> PatternId {
  let elems = pl.elems.into_iter().map(|p| f.fold_pattern(p, arena)).collect();
  arena.alloc_pattern(Pattern::List(PatternList { elems, rest: pl.rest }), span)
}

pub fn fold_pattern_record<F: AstFolder + ?Sized>(f: &mut F, pr: PatternRecord, span: SourceSpan, arena: &mut AstArena) -> PatternId {
  let fields = pr.fields.into_iter().map(|fp| FieldPattern { name: fp.name, pattern: fp.pattern.map(|p| f.fold_pattern(p, arena)) }).collect();
  arena.alloc_pattern(Pattern::Record(PatternRecord { fields, rest: pr.rest }), span)
}

pub fn fold_pattern_constructor<F: AstFolder + ?Sized>(f: &mut F, pc: PatternConstructor, span: SourceSpan, arena: &mut AstArena) -> PatternId {
  let args = pc.args.into_iter().map(|p| f.fold_pattern(p, arena)).collect();
  arena.alloc_pattern(Pattern::Constructor(PatternConstructor { name: pc.name, args }), span)
}
