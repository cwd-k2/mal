use crate::resolve::ast as resolved;
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::ast::{self, Binding, BodyItem, Pattern, Type};
use super::{CheckFailure, CheckResult, Checker, types};

impl Checker {
    pub(super) fn check_binding(
        &mut self,
        binding: &resolved::Binding,
        span: Span,
    ) -> CheckResult<Binding> {
        let annotation = binding
            .annotation
            .as_ref()
            .map(|ty| self.expand_type(ty))
            .transpose()?;
        if let (
            Some(annotation),
            resolved::Pattern::Binding(pattern_binding),
            resolved::Expression::Lambda(lambda),
        ) = (&annotation, &binding.pattern.kind, &binding.value.kind)
            && lambda.self_binding == Some(pattern_binding.id)
        {
            self.values.insert(pattern_binding.id, annotation.clone());
        }
        let value = match self.check_expression(&binding.value, annotation.as_ref()) {
            Ok(value) => value,
            Err(CheckFailure::Abrupt(abrupt)) => {
                return Err(
                    Diagnostic::error("binding initializer must produce a value")
                        .with_primary(abrupt.span, abrupt_reason(&abrupt.kind))
                        .with_note(
                            "a binding needs a value: let one branch or continuation produce it, \
                             for example `(n) -> n`, or write this as a statement",
                        )
                        .into(),
                );
            }
            Err(error) => return Err(error),
        };
        let pattern = self.check_pattern(&binding.pattern, &value.ty)?;
        Ok(Binding {
            pattern,
            annotation,
            value,
            span,
        })
    }

    pub(super) fn check_pattern(
        &mut self,
        pattern: &Node<resolved::Pattern>,
        ty: &Type,
    ) -> CheckResult<Pattern> {
        match &pattern.kind {
            resolved::Pattern::Binding(binding) => {
                self.values.insert(binding.id, ty.clone());
                Ok(Pattern::Binding {
                    binding: binding.clone(),
                    ty: ty.clone(),
                })
            }
            resolved::Pattern::Wildcard => Ok(Pattern::Wildcard {
                ty: ty.clone(),
                span: pattern.span,
            }),
            resolved::Pattern::Product(elements) => {
                let viewed = types::representation_view(ty, pattern.span.file());
                let Type::Product(element_types) = viewed else {
                    return Err(
                        Diagnostic::error("product pattern requires a product value")
                            .with_primary(
                                pattern.span,
                                format!("this value has type `{}`", types::type_name(ty)),
                            )
                            .into(),
                    );
                };
                if elements.len() != element_types.len() {
                    return Err(Diagnostic::error("product pattern has the wrong arity")
                        .with_primary(
                            pattern.span,
                            format!(
                                "expected {} elements, found {}",
                                element_types.len(),
                                elements.len()
                            ),
                        )
                        .into());
                }
                Ok(Pattern::Product {
                    elements: elements
                        .iter()
                        .zip(element_types.iter())
                        .map(|(element, ty)| self.check_pattern(element, ty))
                        .collect::<Result<_, _>>()?,
                    ty: ty.clone(),
                    span: pattern.span,
                })
            }
        }
    }

    fn check_body_item(&mut self, item: &resolved::BodyItem) -> CheckResult<BodyItem> {
        match item {
            resolved::BodyItem::Binding(binding) => Ok(BodyItem::Binding(Box::new(
                self.check_binding(&binding.kind, binding.span)?,
            ))),
            resolved::BodyItem::Expression(expression) => Ok(BodyItem::Expression(Box::new(
                self.check_expression(expression, None)?,
            ))),
        }
    }

    pub(super) fn check_body_items(
        &mut self,
        items: &[resolved::BodyItem],
        result_span: mal_syntax::source::Span,
    ) -> CheckResult<Vec<BodyItem>> {
        let mut checked = Vec::with_capacity(items.len());
        for (index, item) in items.iter().enumerate() {
            match self.check_body_item(item) {
                Ok(item) => checked.push(item),
                Err(CheckFailure::Abrupt(abrupt)) => {
                    let unreachable_span = items
                        .get(index + 1)
                        .map(|item| match item {
                            resolved::BodyItem::Binding(binding) => binding.span,
                            resolved::BodyItem::Expression(expression) => expression.span,
                        })
                        .unwrap_or(result_span);
                    return Err(Diagnostic::error("unreachable code after abrupt completion")
                        .with_primary(
                            unreachable_span,
                            format!(
                                "this expression cannot be reached after control leaves at byte {}",
                                abrupt.span.start()
                            ),
                        )
                        .into());
                }
                Err(error) => return Err(error),
            }
        }
        Ok(checked)
    }
}

/// Why an expression that never produces a value cannot initialize a binding.
fn abrupt_reason(kind: &ast::AbruptExpressionKind) -> &'static str {
    match kind {
        ast::AbruptExpressionKind::ResultTransfer { .. } => {
            "applying a result binder leaves the block, so this never produces a value"
        }
        ast::AbruptExpressionKind::EmptyElimination { .. } => {
            "an empty sum has no value, so this never produces one"
        }
        ast::AbruptExpressionKind::If { .. } => {
            "both branches leave the block, so this never produces a value"
        }
        ast::AbruptExpressionKind::SumElimination { .. } => {
            "every continuation leaves the block, so this never produces a value"
        }
        ast::AbruptExpressionKind::Block(_) => {
            "this block never completes normally, so it produces no value"
        }
    }
}
