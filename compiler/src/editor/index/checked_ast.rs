use crate::check::ast as checked;

use super::Index;

impl Index<'_> {
    pub(super) fn collect_checked_top(&mut self, item: &checked::TopItem) {
        match item {
            checked::TopItem::TypeAlias { binding, ty } => {
                self.type_details
                    .insert(binding.id, crate::check::type_name(ty));
            }
            checked::TopItem::ExternalType { binding } => {
                self.type_details
                    .insert(binding.id, binding.name.text.clone());
            }
            checked::TopItem::ExternalOperation { .. } => {}
            checked::TopItem::Binding(binding) => self.collect_checked_binding(binding),
        }
    }

    fn collect_checked_binding(&mut self, binding: &checked::Binding) {
        self.collect_checked_pattern(&binding.pattern);
        self.collect_checked_expression(&binding.value);
    }

    fn collect_checked_pattern(&mut self, pattern: &checked::Pattern) {
        match pattern {
            checked::Pattern::Binding { binding, ty } => {
                let id = self.canonical_value(binding.id);
                self.value_types.insert(id, crate::check::type_name(ty));
                self.typed_regions
                    .push((binding.name.span, crate::check::type_name(ty)));
            }
            checked::Pattern::Wildcard { ty, span } => {
                self.typed_regions
                    .push((*span, crate::check::type_name(ty)));
            }
            checked::Pattern::Product { elements, ty, span } => {
                self.typed_regions
                    .push((*span, crate::check::type_name(ty)));
                for element in elements {
                    self.collect_checked_pattern(element);
                }
            }
        }
    }

    fn collect_checked_expression(&mut self, expression: &checked::Expression) {
        use checked::ExpressionKind;
        let ty = crate::check::type_name(&expression.ty);
        self.typed_regions.push((expression.span, ty.clone()));
        match &expression.kind {
            ExpressionKind::Reference(reference) => {
                self.value_types
                    .entry(self.canonical_value(reference.id))
                    .or_insert(ty);
            }
            ExpressionKind::MemoryFunction { .. } => {}
            ExpressionKind::Product(elements) => {
                for element in elements {
                    self.collect_checked_expression(element);
                }
            }
            ExpressionKind::Parenthesized(inner) => self.collect_checked_expression(inner),
            ExpressionKind::Lambda(lambda) => {
                for capture in &lambda.captures {
                    self.value_types.insert(
                        self.canonical_value(capture.binding.id),
                        crate::check::type_name(&capture.ty),
                    );
                }
                for parameter in &lambda.parameters {
                    let id = self.canonical_value(parameter.binding.id);
                    self.parameters.insert(id);
                    self.value_types
                        .insert(id, crate::check::type_name(&parameter.ty));
                    self.typed_regions
                        .push((parameter.span, crate::check::type_name(&parameter.ty)));
                }
                self.collect_checked_body(&lambda.body.items, &lambda.body.result);
            }
            ExpressionKind::Call { callee, argument } => {
                self.collect_checked_expression(callee);
                self.collect_checked_expression(argument);
            }
            ExpressionKind::SymbolLength { value }
            | ExpressionKind::Memory {
                argument: value, ..
            }
            | ExpressionKind::NumericConversion { value }
            | ExpressionKind::SumInjection { value, .. } => self.collect_checked_expression(value),
            ExpressionKind::SymbolAt { argument } => self.collect_checked_expression(argument),
            ExpressionKind::ExternalCall { argument, .. } => {
                self.collect_checked_expression(argument);
            }
            ExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.collect_checked_expression(condition);
                self.collect_checked_body(&then_branch.items, &then_branch.result);
                self.collect_checked_body(&else_branch.items, &else_branch.result);
            }
            ExpressionKind::Case { scrutinee, arms } => {
                self.collect_checked_expression(scrutinee);
                for arm in arms {
                    self.collect_checked_pattern(&arm.pattern);
                    self.collect_checked_body(&arm.body.items, &arm.body.result);
                }
            }
            ExpressionKind::Unary { operand, .. } => self.collect_checked_expression(operand),
            ExpressionKind::Binary { left, right, .. } => {
                self.collect_checked_expression(left);
                self.collect_checked_expression(right);
            }
            ExpressionKind::Integer(_)
            | ExpressionKind::Float(_)
            | ExpressionKind::Symbol(_)
            | ExpressionKind::StorageSize(_)
            | ExpressionKind::Unit => {}
        }
    }

    fn collect_checked_body(&mut self, items: &[checked::BodyItem], result: &checked::Expression) {
        for item in items {
            match item {
                checked::BodyItem::Binding(binding) => self.collect_checked_binding(binding),
                checked::BodyItem::Expression(expression) => {
                    self.collect_checked_expression(expression);
                }
            }
        }
        self.collect_checked_expression(result);
    }
}
