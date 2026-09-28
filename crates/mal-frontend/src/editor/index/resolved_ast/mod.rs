mod type_pattern;

use crate::resolve::ast as resolved;

use super::Index;
use crate::editor::{OccurrenceRole, SymbolId};

impl Index {
    pub(super) fn collect_resolved_top(&mut self, item: &mal_syntax::ast::Node<resolved::TopItem>) {
        match &item.kind {
            resolved::TopItem::TypeAlias { binding, value } => {
                let id = SymbolId::Type(binding.id);
                self.type_details
                    .insert(binding.id, super::type_display::type_name(value));
                self.top_level.push(id);
                self.add_raw(
                    id,
                    &binding.name,
                    OccurrenceRole::Declaration,
                    Some(item.span),
                );
                self.collect_resolved_type(value);
            }
            resolved::TopItem::GenericTypeAlias {
                binding,
                parameters,
                value,
            } => {
                let id = SymbolId::Type(binding.id);
                self.type_details
                    .insert(binding.id, super::type_display::type_name(value));
                self.top_level.push(id);
                self.add_raw(
                    id,
                    &binding.name,
                    OccurrenceRole::Declaration,
                    Some(item.span),
                );
                for parameter in parameters {
                    self.add_raw(
                        SymbolId::Type(parameter.id),
                        &parameter.name,
                        OccurrenceRole::Declaration,
                        None,
                    );
                }
                self.collect_resolved_type(value);
            }
            resolved::TopItem::ExternalType { binding } => {
                let id = SymbolId::Type(binding.id);
                self.top_level.push(id);
                self.add_raw(
                    id,
                    &binding.name,
                    OccurrenceRole::Declaration,
                    Some(item.span),
                );
            }
            resolved::TopItem::ExternalOperation { binding, ty, .. } => {
                self.value_types
                    .insert(binding.id, super::type_display::type_name(ty));
                self.functions.insert(binding.id);
                let id = SymbolId::Value(binding.id);
                self.top_level.push(id);
                self.add_raw(
                    id,
                    &binding.name,
                    OccurrenceRole::Declaration,
                    Some(item.span),
                );
                self.collect_resolved_type(ty);
            }
            resolved::TopItem::Binding(binding) => {
                self.collect_resolved_binding(binding, true, item.span);
            }
            resolved::TopItem::GenericBinding {
                binding,
                parameters,
                annotation,
                value,
            } => {
                let id = SymbolId::Value(binding.id);
                self.value_types
                    .insert(binding.id, super::type_display::type_name(annotation));
                self.top_level.push(id);
                self.add_raw(
                    id,
                    &binding.name,
                    OccurrenceRole::Declaration,
                    Some(item.span),
                );
                for parameter in parameters {
                    self.add_raw(
                        SymbolId::Type(parameter.id),
                        &parameter.name,
                        OccurrenceRole::Declaration,
                        None,
                    );
                }
                self.collect_resolved_type(annotation);
                self.collect_resolved_expression_with_expected(value, Some(annotation));
            }
            resolved::TopItem::OperationFamily {
                binding,
                parameters,
                annotation,
            } => {
                let id = SymbolId::Value(binding.id);
                self.value_types
                    .insert(binding.id, super::type_display::type_name(annotation));
                self.top_level.push(id);
                self.add_raw(
                    id,
                    &binding.name,
                    OccurrenceRole::Declaration,
                    Some(item.span),
                );
                for parameter in parameters {
                    self.add_raw(
                        SymbolId::Type(parameter.id),
                        &parameter.name,
                        OccurrenceRole::Declaration,
                        None,
                    );
                }
                self.collect_resolved_type(annotation);
            }
            resolved::TopItem::OperationImplementation {
                family,
                arguments,
                annotation,
                value,
            } => {
                self.add_raw(
                    SymbolId::Value(family.id),
                    &family.name,
                    OccurrenceRole::Reference,
                    None,
                );
                for argument in arguments {
                    self.collect_resolved_type(argument);
                }
                self.collect_resolved_type(annotation);
                self.collect_resolved_expression_with_expected(value, Some(annotation));
            }
        }
    }

    fn collect_resolved_binding(
        &mut self,
        binding: &resolved::Binding,
        top_level: bool,
        declaration_span: mal_syntax::source::Span,
    ) {
        if let Some(annotation) = &binding.annotation {
            self.collect_resolved_type(annotation);
            self.apply_declared_pattern_type(&binding.pattern, annotation);
        } else if let Some(inferred) = self.expression_display_type(&binding.value) {
            self.apply_declared_pattern_type(&binding.pattern, &inferred);
        } else if let resolved::Pattern::Binding(name) = &binding.pattern.kind
            && let Some(inferred) = self.inferred_expression_display_name(&binding.value)
        {
            self.value_types
                .insert(self.canonical_value(name.id), inferred);
        }
        self.collect_resolved_expression_with_expected(&binding.value, binding.annotation.as_ref());
        self.collect_resolved_pattern(&binding.pattern, top_level, declaration_span);
    }

    fn collect_resolved_expression_with_expected(
        &mut self,
        expression: &mal_syntax::ast::Node<resolved::Expression>,
        expected: Option<&mal_syntax::ast::Node<resolved::TypeExpression>>,
    ) {
        match (&expression.kind, expected) {
            (resolved::Expression::Parenthesized(inner), Some(expected)) => {
                self.collect_resolved_expression_with_expected(inner, Some(expected));
            }
            (resolved::Expression::Lambda(lambda), Some(expected)) => {
                let expanded = self.expanded_type(expected);
                let resolved::TypeExpression::Function { parameter, result } = expanded.kind else {
                    self.collect_resolved_expression(expression);
                    return;
                };
                self.collect_resolved_lambda(lambda, Some(&parameter), Some(&result));
            }
            (resolved::Expression::Block(block), Some(expected)) => {
                for item in &block.items {
                    match item {
                        resolved::BodyItem::Binding(binding) => {
                            self.collect_resolved_binding(&binding.kind, false, binding.span);
                        }
                        resolved::BodyItem::Expression(expression) => {
                            self.collect_resolved_expression(expression);
                        }
                    }
                }
                self.collect_resolved_expression_with_expected(&block.result, Some(expected));
            }
            (
                resolved::Expression::ResultBlock {
                    result_binders,
                    body,
                },
                Some(expected),
            ) => {
                self.collect_resolved_result_binders(result_binders, Some(expected));
                for item in &body.items {
                    match item {
                        resolved::BodyItem::Binding(binding) => {
                            self.collect_resolved_binding(&binding.kind, false, binding.span);
                        }
                        resolved::BodyItem::Expression(expression) => {
                            self.collect_resolved_expression(expression);
                        }
                    }
                }
                self.collect_resolved_expression_with_expected(&body.result, Some(expected));
            }
            _ => self.collect_resolved_expression(expression),
        }
    }

    fn collect_resolved_result_binders(
        &mut self,
        result_binders: &[resolved::ValueBinding],
        result_type: Option<&mal_syntax::ast::Node<resolved::TypeExpression>>,
    ) {
        match result_binders {
            [binding] => {
                if let Some(result_type) = result_type {
                    self.value_types.insert(
                        self.canonical_value(binding.id),
                        super::type_display::type_name(result_type),
                    );
                }
            }
            bindings => {
                if let Some(result_type) = result_type
                    && let resolved::TypeExpression::Sum(members) =
                        self.expanded_type(result_type).kind
                {
                    for (binding, member) in bindings.iter().zip(&members) {
                        self.value_types.insert(
                            self.canonical_value(binding.id),
                            super::type_display::type_name(member),
                        );
                    }
                }
            }
        }
        for binding in result_binders {
            let id = SymbolId::Value(self.canonical_value(binding.id));
            self.add_raw(
                id,
                &binding.name,
                OccurrenceRole::Declaration,
                Some(binding.name.span),
            );
        }
    }

    fn collect_resolved_lambda(
        &mut self,
        lambda: &resolved::Lambda,
        parameter_type: Option<&mal_syntax::ast::Node<resolved::TypeExpression>>,
        result_type: Option<&mal_syntax::ast::Node<resolved::TypeExpression>>,
    ) {
        if let Some(parameter) = &lambda.parameter {
            if let Some(parameter_type) = parameter_type {
                self.apply_declared_pattern_type(parameter, parameter_type);
            }
            self.collect_resolved_pattern(parameter, false, parameter.span);
        }
        for item in &lambda.body.items {
            match item {
                resolved::BodyItem::Binding(binding) => {
                    self.collect_resolved_binding(&binding.kind, false, binding.span);
                }
                resolved::BodyItem::Expression(expression) => {
                    self.collect_resolved_expression(expression);
                }
            }
        }
        self.collect_resolved_expression_with_expected(&lambda.body.result, result_type);
    }

    fn collect_resolved_expression(
        &mut self,
        expression: &mal_syntax::ast::Node<resolved::Expression>,
    ) {
        use resolved::Expression;
        match &expression.kind {
            Expression::Reference(reference) => self.add_raw(
                SymbolId::Value(self.canonical_value(reference.id)),
                &reference.name,
                OccurrenceRole::Reference,
                None,
            ),
            Expression::GenericReference {
                reference,
                arguments,
            } => {
                self.add_raw(
                    SymbolId::Value(self.canonical_value(reference.id)),
                    &reference.name,
                    OccurrenceRole::Reference,
                    None,
                );
                for argument in arguments {
                    self.collect_resolved_type(argument);
                }
            }
            Expression::Parenthesized(inner) => self.collect_resolved_expression(inner),
            Expression::Product(elements) => {
                for element in elements {
                    self.collect_resolved_expression(element);
                }
            }
            Expression::Block(block) => {
                self.collect_resolved_body(&block.items, &block.result);
            }
            Expression::ResultBlock {
                result_binders,
                body,
            } => {
                self.collect_resolved_result_binders(result_binders, None);
                self.collect_resolved_body(&body.items, &body.result);
            }
            Expression::Lambda(lambda) => {
                self.collect_resolved_lambda(lambda, None, None);
            }
            Expression::Call { callee, arguments } => self.collect_resolved_call(callee, arguments),
            Expression::ContinuationApplication {
                value,
                continuations,
            } => {
                self.collect_resolved_expression(value);
                let members = self
                    .expression_display_type(value)
                    .map(|ty| self.expanded_type(&ty))
                    .and_then(|ty| match ty.kind {
                        resolved::TypeExpression::Sum(members) => Some(members),
                        _ => None,
                    });
                for (index, continuation) in continuations.iter().enumerate() {
                    self.collect_resolved_continuation(
                        continuation,
                        members.as_ref().and_then(|members| members.get(index)),
                    );
                }
            }
            Expression::Conversion { value, .. } => self.collect_resolved_expression(value),
            Expression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.collect_resolved_expression(condition);
                self.collect_resolved_body(&then_branch.items, &then_branch.result);
                self.collect_resolved_body(&else_branch.items, &else_branch.result);
            }
            Expression::When { condition, body } => {
                self.collect_resolved_expression(condition);
                self.collect_resolved_body(&body.items, &body.result);
            }
            Expression::Unary { operand, .. } => self.collect_resolved_expression(operand),
            Expression::Binary { left, right, .. } => {
                let mut pending = vec![right.as_ref(), left.as_ref()];
                while let Some(expression) = pending.pop() {
                    if let Expression::Binary { left, right, .. } = &expression.kind {
                        pending.push(right);
                        pending.push(left);
                    } else {
                        self.collect_resolved_expression(expression);
                    }
                }
            }
            Expression::Integer(_)
            | Expression::Float(_)
            | Expression::Byte(_)
            | Expression::Symbol(_)
            | Expression::Unit => {}
        }
    }

    fn collect_resolved_continuation(
        &mut self,
        continuation: &resolved::Continuation,
        parameter_type: Option<&mal_syntax::ast::Node<resolved::TypeExpression>>,
    ) {
        match continuation {
            resolved::Continuation::Function(expression) => {
                if let (resolved::Expression::Lambda(lambda), Some(parameter_type)) =
                    (&expression.kind, parameter_type)
                {
                    self.collect_resolved_lambda(lambda, Some(parameter_type), None);
                } else {
                    self.collect_resolved_expression(expression);
                }
            }
            resolved::Continuation::Branch(branch) => {
                if let Some(parameter) = &branch.parameter {
                    if let Some(parameter_type) = parameter_type {
                        self.apply_declared_pattern_type(parameter, parameter_type);
                    }
                    self.collect_resolved_pattern(parameter, false, parameter.span);
                }
                self.collect_resolved_body(&branch.body.items, &branch.body.result);
            }
        }
    }

    fn collect_resolved_call(
        &mut self,
        callee: &mal_syntax::ast::Node<resolved::Expression>,
        arguments: &[mal_syntax::ast::Node<resolved::Expression>],
    ) {
        let parameter = self.expression_display_type(callee).and_then(|signature| {
            match self.expanded_type(&signature).kind {
                resolved::TypeExpression::Function { parameter, .. } => Some(*parameter),
                _ => None,
            }
        });
        self.collect_resolved_expression(callee);
        match (arguments, parameter) {
            ([argument], Some(parameter)) => {
                self.collect_resolved_expression_with_expected(argument, Some(&parameter));
            }
            (arguments, Some(parameter)) => {
                let expanded = self.expanded_type(&parameter);
                if let resolved::TypeExpression::Product(parameters) = expanded.kind
                    && parameters.len() == arguments.len()
                {
                    for (argument, parameter) in arguments.iter().zip(&parameters) {
                        self.collect_resolved_expression_with_expected(argument, Some(parameter));
                    }
                } else {
                    for argument in arguments {
                        self.collect_resolved_expression(argument);
                    }
                }
            }
            (arguments, None) => {
                for argument in arguments {
                    self.collect_resolved_expression(argument);
                }
            }
        }
    }

    fn collect_resolved_body(
        &mut self,
        items: &[resolved::BodyItem],
        result: &mal_syntax::ast::Node<resolved::Expression>,
    ) {
        for item in items {
            match item {
                resolved::BodyItem::Binding(binding) => {
                    self.collect_resolved_binding(&binding.kind, false, binding.span);
                }
                resolved::BodyItem::Expression(expression) => {
                    self.collect_resolved_expression(expression);
                }
            }
        }
        self.collect_resolved_expression(result);
    }
}
