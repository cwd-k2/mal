use std::collections::HashSet;

use crate::ast;
use crate::diagnostic::Diagnostic;

use super::Resolver;
use super::ast::{
    BodyItem, Capture, CaseArm, Expression, ExpressionBlock, ExternalOperationReference, Lambda,
    LambdaBody, Parameter, ValueOwner, ValueReference,
};

impl Resolver {
    pub(super) fn resolve_expression(
        &mut self,
        expression: &ast::Node<ast::Expression>,
    ) -> Result<ast::Node<Expression>, Diagnostic> {
        let kind = match &expression.kind {
            ast::Expression::Name(name) => {
                Expression::Reference(self.resolve_value_reference(name)?)
            }
            ast::Expression::Integer(value) => Expression::Integer(value.clone()),
            ast::Expression::Float(value) => Expression::Float(value.clone()),
            ast::Expression::Byte(value) => Expression::Byte(*value),
            ast::Expression::String(value) => Expression::String(value.clone()),
            ast::Expression::Unit => Expression::Unit,
            ast::Expression::Parenthesized(inner) => {
                Expression::Parenthesized(Box::new(self.resolve_expression(inner)?))
            }
            ast::Expression::Product(elements) => Expression::Product(
                elements
                    .iter()
                    .map(|element| self.resolve_expression(element))
                    .collect::<Result<_, _>>()?,
            ),
            ast::Expression::Lambda(lambda) => Expression::Lambda(self.resolve_lambda(lambda)?),
            ast::Expression::Call { callee, arguments } => Expression::Call {
                callee: Box::new(self.resolve_expression(callee)?),
                arguments: arguments
                    .iter()
                    .map(|argument| self.resolve_expression(argument))
                    .collect::<Result<_, _>>()?,
            },
            ast::Expression::ExternalCall { name, arguments } => Expression::ExternalCall {
                operation: self.resolve_external_reference(name)?,
                arguments: arguments
                    .iter()
                    .map(|argument| self.resolve_expression(argument))
                    .collect::<Result<_, _>>()?,
            },
            ast::Expression::Conversion { type_name, value } => Expression::Conversion {
                type_ref: self.type_reference(type_name)?,
                value: Box::new(self.resolve_expression(value)?),
            },
            ast::Expression::SumInjection {
                type_name,
                index,
                value,
            } => Expression::SumInjection {
                type_ref: self.type_reference(type_name)?,
                index: index.clone(),
                value: Box::new(self.resolve_expression(value)?),
            },
            ast::Expression::If {
                condition,
                then_branch,
                else_branch,
            } => Expression::If {
                condition: Box::new(self.resolve_expression(condition)?),
                then_branch: self.resolve_expression_block(then_branch)?,
                else_branch: self.resolve_expression_block(else_branch)?,
            },
            ast::Expression::Case { scrutinee, arms } => Expression::Case {
                scrutinee: Box::new(self.resolve_expression(scrutinee)?),
                arms: arms
                    .iter()
                    .map(|arm| self.resolve_case_arm(arm))
                    .collect::<Result<_, _>>()?,
            },
            ast::Expression::Unary { operator, operand } => Expression::Unary {
                operator: operator.clone(),
                operand: Box::new(self.resolve_expression(operand)?),
            },
            ast::Expression::Binary {
                operator,
                left,
                right,
            } => Expression::Binary {
                operator: operator.clone(),
                left: Box::new(self.resolve_expression(left)?),
                right: Box::new(self.resolve_expression(right)?),
            },
        };
        Ok(ast::Node::new(kind, expression.span))
    }

    fn resolve_lambda(&mut self, lambda: &ast::Lambda) -> Result<Lambda, Diagnostic> {
        self.resolve_lambda_with_self(lambda, None)
    }

    pub(super) fn resolve_lambda_with_self(
        &mut self,
        lambda: &ast::Lambda,
        self_binding: Option<super::ast::ValueBinding>,
    ) -> Result<Lambda, Diagnostic> {
        let mut seen = HashSet::new();
        let mut sources = Vec::with_capacity(lambda.captures.len());
        for capture in &lambda.captures {
            if !seen.insert(capture.text.as_str()) {
                return Err(
                    Diagnostic::error(format!("duplicate capture `{}`", capture.text))
                        .with_primary(capture.span, "already listed in this capture list"),
                );
            }
            let source = self
                .lookup_value(&capture.text)
                .ok_or_else(|| self.unknown(capture, "captured value"))?;
            match source.owner {
                ValueOwner::Lambda(owner) if Some(owner) == self.current_lambda => {}
                ValueOwner::Lambda(_) => {
                    return Err(Diagnostic::error(format!(
                        "capture `{}` crosses a lambda boundary",
                        capture.text
                    ))
                    .with_primary(capture.span, "capture it in each enclosing lambda first"));
                }
                ValueOwner::Predefined | ValueOwner::TopLevel => {
                    return Err(Diagnostic::error(format!(
                        "cannot capture non-local value `{}`",
                        capture.text
                    ))
                    .with_primary(
                        capture.span,
                        "top-level and predefined values are referenced directly",
                    ));
                }
            }
            sources.push((capture, source));
        }

        let id = self.allocate_lambda();
        let outer_lambda = self.current_lambda;
        let outer_recursive_lambda = self.recursive_lambda;
        self.current_lambda = Some(id);
        self.recursive_lambda = self_binding.as_ref().map(|binding| (id, binding.id));
        self.push_scope();
        let result = (|| {
            let mut captures = Vec::with_capacity(sources.len());
            for (name, source) in sources {
                let binding = self.declare_value(name, ValueOwner::Lambda(id))?;
                captures.push(Capture {
                    source: ValueReference {
                        id: source.id,
                        name: name.clone(),
                    },
                    binding,
                });
            }

            let mut parameters = Vec::with_capacity(lambda.parameters.len());
            for parameter in &lambda.parameters {
                let ty = self.resolve_type(&parameter.ty)?;
                let binding = self.declare_value(&parameter.name, ValueOwner::Lambda(id))?;
                parameters.push(Parameter {
                    binding,
                    ty,
                    span: parameter.span,
                });
            }
            let body = self.resolve_lambda_body(&lambda.body)?;
            Ok(Lambda {
                id,
                self_binding: self_binding.map(|binding| binding.id),
                captures,
                parameters,
                body,
            })
        })();
        self.pop_scope();
        self.current_lambda = outer_lambda;
        self.recursive_lambda = outer_recursive_lambda;
        result
    }

    fn resolve_lambda_body(&mut self, body: &ast::LambdaBody) -> Result<LambdaBody, Diagnostic> {
        let mut items = Vec::with_capacity(body.items.len());
        for item in &body.items {
            items.push(self.resolve_body_item(item)?);
        }
        Ok(LambdaBody {
            items,
            result: Box::new(self.resolve_expression(&body.result)?),
            span: body.span,
        })
    }

    fn resolve_body_item(&mut self, item: &ast::BodyItem) -> Result<BodyItem, Diagnostic> {
        match item {
            ast::BodyItem::Binding(binding) => {
                let owner = self.local_owner(binding.span)?;
                Ok(BodyItem::Binding(ast::Node::new(
                    self.resolve_binding(&binding.kind, owner)?,
                    binding.span,
                )))
            }
            ast::BodyItem::Expression(expression) => {
                Ok(BodyItem::Expression(self.resolve_expression(expression)?))
            }
        }
    }

    fn resolve_expression_block(
        &mut self,
        block: &ast::ExpressionBlock,
    ) -> Result<ExpressionBlock, Diagnostic> {
        self.push_scope();
        let result = (|| {
            let mut items = Vec::with_capacity(block.items.len());
            for item in &block.items {
                items.push(self.resolve_body_item(item)?);
            }
            Ok(ExpressionBlock {
                items,
                result: Box::new(self.resolve_expression(&block.result)?),
                span: block.span,
            })
        })();
        self.pop_scope();
        result
    }

    fn resolve_case_arm(&mut self, arm: &ast::CaseArm) -> Result<CaseArm, Diagnostic> {
        self.push_scope();
        let result = (|| {
            let owner = self.local_owner(arm.pattern.span)?;
            Ok(CaseArm {
                index: arm.index.clone(),
                pattern: self.declare_pattern(&arm.pattern, owner)?,
                value: self.resolve_expression(&arm.value)?,
                span: arm.span,
            })
        })();
        self.pop_scope();
        result
    }

    fn resolve_value_reference(&self, name: &ast::Name) -> Result<ValueReference, Diagnostic> {
        let binding = self
            .lookup_value(&name.text)
            .ok_or_else(|| self.unknown(name, "value"))?;
        if let ValueOwner::Lambda(owner) = binding.owner
            && Some(owner) != self.current_lambda
            && self.recursive_lambda != self.current_lambda.map(|lambda| (lambda, binding.id))
        {
            return Err(
                Diagnostic::error(format!("value `{}` is not captured", name.text))
                    .with_primary(name.span, "add this value to the lambda capture list"),
            );
        }
        Ok(ValueReference {
            id: binding.id,
            name: name.clone(),
        })
    }

    fn resolve_external_reference(
        &self,
        name: &ast::Name,
    ) -> Result<ExternalOperationReference, Diagnostic> {
        let binding = self
            .externals
            .get(&name.text)
            .ok_or_else(|| self.unknown(name, "external operation"))?;
        Ok(ExternalOperationReference {
            id: binding.id,
            name: name.clone(),
        })
    }
}
