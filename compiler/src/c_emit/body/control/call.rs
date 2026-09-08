use crate::c_emit::syntax::Expr;
use crate::closure::ast::{Atom, AtomKind, Reference};
use crate::control::ast as control;

use super::super::BodyEmitter;

impl BodyEmitter<'_> {
    pub(super) fn control_function(
        &self,
        id: crate::closure::ast::FunctionId,
    ) -> &control::Function {
        self.control
            .functions
            .iter()
            .find(|function| function.id == id)
            .expect("control lowering preserves function identities")
    }

    pub(super) fn emit_local_control_call(&self, callee: &Atom, argument: &Atom) -> Expr {
        if let AtomKind::Reference(Reference::Binding(id)) = callee.kind
            && self.closure_uses.direct_closure(id).is_some_and(|target| {
                self.control_frames
                    .closure_crosses_suspension(target.creator)
            })
        {
            let callee = self.emit_atom(callee);
            return Expr::call(
                callee.clone().field("call"),
                [
                    Expr::identifier("mal_context"),
                    callee.field("environment"),
                    self.emit_atom(argument),
                ],
            );
        }
        self.emit_call(callee, argument, false)
    }
}
