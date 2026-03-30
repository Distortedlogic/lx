use lx_ast::ast::{FieldPattern, Pattern, PatternConstructor, PatternId, PatternList, PatternRecord};
use lx_span::sym::Sym;

use super::Checker;
use super::semantic::DefKind;
use super::type_arena::TypeId;
use super::types::Type;

const CTOR_SOME: &str = "Some";
const CTOR_OK: &str = "Ok";
const CTOR_ERR: &str = "Err";

impl Checker<'_> {
  pub(super) fn infer_pattern_bindings(&mut self, pid: PatternId, scrutinee_type: TypeId) {
    let resolved = self.table.resolve(scrutinee_type, &self.type_arena);
    let scrut = self.type_arena.get(resolved).clone();
    match scrut {
      Type::Unknown | Type::Todo | Type::Error => self.bind_pattern_unknown(pid),
      _ => self.infer_pattern_from_type(pid, resolved),
    }
  }

  fn bind_pattern_unknown(&mut self, pid: PatternId) {
    let unknown = self.type_arena.unknown();
    let span = self.arena.pattern_span(pid);
    match self.arena.pattern(pid).clone() {
      Pattern::Bind(name) => {
        let def_id = self.sem.add_definition(name, DefKind::PatternBind, span, false);
        self.sem.set_definition_type(def_id, unknown);
      },
      Pattern::Constructor(PatternConstructor { args, .. }) => {
        for arg in &args {
          self.bind_pattern_unknown(*arg);
        }
      },
      Pattern::Tuple(pats) => {
        for p in &pats {
          self.bind_pattern_unknown(*p);
        }
      },
      Pattern::List(PatternList { elems, rest }) => {
        for p in &elems {
          self.bind_pattern_unknown(*p);
        }
        if let Some(name) = rest {
          let def_id = self.sem.add_definition(name, DefKind::PatternBind, span, false);
          self.sem.set_definition_type(def_id, unknown);
        }
      },
      Pattern::Record(PatternRecord { fields, rest }) => {
        for f in &fields {
          if let Some(p) = f.pattern {
            self.bind_pattern_unknown(p);
          } else {
            let def_id = self.sem.add_definition(f.name, DefKind::PatternBind, span, false);
            self.sem.set_definition_type(def_id, unknown);
          }
        }
        if let Some(name) = rest {
          let def_id = self.sem.add_definition(name, DefKind::PatternBind, span, false);
          self.sem.set_definition_type(def_id, unknown);
        }
      },
      Pattern::Literal(_) | Pattern::Wildcard => {},
    }
  }

  fn infer_pattern_from_type(&mut self, pid: PatternId, scrutinee_type: TypeId) {
    let span = self.arena.pattern_span(pid);
    match self.arena.pattern(pid).clone() {
      Pattern::Bind(name) => {
        let def_id = self.sem.add_definition(name, DefKind::PatternBind, span, false);
        self.sem.set_definition_type(def_id, scrutinee_type);
      },
      Pattern::Constructor(PatternConstructor { name, args }) => {
        self.infer_constructor_bindings(name, &args, scrutinee_type);
      },
      Pattern::Tuple(pats) => {
        let scrut = self.type_arena.get(scrutinee_type).clone();
        if let Type::Tuple(elems) = scrut {
          for (i, p) in pats.iter().enumerate() {
            let elem_ty = elems.get(i).copied().unwrap_or_else(|| self.type_arena.unknown());
            self.infer_pattern_from_type(*p, elem_ty);
          }
        } else {
          for p in &pats {
            self.bind_pattern_unknown(*p);
          }
        }
      },
      Pattern::List(PatternList { elems, rest }) => {
        let scrut = self.type_arena.get(scrutinee_type).clone();
        if let Type::List(inner) = scrut {
          for p in &elems {
            self.infer_pattern_from_type(*p, inner);
          }
          if let Some(rest_name) = rest {
            let list_ty = self.type_arena.alloc(Type::List(inner));
            let def_id = self.sem.add_definition(rest_name, DefKind::PatternBind, span, false);
            self.sem.set_definition_type(def_id, list_ty);
          }
        } else {
          for p in &elems {
            self.bind_pattern_unknown(*p);
          }
          if let Some(rest_name) = rest {
            let unknown = self.type_arena.unknown();
            let def_id = self.sem.add_definition(rest_name, DefKind::PatternBind, span, false);
            self.sem.set_definition_type(def_id, unknown);
          }
        }
      },
      Pattern::Record(PatternRecord { fields, rest }) => {
        let scrut = self.type_arena.get(scrutinee_type).clone();
        if let Type::Record(type_fields) = scrut {
          self.infer_record_bindings(&fields, rest, &type_fields, span);
        } else {
          for f in &fields {
            if let Some(p) = f.pattern {
              self.bind_pattern_unknown(p);
            } else {
              let unknown = self.type_arena.unknown();
              let def_id = self.sem.add_definition(f.name, DefKind::PatternBind, span, false);
              self.sem.set_definition_type(def_id, unknown);
            }
          }
          if let Some(rest_name) = rest {
            let unknown = self.type_arena.unknown();
            let def_id = self.sem.add_definition(rest_name, DefKind::PatternBind, span, false);
            self.sem.set_definition_type(def_id, unknown);
          }
        }
      },
      Pattern::Literal(_) | Pattern::Wildcard => {},
    }
  }

  fn infer_constructor_bindings(&mut self, ctor_name: Sym, args: &[PatternId], scrutinee_type: TypeId) {
    let scrut = self.type_arena.get(scrutinee_type).clone();
    let field_types = match &scrut {
      Type::Maybe(inner) => {
        if ctor_name.as_str() == CTOR_SOME {
          vec![*inner]
        } else {
          vec![]
        }
      },
      Type::Result { ok, err } => {
        if ctor_name.as_str() == CTOR_OK {
          vec![*ok]
        } else if ctor_name.as_str() == CTOR_ERR {
          vec![*err]
        } else {
          vec![]
        }
      },
      Type::Union { variants, .. } => variants.iter().find(|v| v.name == ctor_name).map(|v| v.fields.clone()).unwrap_or_default(),
      _ => vec![],
    };
    for (i, arg) in args.iter().enumerate() {
      let field_ty = field_types.get(i).copied().unwrap_or_else(|| self.type_arena.unknown());
      let resolved_field = self.table.resolve(field_ty, &self.type_arena);
      let field_scrut = self.type_arena.get(resolved_field).clone();
      match field_scrut {
        Type::Unknown | Type::Todo | Type::Error => self.bind_pattern_unknown(*arg),
        _ => self.infer_pattern_from_type(*arg, resolved_field),
      }
    }
  }

  fn infer_record_bindings(&mut self, fields: &[FieldPattern], rest: Option<Sym>, type_fields: &[(Sym, TypeId)], span: miette::SourceSpan) {
    let mut matched_names = Vec::new();
    for f in fields {
      let field_ty = type_fields.iter().find(|(n, _)| *n == f.name).map(|(_, t)| *t).unwrap_or_else(|| self.type_arena.unknown());
      if let Some(p) = f.pattern {
        self.infer_pattern_from_type(p, field_ty);
      } else {
        let def_id = self.sem.add_definition(f.name, DefKind::PatternBind, span, false);
        self.sem.set_definition_type(def_id, field_ty);
      }
      matched_names.push(f.name);
    }
    if let Some(rest_name) = rest {
      let remaining: Vec<(Sym, TypeId)> = type_fields.iter().filter(|(n, _)| !matched_names.contains(n)).cloned().collect();
      let rest_ty = self.type_arena.alloc(Type::Record(remaining));
      let def_id = self.sem.add_definition(rest_name, DefKind::PatternBind, span, false);
      self.sem.set_definition_type(def_id, rest_ty);
    }
  }
}
