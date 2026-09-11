use crate::ast::{BodyItem, Expression, ExpressionBlock, Node, Program, TopItem};
use crate::lexer::{Lexed, TokenKind};

pub(super) struct ControlLayout {
    aligned: Vec<bool>,
    sum_continuations: Vec<Option<bool>>,
    sum_break_before: Vec<bool>,
}

impl ControlLayout {
    pub(super) fn new(lexed: &Lexed, program: &Program) -> Self {
        let mut layout = Self {
            aligned: vec![false; lexed.tokens.len()],
            sum_continuations: vec![None; lexed.tokens.len()],
            sum_break_before: vec![false; lexed.tokens.len()],
        };
        for item in &program.items {
            if let TopItem::Binding(binding) = &item.kind {
                layout.mark_expression(lexed, &binding.value, false);
            }
        }
        layout
    }

    pub(super) fn is_aligned(&self, token_index: usize) -> bool {
        self.aligned[token_index]
    }

    pub(super) fn sum_continuation_alignment(&self, token_index: usize) -> Option<bool> {
        self.sum_continuations[token_index]
    }

    pub(super) fn should_break_before_sum_token(&self, token_index: usize) -> bool {
        self.sum_break_before[token_index]
    }

    fn mark_block(&mut self, lexed: &Lexed, block: &ExpressionBlock) {
        for item in &block.items {
            match item {
                BodyItem::Binding(binding) => {
                    self.mark_expression(lexed, &binding.kind.value, false);
                }
                BodyItem::Expression(expression) => {
                    self.mark_expression(lexed, expression, true);
                }
            }
        }
        self.mark_expression(lexed, &block.result, true);
    }

    fn mark_expression(
        &mut self,
        lexed: &Lexed,
        expression: &Node<Expression>,
        block_position: bool,
    ) {
        match &expression.kind {
            Expression::Parenthesized(inner) => {
                self.mark_expression(lexed, inner, block_position);
            }
            Expression::Product(fields) => {
                for field in fields {
                    self.mark_expression(lexed, field, false);
                }
            }
            Expression::Lambda(lambda) => self.mark_block(lexed, &lambda.body),
            Expression::Call { callee, arguments } => {
                self.mark_expression(lexed, callee, false);
                for argument in arguments {
                    self.mark_expression(lexed, argument, false);
                }
            }
            Expression::ContinuationApplication {
                value,
                continuations,
            } => {
                if continuations.len() >= 2 {
                    self.mark_sum_continuation(
                        lexed,
                        expression,
                        value,
                        continuations,
                        block_position,
                    );
                }
                self.mark_expression(lexed, value, false);
                for continuation in continuations {
                    self.mark_expression(lexed, continuation, false);
                }
            }
            Expression::Conversion { value, .. } => {
                self.mark_expression(lexed, value, false);
            }
            Expression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.mark_control_start(lexed, expression, block_position);
                self.mark_expression(lexed, condition, false);
                self.mark_block(lexed, then_branch);
                self.mark_block(lexed, else_branch);
            }
            Expression::When { condition, body } => {
                self.mark_control_start(lexed, expression, block_position);
                self.mark_expression(lexed, condition, false);
                self.mark_block(lexed, body);
            }
            Expression::Unary { operand, .. } => self.mark_expression(lexed, operand, false),
            Expression::Binary { left, right, .. } => {
                self.mark_expression(lexed, left, false);
                self.mark_expression(lexed, right, false);
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

    fn mark_sum_continuation(
        &mut self,
        lexed: &Lexed,
        expression: &Node<Expression>,
        value: &Node<Expression>,
        continuations: &[Node<Expression>],
        block_position: bool,
    ) {
        let Some(bracket) = lexed.tokens.iter().position(|token| {
            token.span.start() >= value.span.end()
                && token.span.end() <= expression.span.end()
                && matches!(token.kind, TokenKind::LeftBracket)
        }) else {
            return;
        };
        self.sum_continuations[bracket] = Some(block_position);
        for continuation in continuations {
            if let Ok(index) = lexed
                .tokens
                .binary_search_by_key(&continuation.span.start(), |token| token.span.start())
            {
                self.sum_break_before[index] = true;
            }
        }
        if let Ok(index) = lexed
            .tokens
            .binary_search_by_key(&expression.span.end(), |token| token.span.end())
        {
            self.sum_break_before[index] = true;
        }
    }

    fn mark_control_start(
        &mut self,
        lexed: &Lexed,
        expression: &Node<Expression>,
        block_position: bool,
    ) {
        if !block_position {
            return;
        }
        if let Ok(index) = lexed
            .tokens
            .binary_search_by_key(&expression.span.start(), |token| token.span.start())
        {
            self.aligned[index] = true;
        }
    }
}
