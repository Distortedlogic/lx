use crate::ast::{AstArena, FieldPattern, Pattern, PatternConstructor, PatternId, PatternList, PatternRecord};

use crate::visitor::transformer::{AstTransformer, TransformOp};

pub fn walk_transform_pattern<T: AstTransformer + ?Sized>(t: &mut T, id: PatternId, arena: &mut AstArena) -> PatternId {
  let span = arena.pattern_span(id);
  let original = arena.pattern(id).clone();

  match t.transform_pattern(id, original.clone(), span, arena) {
    TransformOp::Stop => id,
    TransformOp::Skip(node) => {
      let final_node = t.leave_pattern(id, node, span, arena);
      if final_node == original {
        return id;
      }
      arena.alloc_pattern(final_node, span)
    },
    TransformOp::Continue(node) => {
      let recursed = recurse_pattern_children(t, node, arena);
      let final_node = t.leave_pattern(id, recursed, span, arena);
      if final_node == original {
        return id;
      }
      arena.alloc_pattern(final_node, span)
    },
  }
}

fn recurse_pattern_children<T: AstTransformer + ?Sized>(t: &mut T, pattern: Pattern, arena: &mut AstArena) -> Pattern {
  match pattern {
    Pattern::Tuple(elems) => Pattern::Tuple(elems.into_iter().map(|p| walk_transform_pattern(t, p, arena)).collect()),
    Pattern::List(PatternList { elems, rest }) => {
      let folded = elems.into_iter().map(|p| walk_transform_pattern(t, p, arena)).collect();
      Pattern::List(PatternList { elems: folded, rest })
    },
    Pattern::Record(PatternRecord { fields, rest }) => {
      let folded = fields.into_iter().map(|f| FieldPattern { name: f.name, pattern: f.pattern.map(|p| walk_transform_pattern(t, p, arena)) }).collect();
      Pattern::Record(PatternRecord { fields: folded, rest })
    },
    Pattern::Constructor(PatternConstructor { name, args }) => {
      let folded = args.into_iter().map(|p| walk_transform_pattern(t, p, arena)).collect();
      Pattern::Constructor(PatternConstructor { name, args: folded })
    },
    other => other,
  }
}
