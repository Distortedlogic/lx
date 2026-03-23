macro_rules! type_visitor_hooks {
  () => {
    fn visit_type_expr(&mut self, _id: TypeExprId, _type_expr: &TypeExpr, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
      VisitAction::Descend
    }
    fn leave_type_expr(&mut self, _id: TypeExprId, _type_expr: &TypeExpr, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
      ControlFlow::Continue(())
    }
    fn visit_type_named(&mut self, _id: TypeExprId, _name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
      VisitAction::Descend
    }
    fn visit_type_var(&mut self, _id: TypeExprId, _name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
      VisitAction::Descend
    }
    fn visit_type_applied(&mut self, _id: TypeExprId, _name: Sym, _args: &[TypeExprId], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
      VisitAction::Descend
    }
    fn leave_type_applied(&mut self, _id: TypeExprId, _name: Sym, _args: &[TypeExprId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
      ControlFlow::Continue(())
    }
    fn visit_type_list(&mut self, _id: TypeExprId, _inner: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
      VisitAction::Descend
    }
    fn leave_type_list(&mut self, _id: TypeExprId, _inner: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
      ControlFlow::Continue(())
    }
    fn visit_type_map(&mut self, _id: TypeExprId, _key: TypeExprId, _value: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
      VisitAction::Descend
    }
    fn leave_type_map(&mut self, _id: TypeExprId, _key: TypeExprId, _value: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
      ControlFlow::Continue(())
    }
    fn visit_type_record(&mut self, _id: TypeExprId, _fields: &[TypeField], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
      VisitAction::Descend
    }
    fn leave_type_record(&mut self, _id: TypeExprId, _fields: &[TypeField], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
      ControlFlow::Continue(())
    }
    fn visit_type_tuple(&mut self, _id: TypeExprId, _elems: &[TypeExprId], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
      VisitAction::Descend
    }
    fn leave_type_tuple(&mut self, _id: TypeExprId, _elems: &[TypeExprId], _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
      ControlFlow::Continue(())
    }
    fn visit_type_func(&mut self, _id: TypeExprId, _param: TypeExprId, _ret: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
      VisitAction::Descend
    }
    fn leave_type_func(&mut self, _id: TypeExprId, _param: TypeExprId, _ret: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
      ControlFlow::Continue(())
    }
    fn visit_type_fallible(&mut self, _id: TypeExprId, _ok: TypeExprId, _err: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
      VisitAction::Descend
    }
    fn leave_type_fallible(&mut self, _id: TypeExprId, _ok: TypeExprId, _err: TypeExprId, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()> {
      ControlFlow::Continue(())
    }
  };
}

pub(crate) use type_visitor_hooks;
