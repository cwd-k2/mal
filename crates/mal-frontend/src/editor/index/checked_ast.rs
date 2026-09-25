use crate::check::ast as checked;
use crate::resolve::ast as resolved;
use mal_syntax::source::Span;

use super::{Choice, Index, merge_targets};
use crate::editor::Exit;

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
                self.collect_checked_body(&block.items, &block.result, false);
            }
            ExpressionKind::ResultBlock {
                target,
                result_binders,
                body,
            } => {
                for binder in result_binders {
                    let id = self.canonical_value(binder.binding.id);
                    let parameter_type = crate::check::type_name(&binder.parameter_type);
                    self.result_binders.insert(id);
                    self.result_binder_names.insert(
                        (self.canonical_value(*target), binder.variant),
                        binder.binding.name.text.clone(),
                    );
                    self.value_types.insert(id, parameter_type.clone());
                    self.typed_regions
                        .push((binder.binding.name.span, parameter_type));
                }
                self.collect_checked_body(&body.items, &body.result, false);
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
                self.collect_checked_body(&lambda.body.items, &lambda.body.result, false);
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
                let choices = continuations
                    .iter()
                    .map(|continuation| match continuation {
                        checked::SumContinuation::Function(function) => Choice {
                            span: function.span,
                            leaves: false,
                            targets: Vec::new(),
                        },
                        checked::SumContinuation::Branch(branch) => {
                            self.choice(branch.span, branch.body.result.as_ref())
                        }
                        checked::SumContinuation::Transfer(transfer) => Choice {
                            span: transfer.span,
                            leaves: true,
                            targets: self.transfer_names(transfer.target, transfer.variant),
                        },
                    })
                    .collect::<Vec<_>>();
                let reported = self.note_exits(&choices, expression.span);
                for (continuation, tail_reported) in continuations.iter().zip(reported) {
                    self.collect_checked_continuation(continuation, tail_reported);
                }
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
                let [then_reported, else_reported] =
                    self.note_branch_exits(then_branch, else_branch, expression.span);
                self.collect_checked_body(&then_branch.items, &then_branch.result, then_reported);
                self.collect_checked_body(&else_branch.items, &else_branch.result, else_reported);
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

    /// One side of a choice: whether every path through it leaves the block, and to which binders.
    fn choice(&self, span: Span, completion: &checked::Completion) -> Choice {
        Choice {
            span,
            leaves: matches!(completion, checked::Completion::Abrupt(_)),
            targets: self.completion_targets(completion),
        }
    }

    fn transfer_names(&self, target: resolved::ValueId, variant: Option<usize>) -> Vec<String> {
        self.result_binder_names
            .get(&(self.canonical_value(target), variant))
            .cloned()
            .into_iter()
            .collect()
    }

    /// The result binders a path may transfer to, in source order. An empty elimination names none.
    fn completion_targets(&self, completion: &checked::Completion) -> Vec<String> {
        match completion {
            checked::Completion::Value(_) => Vec::new(),
            checked::Completion::Abrupt(abrupt) => self.abrupt_targets(abrupt),
        }
    }

    fn abrupt_targets(&self, abrupt: &checked::AbruptExpression) -> Vec<String> {
        let mut targets = Vec::new();
        match &abrupt.kind {
            checked::AbruptExpressionKind::ResultTransfer {
                target, variant, ..
            } => {
                merge_targets(&mut targets, self.transfer_names(*target, *variant));
            }
            checked::AbruptExpressionKind::EmptyElimination { .. } => {}
            checked::AbruptExpressionKind::If {
                then_branch,
                else_branch,
                ..
            } => {
                merge_targets(&mut targets, self.completion_targets(&then_branch.result));
                merge_targets(&mut targets, self.completion_targets(&else_branch.result));
            }
            checked::AbruptExpressionKind::SumElimination { continuations, .. } => {
                for continuation in continuations {
                    let names = match continuation {
                        checked::SumContinuation::Function(_) => Vec::new(),
                        checked::SumContinuation::Branch(branch) => {
                            self.completion_targets(&branch.body.result)
                        }
                        checked::SumContinuation::Transfer(transfer) => {
                            self.transfer_names(transfer.target, transfer.variant)
                        }
                    };
                    merge_targets(&mut targets, names);
                }
            }
            checked::AbruptExpressionKind::Block(block) => {
                merge_targets(&mut targets, self.completion_targets(&block.result));
            }
        }
        targets
    }

    /// Records where a choice leaves the block. When some choices continue, the leaving ones are marked; when all of
    /// them leave, the whole choice is. Returns, for each side, whether a mark already covers where that side ends.
    fn note_exits(&mut self, choices: &[Choice], whole: Span) -> Vec<bool> {
        let leaving = choices.iter().filter(|choice| choice.leaves).count();
        if leaving == 0 {
            return vec![false; choices.len()];
        }
        if leaving == choices.len() {
            let mut targets = Vec::new();
            for choice in choices {
                merge_targets(&mut targets, choice.targets.clone());
            }
            self.exits.push(Exit {
                span: whole,
                targets,
            });
        } else {
            self.exits.extend(
                choices
                    .iter()
                    .filter(|choice| choice.leaves)
                    .map(|choice| Exit {
                        span: choice.span,
                        targets: choice.targets.clone(),
                    }),
            );
        }
        choices.iter().map(|choice| choice.leaves).collect()
    }

    fn note_branch_exits(
        &mut self,
        then_branch: &checked::ExpressionBlock,
        else_branch: &checked::ExpressionBlock,
        whole: Span,
    ) -> [bool; 2] {
        let then_choice = self.choice(then_branch.span, then_branch.result.as_ref());
        let mut else_choice = self.choice(else_branch.span, else_branch.result.as_ref());
        // `when` has no written else branch; its synthetic one spans the whole expression.
        else_choice.leaves &= else_branch.span != whole;
        let reported = self.note_exits(&[then_choice, else_choice], whole);
        [reported[0], reported[1]]
    }

    fn collect_checked_continuation(
        &mut self,
        continuation: &checked::SumContinuation,
        tail_reported: bool,
    ) {
        match continuation {
            checked::SumContinuation::Function(expression) => {
                self.collect_checked_expression(expression);
            }
            checked::SumContinuation::Branch(branch) => {
                if let Some(parameter) = &branch.parameter {
                    self.collect_checked_pattern(parameter);
                    self.mark_parameter_bindings(parameter);
                }
                self.collect_checked_body(&branch.body.items, &branch.body.result, tail_reported);
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

    /// `tail_reported` is whether a hint already marks the unit whose end this body's completion is. The statements
    /// before the completion start fresh paths: an exit among them is a place where control may leave early, which
    /// stays visible however the unit ends.
    fn collect_checked_body(
        &mut self,
        items: &[checked::BodyItem],
        result: &checked::Completion,
        tail_reported: bool,
    ) {
        for item in items {
            match item {
                checked::BodyItem::Binding(binding) => self.collect_checked_binding(binding),
                checked::BodyItem::Expression(expression) => {
                    self.collect_checked_expression(expression);
                }
            }
        }
        self.collect_checked_completion(result, tail_reported);
    }

    /// A choice that ends its block on every side is marked, unless it is the end of a unit that is already marked.
    fn note_abrupt_exit(&mut self, abrupt: &checked::AbruptExpression, tail_reported: bool) {
        if !tail_reported {
            self.exits.push(Exit {
                span: abrupt.span,
                targets: self.abrupt_targets(abrupt),
            });
        }
    }

    fn collect_checked_completion(
        &mut self,
        completion: &checked::Completion,
        tail_reported: bool,
    ) {
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
                        self.note_abrupt_exit(abrupt, tail_reported);
                        // Every side leaves, so each ends in the unit this choice marks or that marks it.
                        self.collect_checked_body(&then_branch.items, &then_branch.result, true);
                        self.collect_checked_body(&else_branch.items, &else_branch.result, true);
                    }
                    checked::AbruptExpressionKind::SumElimination {
                        scrutinee,
                        continuations,
                    } => {
                        self.collect_checked_expression(scrutinee);
                        self.note_abrupt_exit(abrupt, tail_reported);
                        for continuation in continuations {
                            self.collect_checked_continuation(continuation, true);
                        }
                    }
                    checked::AbruptExpressionKind::Block(block) => {
                        self.collect_checked_body(&block.items, &block.result, tail_reported);
                    }
                }
            }
        }
    }
}
