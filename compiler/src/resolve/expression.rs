use crate::ast;
use crate::diagnostic::Diagnostic;

use super::Resolver;
use super::ast::{
    BodyItem, Capture, Expression, ExpressionBlock, Lambda, LambdaBody, ValueOwner, ValueReference,
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
            ast::Expression::Symbol(value) => Expression::Symbol(value.clone()),
            ast::Expression::TypeQualifiedPrimitive { type_name, member } => {
                Expression::TypeQualifiedPrimitive {
                    type_ref: self.type_reference(type_name)?,
                    member: member.clone(),
                }
            }
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
            ast::Expression::ContinuationApplication {
                value,
                continuations,
            } => Expression::ContinuationApplication {
                value: Box::new(self.resolve_expression(value)?),
                continuations: continuations
                    .iter()
                    .map(|continuation| self.resolve_expression(continuation))
                    .collect::<Result<_, _>>()?,
            },
            ast::Expression::Conversion { type_name, value } => Expression::Conversion {
                type_ref: self.type_reference(type_name)?,
                lambda_id: self.allocate_lambda(),
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
            ast::Expression::When { condition, body } => Expression::When {
                condition: Box::new(self.resolve_expression(condition)?),
                body: self.resolve_expression_block(body)?,
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
        let id = self.allocate_lambda();
        let outer_lambda = self.current_lambda;
        let outer_recursive_lambda = self.recursive_lambda;
        self.current_lambda = Some(id);
        self.recursive_lambda = self_binding.as_ref().map(|binding| (id, binding.id));
        self.push_scope();
        self.lambda_frames.push(super::LambdaFrame {
            id,
            captures: Vec::new(),
            captured_sources: std::collections::HashMap::new(),
        });
        let result = (|| {
            let parameter = lambda
                .parameter
                .as_ref()
                .map(|parameter| self.declare_pattern(parameter, ValueOwner::Lambda(id)))
                .transpose()?
                .map(Box::new);
            let return_binders = lambda
                .return_binders
                .as_ref()
                .map(|binders| {
                    binders
                        .iter()
                        .map(|name| self.declare_value(name, ValueOwner::Return(id)))
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?;
            let body = self.resolve_lambda_body(&lambda.body)?;
            let captures = std::mem::take(
                &mut self
                    .lambda_frames
                    .last_mut()
                    .expect("active lambda frame")
                    .captures,
            );
            Ok(Lambda {
                id,
                self_binding: self_binding.map(|binding| binding.id),
                captures,
                parameter,
                return_binders,
                body,
            })
        })();
        self.lambda_frames.pop().expect("active lambda frame");
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
        let result = self.resolve_expression_block_contents(block);
        self.pop_scope();
        result
    }

    fn resolve_expression_block_contents(
        &mut self,
        block: &ast::ExpressionBlock,
    ) -> Result<ExpressionBlock, Diagnostic> {
        let mut items = Vec::with_capacity(block.items.len());
        for item in &block.items {
            items.push(self.resolve_body_item(item)?);
        }
        Ok(ExpressionBlock {
            items,
            result: Box::new(self.resolve_expression(&block.result)?),
            span: block.span,
        })
    }

    fn resolve_value_reference(&mut self, name: &ast::Name) -> Result<ValueReference, Diagnostic> {
        let mut binding = self
            .lookup_value(&name.text)
            .ok_or_else(|| self.unknown(name, "value"))?;
        if let ValueOwner::Return(owner) = binding.owner
            && Some(owner) != self.current_lambda
        {
            return Err(
                Diagnostic::error("return binder cannot be captured").with_primary(
                    name.span,
                    "this binder belongs to an enclosing lambda invocation",
                ),
            );
        }
        if let ValueOwner::Lambda(owner) = binding.owner
            && Some(owner) != self.current_lambda
            && self.recursive_lambda != self.current_lambda.map(|lambda| (lambda, binding.id))
        {
            let owner_index = self
                .lambda_frames
                .iter()
                .position(|frame| frame.id == owner)
                .expect("an in-scope lambda-owned value has an active owner");
            for frame_index in owner_index + 1..self.lambda_frames.len() {
                binding = self.capture_in_frame(frame_index, binding, name);
            }
        }
        Ok(ValueReference {
            id: binding.id,
            name: name.clone(),
        })
    }

    fn capture_in_frame(
        &mut self,
        frame_index: usize,
        source: super::ast::ValueBinding,
        name: &ast::Name,
    ) -> super::ast::ValueBinding {
        if let Some(binding) = self.lambda_frames[frame_index]
            .captured_sources
            .get(&source.id)
        {
            return binding.clone();
        }
        let owner = ValueOwner::Lambda(self.lambda_frames[frame_index].id);
        let binding = self.allocate_value_binding(name, owner);
        self.lambda_frames[frame_index]
            .captured_sources
            .insert(source.id, binding.clone());
        self.lambda_frames[frame_index].captures.push(Capture {
            source: ValueReference {
                id: source.id,
                name: name.clone(),
            },
            binding: binding.clone(),
        });
        binding
    }
}
