use crate::check::ast as checked;

use super::Index;

impl Index {
    pub(super) fn collect_checked_top(&mut self, item: &checked::TopItem) {
        match item {
            checked::TopItem::TypeAlias { binding, ty, .. } => {
                self.type_details
                    .insert(binding.id, crate::check::type_name(ty));
            }
            checked::TopItem::ExternalType { binding } => {
                self.type_details
                    .insert(binding.id, binding.name.text.clone());
            }
            checked::TopItem::ExternalOperation { .. } => {}
            checked::TopItem::GenericBinding(binding) => {
                let id = self.canonical_value(binding.binding.id);
                if matches!(binding.ty, checked::Type::Function { .. }) {
                    self.functions.insert(id);
                }
                let ty = crate::check::type_name(&binding.ty);
                self.value_types.insert(id, ty.clone());
                self.typed_regions.push((binding.binding.name.span, ty));
                self.collect_checked_expression(&binding.value);
            }
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
                if matches!(ty, checked::Type::Function { .. }) {
                    self.functions.insert(id);
                }
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
            ExpressionKind::GenericReference { reference, .. } => {
                self.value_types
                    .entry(self.canonical_value(reference.id))
                    .or_insert(ty);
            }
            ExpressionKind::Product(elements) => {
                for element in elements {
                    self.collect_checked_expression(element);
                }
            }
            ExpressionKind::Parenthesized(inner) => self.collect_checked_expression(inner),
            ExpressionKind::Block(block) => {
                self.collect_checked_body(&block.items, &block.result);
            }
            ExpressionKind::ResultBlock {
                result_binders,
                body,
                ..
            } => {
                for binder in result_binders {
                    let id = self.canonical_value(binder.binding.id);
                    let parameter_type = crate::check::type_name(&binder.parameter_type);
                    self.result_binders.insert(id);
                    self.value_types.insert(id, parameter_type.clone());
                    self.typed_regions
                        .push((binder.binding.name.span, parameter_type));
                }
                self.collect_checked_body(&body.items, &body.result);
            }
            ExpressionKind::Lambda(lambda) => {
                for capture in &lambda.captures {
                    self.value_types.insert(
                        self.canonical_value(capture.binding.id),
                        crate::check::type_name(&capture.ty),
                    );
                }
                if let Some(parameter) = &lambda.parameter {
                    self.collect_checked_pattern(parameter);
                    self.mark_parameter_bindings(parameter);
                }
                self.collect_checked_body(&lambda.body.items, &lambda.body.result);
            }
            ExpressionKind::Call { callee, argument } => {
                self.collect_checked_expression(callee);
                self.collect_checked_expression(argument);
            }
            ExpressionKind::SumElimination {
                scrutinee,
                continuations,
            } => {
                self.collect_checked_expression(scrutinee);
                for continuation in continuations {
                    self.collect_checked_continuation(continuation);
                }
                let choices = continuations
                    .iter()
                    .map(|continuation| match continuation {
                        checked::SumContinuation::Function(function) => (function.span, false),
                        checked::SumContinuation::Branch(branch) => (
                            branch.span,
                            matches!(branch.body.result.as_ref(), checked::Completion::Abrupt(_)),
                        ),
                        checked::SumContinuation::Transfer(transfer) => (transfer.span, true),
                    })
                    .collect::<Vec<_>>();
                self.note_exits(&choices, expression.span);
            }
            ExpressionKind::SymbolLength { value }
            | ExpressionKind::NumericConversion { value }
            | ExpressionKind::SumInjection { value, .. } => self.collect_checked_expression(value),
            ExpressionKind::Memory { operands, .. } => {
                for operand in operands {
                    self.collect_checked_expression(operand);
                }
            }
            ExpressionKind::SymbolAt { argument } => self.collect_checked_expression(argument),
            ExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.collect_checked_expression(condition);
                self.collect_checked_body(&then_branch.items, &then_branch.result);
                self.collect_checked_body(&else_branch.items, &else_branch.result);
                self.note_branch_exits(then_branch, else_branch, expression.span);
            }
            ExpressionKind::Unary { operand, .. } => self.collect_checked_expression(operand),
            ExpressionKind::Binary { left, right, .. } => {
                let mut pending = vec![right.as_ref(), left.as_ref()];
                while let Some(expression) = pending.pop() {
                    if let ExpressionKind::Binary { left, right, .. } = &expression.kind {
                        pending.push(right);
                        pending.push(left);
                    } else {
                        self.collect_checked_expression(expression);
                    }
                }
            }
            ExpressionKind::Integer(_)
            | ExpressionKind::Float(_)
            | ExpressionKind::Symbol(_)
            | ExpressionKind::Unit => {}
        }
    }

    /// Records where a choice leaves the block. When some choices continue, the leaving ones are marked;
    /// when all of them leave, the whole choice is.
    fn note_exits(
        &mut self,
        choices: &[(mal_syntax::source::Span, bool)],
        whole: mal_syntax::source::Span,
    ) {
        let leaving = choices.iter().filter(|(_, leaves)| *leaves).count();
        if leaving == 0 {
            return;
        }
        if leaving == choices.len() {
            self.exits.push(whole);
        } else {
            self.exits.extend(
                choices
                    .iter()
                    .filter(|(_, leaves)| *leaves)
                    .map(|(span, _)| *span),
            );
        }
    }

    fn note_branch_exits(
        &mut self,
        then_branch: &checked::ExpressionBlock,
        else_branch: &checked::ExpressionBlock,
        whole: mal_syntax::source::Span,
    ) {
        let leaves = |branch: &checked::ExpressionBlock| {
            matches!(branch.result.as_ref(), checked::Completion::Abrupt(_))
        };
        // `when` has no written else branch; its synthetic one spans the whole expression.
        let choices = [
            (then_branch.span, leaves(then_branch)),
            (
                else_branch.span,
                leaves(else_branch) && else_branch.span != whole,
            ),
        ];
        self.note_exits(&choices, whole);
    }

    fn collect_checked_continuation(&mut self, continuation: &checked::SumContinuation) {
        match continuation {
            checked::SumContinuation::Function(expression) => {
                self.collect_checked_expression(expression);
            }
            checked::SumContinuation::Branch(branch) => {
                if let Some(parameter) = &branch.parameter {
                    self.collect_checked_pattern(parameter);
                    self.mark_parameter_bindings(parameter);
                }
                self.collect_checked_body(&branch.body.items, &branch.body.result);
            }
            checked::SumContinuation::Transfer(_) => {}
        }
    }

    fn mark_parameter_bindings(&mut self, pattern: &checked::Pattern) {
        match pattern {
            checked::Pattern::Binding { binding, .. } => {
                let id = self.canonical_value(binding.id);
                self.parameters.insert(id);
            }
            checked::Pattern::Product { elements, .. } => {
                for element in elements {
                    self.mark_parameter_bindings(element);
                }
            }
            checked::Pattern::Wildcard { .. } => {}
        }
    }

    fn collect_checked_body(&mut self, items: &[checked::BodyItem], result: &checked::Completion) {
        for item in items {
            match item {
                checked::BodyItem::Binding(binding) => self.collect_checked_binding(binding),
                checked::BodyItem::Expression(expression) => {
                    self.collect_checked_expression(expression);
                }
            }
        }
        self.collect_checked_completion(result);
    }

    fn collect_checked_completion(&mut self, completion: &checked::Completion) {
        match completion {
            checked::Completion::Value(expression) => self.collect_checked_expression(expression),
            checked::Completion::Abrupt(abrupt) => {
                for expression in &abrupt.preceding {
                    self.collect_checked_expression(expression);
                }
                match &abrupt.kind {
                    checked::AbruptExpressionKind::ResultTransfer { value, .. } => {
                        self.collect_checked_expression(value);
                    }
                    checked::AbruptExpressionKind::EmptyElimination { scrutinee } => {
                        self.collect_checked_expression(scrutinee);
                    }
                    checked::AbruptExpressionKind::If {
                        condition,
                        then_branch,
                        else_branch,
                    } => {
                        self.collect_checked_expression(condition);
                        self.collect_checked_body(&then_branch.items, &then_branch.result);
                        self.collect_checked_body(&else_branch.items, &else_branch.result);
                        self.exits.push(abrupt.span);
                    }
                    checked::AbruptExpressionKind::SumElimination {
                        scrutinee,
                        continuations,
                    } => {
                        self.collect_checked_expression(scrutinee);
                        for continuation in continuations {
                            self.collect_checked_continuation(continuation);
                        }
                        self.exits.push(abrupt.span);
                    }
                    checked::AbruptExpressionKind::Block(block) => {
                        self.collect_checked_body(&block.items, &block.result);
                    }
                }
            }
        }
    }
}
