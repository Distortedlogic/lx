use std::ops::ControlFlow;

use crate::ast::{
  AstArena, ExprAssert, ExprBinary, ExprCoalesce, ExprEmit, ExprFieldAccess, ExprId, ExprNamedArg, ExprPipe, ExprSlice, ExprUnary, ExprYield, FieldKind,
  Section, UnaryOp,
};
use crate::visitor::AstLeave;
use miette::SourceSpan;

use super::diagnostics::DiagnosticKind;
use super::types::Type;
use super::{Checker, DiagLevel};

impl AstLeave for Checker<'_> {
  fn leave_binary(&mut self, binary: &ExprBinary, span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    let rt = self.pop_type();
    let lt = self.pop_type();
    let ty = self.synth_binary_type(&binary.op, &lt, &rt, span);
    self.push_type(ty);
    ControlFlow::Continue(())
  }

  fn leave_unary(&mut self, unary: &ExprUnary, span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    let t = self.pop_type();
    let ty = match unary.op {
      UnaryOp::Neg => match self.table.resolve(&t) {
        Type::Int | Type::Float => t,
        Type::Error => Type::Error,
        _ => {
          self.emit(DiagLevel::Error, DiagnosticKind::NegationRequiresNumeric, span);
          Type::Error
        },
      },
      UnaryOp::Not => Type::Bool,
    };
    self.push_type(ty);
    ControlFlow::Continue(())
  }

  fn leave_pipe(&mut self, _pipe: &ExprPipe, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    let rt = self.pop_type();
    self.pop_type();
    self.push_type(rt);
    ControlFlow::Continue(())
  }

  fn leave_propagate(&mut self, _inner: ExprId, span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    let t = self.pop_type();
    let ty = match self.table.resolve(&t) {
      Type::Result { ok, .. } => *ok,
      Type::Maybe(inner) => *inner,
      Type::Unknown => Type::Unknown,
      Type::Error => Type::Error,
      _ => {
        self.emit(DiagLevel::Error, DiagnosticKind::PropagateRequiresResultOrMaybe, span);
        Type::Error
      },
    };
    self.push_type(ty);
    ControlFlow::Continue(())
  }

  fn leave_coalesce(&mut self, _coalesce: &ExprCoalesce, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    let default_t = self.pop_type();
    self.pop_type();
    self.push_type(default_t);
    ControlFlow::Continue(())
  }

  fn leave_tuple(&mut self, elems: &[ExprId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    let count = elems.len();
    let mut types = Vec::with_capacity(count);
    for _ in 0..count {
      types.push(self.pop_type());
    }
    types.reverse();
    self.push_type(Type::Tuple(types));
    ControlFlow::Continue(())
  }

  fn leave_section(&mut self, section: &Section, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    match section {
      Section::Right { .. } | Section::Left { .. } => {
        self.pop_type();
      },
      _ => {},
    }
    self.push_type(Type::Func { params: vec![Type::Unknown], ret: Box::new(Type::Unknown) });
    ControlFlow::Continue(())
  }

  fn leave_field_access(&mut self, fa: &ExprFieldAccess, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    if let FieldKind::Computed(_) = &fa.field {
      self.pop_type();
    }
    self.pop_type();
    self.push_type(Type::Unknown);
    ControlFlow::Continue(())
  }

  fn leave_named_arg(&mut self, _na: &ExprNamedArg, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    let t = self.pop_type();
    self.push_type(t);
    ControlFlow::Continue(())
  }

  fn leave_emit(&mut self, _emit: &ExprEmit, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    self.pop_type();
    self.push_type(Type::Unit);
    ControlFlow::Continue(())
  }

  fn leave_yield(&mut self, _yld: &ExprYield, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    self.pop_type();
    self.push_type(Type::Unknown);
    ControlFlow::Continue(())
  }

  fn leave_slice(&mut self, slice: &ExprSlice, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    if slice.end.is_some() {
      self.pop_type();
    }
    if slice.start.is_some() {
      self.pop_type();
    }
    let expr_type = self.pop_type();
    self.push_type(expr_type);
    ControlFlow::Continue(())
  }

  fn leave_break(&mut self, value: Option<ExprId>, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    if value.is_some() {
      self.pop_type();
    }
    self.push_type(Type::Unit);
    ControlFlow::Continue(())
  }

  fn leave_assert(&mut self, assert: &ExprAssert, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
    if assert.msg.is_some() {
      self.pop_type();
    }
    self.pop_type();
    self.push_type(Type::Unit);
    ControlFlow::Continue(())
  }
}
