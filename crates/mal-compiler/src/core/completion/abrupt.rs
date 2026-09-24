use crate::core::ast::{Binding, Expression, ExpressionKind, Pattern};
use mal_frontend::check::ast as checked;

use super::Lowerer;

impl Lowerer {
    pub(super) fn lower_abrupt(
        &mut self,
        abrupt: &checked::AbruptExpression,
        result_type: &checked::Type,
    ) -> Expression {
        let terminal = match &abrupt.kind {
            checked::AbruptExpressionKind::ResultTransfer {
                target: source_target,
                value,
            } => {
                let target = *self
                    .result_targets
                    .get(source_target)
                    .expect("a result binder is lowered inside its result block");
                let span = abrupt.span;
                let mut jump = |_: &mut Lowerer, value: Expression| Expression {
                    kind: ExpressionKind::Goto {
                        target,
                        value: Box::new(value),
                    },
                    ty: result_type.clone(),
                    span,
                };
                self.lower_value_with(value, result_type, &mut jump)
            }
            checked::AbruptExpressionKind::EmptyElimination { scrutinee } => {
                let span = abrupt.span;
                let mut eliminate = |_lowerer: &mut Lowerer, value: Expression| Expression {
                    kind: ExpressionKind::Case {
                        scrutinee: Box::new(value),
                        arms: Vec::new(),
                    },
                    ty: result_type.clone(),
                    span,
                };
                self.lower_value_with(scrutinee, result_type, &mut eliminate)
            }
            checked::AbruptExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.lower_control_if_body(
                condition,
                then_branch,
                else_branch,
                result_type,
                &mut |_: &mut Lowerer, value| value,
                abrupt.span,
            ),
            checked::AbruptExpressionKind::Block(block) => {
                let mut identity = |_: &mut Lowerer, value: Expression| value;
                self.lower_items_with(&block.items, &block.result, result_type, &mut identity)
            }
        };
        self.lower_preceding(&abrupt.preceding, terminal, result_type, abrupt.span)
    }

    fn lower_preceding(
        &mut self,
        preceding: &[checked::Expression],
        terminal: Expression,
        result_type: &checked::Type,
        span: mal_syntax::source::Span,
    ) -> Expression {
        let mut body = terminal;
        for value in preceding.iter().rev() {
            let mut rest = Some(body);
            let mut next = |_: &mut Lowerer, value: Expression| {
                let value_span = value.span;
                Expression {
                    kind: ExpressionKind::Let {
                        binding: Box::new(Binding {
                            pattern: Pattern::Wildcard {
                                ty: value.ty.clone(),
                                span: value_span,
                            },
                            value,
                            span: value_span,
                        }),
                        body: Box::new(rest.take().expect("a join continuation is lowered once")),
                    },
                    ty: result_type.clone(),
                    span,
                }
            };
            body = self.lower_value_with(value, result_type, &mut next);
        }
        body
    }
}
