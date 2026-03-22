use crate::ast::{
    FieldPattern, Pattern, PatternConstructor, PatternList, PatternRecord,
    SPattern,
};
use miette::SourceSpan;

use super::AstFolder;

pub fn fold_pattern<F: AstFolder + ?Sized>(
    f: &mut F,
    pattern: Pattern,
    span: SourceSpan,
) -> SPattern {
    match pattern {
        Pattern::Literal(lit) => {
            let folded = f.fold_literal(lit, span);
            SPattern::new(Pattern::Literal(folded), span)
        },
        Pattern::Bind(name) => SPattern::new(Pattern::Bind(name), span),
        Pattern::Wildcard => SPattern::new(Pattern::Wildcard, span),
        Pattern::Tuple(elems) => {
            let folded = elems
                .into_iter()
                .map(|p| f.fold_pattern(p.node, p.span))
                .collect();
            SPattern::new(Pattern::Tuple(folded), span)
        },
        Pattern::List(pl) => f.fold_pattern_list(pl, span),
        Pattern::Record(pr) => f.fold_pattern_record(pr, span),
        Pattern::Constructor(pc) => f.fold_pattern_constructor(pc, span),
    }
}

pub fn fold_pattern_list<F: AstFolder + ?Sized>(
    f: &mut F,
    pl: PatternList,
    span: SourceSpan,
) -> SPattern {
    let elems = pl
        .elems
        .into_iter()
        .map(|p| f.fold_pattern(p.node, p.span))
        .collect();
    SPattern::new(Pattern::List(PatternList { elems, rest: pl.rest }), span)
}

pub fn fold_pattern_record<F: AstFolder + ?Sized>(
    f: &mut F,
    pr: PatternRecord,
    span: SourceSpan,
) -> SPattern {
    let fields = pr
        .fields
        .into_iter()
        .map(|fp| FieldPattern {
            name: fp.name,
            pattern: fp.pattern.map(|p| f.fold_pattern(p.node, p.span)),
        })
        .collect();
    SPattern::new(Pattern::Record(PatternRecord { fields, rest: pr.rest }), span)
}

pub fn fold_pattern_constructor<F: AstFolder + ?Sized>(
    f: &mut F,
    pc: PatternConstructor,
    span: SourceSpan,
) -> SPattern {
    let args = pc
        .args
        .into_iter()
        .map(|p| f.fold_pattern(p.node, p.span))
        .collect();
    SPattern::new(
        Pattern::Constructor(PatternConstructor { name: pc.name, args }),
        span,
    )
}
