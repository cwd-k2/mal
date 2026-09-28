use crate::core::ast::{Expression, Pattern};
use mal_frontend::check::ast as checked;

use super::{Continuation, Lowerer};

impl Lowerer {
    /// Lowers a result block with a dynamically scoped mapping from its source binder to a join.
    ///
    /// The join is registered before the block is lowered so transfers within the block can target
    /// it. Removing the mapping afterwards prevents the source identity from leaking into siblings.
    pub(super) fn lower_result_block_with(
        &mut self,
        source_target: mal_frontend::resolve::ast::ValueId,
        block: &checked::ExpressionBlock,
        value_type: &checked::Type,
        result_type: &checked::Type,
        continuation: &mut Continuation<'_>,
        span: mal_syntax::source::Span,
    ) -> Expression {
        let parameter = self.temporary();
        let argument = self.reference(parameter, value_type.clone(), span);
        let join_body = continuation(self, argument);
        let target = crate::core::ast::JoinId(self.joins.len());
        self.joins.push(crate::core::ast::Join {
            parameter: Pattern::Binding {
                id: parameter,
                ty: value_type.clone(),
            },
            body: join_body,
            span,
        });
        let previous = self.result_targets.insert(source_target, target);
        debug_assert!(previous.is_none());
        let mut identity = |_: &mut Lowerer, value: Expression| value;
        let body = self.lower_items_with(&block.items, &block.result, result_type, &mut identity);
        self.result_targets.remove(&source_target);
        body
    }
}
