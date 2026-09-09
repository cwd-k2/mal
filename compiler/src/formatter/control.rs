use crate::ast::{BodyItem, Expression, ExpressionBlock, Node, Program, TopItem};
use crate::lexer::Lexed;

pub(super) struct ControlLayout {
    aligned: Vec<bool>,
}

impl ControlLayout {
    pub(super) fn new(lexed: &Lexed, program: &Program) -> Self {
        let mut aligned = vec![false; lexed.tokens.len()];
        for item in &program.items {
            if let TopItem::Binding(binding) = &item.kind {
                mark_expression(lexed, &binding.value, false, &mut aligned);
            }
        }
        Self { aligned }
    }

    pub(super) fn is_aligned(&self, token_index: usize) -> bool {
        self.aligned[token_index]
    }
}

fn mark_block(lexed: &Lexed, block: &ExpressionBlock, aligned: &mut [bool]) {
    for item in &block.items {
        match item {
            BodyItem::Binding(binding) => {
                mark_expression(lexed, &binding.kind.value, false, aligned);
            }
            BodyItem::Expression(expression) => {
                mark_expression(lexed, expression, true, aligned);
            }
        }
    }
    mark_expression(lexed, &block.result, true, aligned);
}

fn mark_expression(
    lexed: &Lexed,
    expression: &Node<Expression>,
    block_position: bool,
    aligned: &mut [bool],
) {
    match &expression.kind {
        Expression::Parenthesized(inner) => {
            mark_expression(lexed, inner, block_position, aligned);
        }
        Expression::Product(fields) => {
            for field in fields {
                mark_expression(lexed, field, false, aligned);
            }
        }
        Expression::Lambda(lambda) => mark_block(lexed, &lambda.body, aligned),
        Expression::Call { callee, arguments } => {
            mark_expression(lexed, callee, false, aligned);
            for argument in arguments {
                mark_expression(lexed, argument, false, aligned);
            }
        }
        Expression::ExternalCall { arguments, .. } => {
            for argument in arguments {
                mark_expression(lexed, argument, false, aligned);
            }
        }
        Expression::Conversion { value, .. } | Expression::SumInjection { value, .. } => {
            mark_expression(lexed, value, false, aligned);
        }
        Expression::If {
            condition,
            then_branch,
            else_branch,
        } => {
            mark_control_start(lexed, expression, block_position, aligned);
            mark_expression(lexed, condition, false, aligned);
            mark_block(lexed, then_branch, aligned);
            mark_block(lexed, else_branch, aligned);
        }
        Expression::Case { scrutinee, arms } => {
            mark_control_start(lexed, expression, block_position, aligned);
            mark_expression(lexed, scrutinee, false, aligned);
            for arm in arms {
                mark_block(lexed, &arm.body, aligned);
            }
        }
        Expression::Unary { operand, .. } => mark_expression(lexed, operand, false, aligned),
        Expression::Binary { left, right, .. } => {
            mark_expression(lexed, left, false, aligned);
            mark_expression(lexed, right, false, aligned);
        }
        Expression::Name(_)
        | Expression::Integer(_)
        | Expression::Float(_)
        | Expression::Byte(_)
        | Expression::Symbol(_)
        | Expression::TypeQualifiedPrimitive { .. }
        | Expression::Unit => {}
    }
}

fn mark_control_start(
    lexed: &Lexed,
    expression: &Node<Expression>,
    block_position: bool,
    aligned: &mut [bool],
) {
    if !block_position {
        return;
    }
    if let Ok(index) = lexed
        .tokens
        .binary_search_by_key(&expression.span.start(), |token| token.span.start())
    {
        aligned[index] = true;
    }
}
