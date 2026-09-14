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

    fn mark_expression(
        &mut self,
        lexed: &Lexed,
        expression: &Node<Expression>,
        block_position: bool,
    ) {
        let mut pending = vec![(expression, block_position)];
        while let Some((expression, block_position)) = pending.pop() {
            match &expression.kind {
                Expression::Parenthesized(inner) => pending.push((inner, block_position)),
                Expression::Product(fields) => {
                    pending.extend(fields.iter().rev().map(|field| (field, false)));
                }
                Expression::Lambda(lambda) => push_block(&mut pending, &lambda.body),
                Expression::Block(block) => push_block(&mut pending, block),
                Expression::ResultBlock { body, .. } => push_block(&mut pending, body),
                Expression::Call { callee, arguments } => {
                    pending.extend(arguments.iter().rev().map(|argument| (argument, false)));
                    pending.push((callee, false));
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
                    pending.extend(
                        continuations
                            .iter()
                            .rev()
                            .map(|continuation| (continuation, false)),
                    );
                    pending.push((value, false));
                }
                Expression::Conversion { value, .. } => pending.push((value, false)),
                Expression::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    self.mark_control_start(lexed, expression, block_position);
                    push_block(&mut pending, else_branch);
                    push_block(&mut pending, then_branch);
                    pending.push((condition, false));
                }
                Expression::When { condition, body } => {
                    self.mark_control_start(lexed, expression, block_position);
                    push_block(&mut pending, body);
                    pending.push((condition, false));
                }
                Expression::Unary { operand, .. } => pending.push((operand, false)),
                Expression::Binary { left, right, .. } => {
                    pending.push((right, false));
                    pending.push((left, false));
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
    }

    fn mark_sum_continuation(
        &mut self,
        lexed: &Lexed,
        expression: &Node<Expression>,
        value: &Node<Expression>,
        continuations: &[Node<Expression>],
        block_position: bool,
    ) {
        let start = lexed
            .tokens
            .partition_point(|token| token.span.start() < value.span.end());
        let Some(offset) = lexed.tokens[start..]
            .iter()
            .take_while(|token| token.span.end() <= expression.span.end())
            .position(|token| matches!(token.kind, TokenKind::LeftBracket))
        else {
            return;
        };
        let bracket = start + offset;
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

fn push_block<'a>(pending: &mut Vec<(&'a Node<Expression>, bool)>, block: &'a ExpressionBlock) {
    pending.push((&block.result, true));
    pending.extend(block.items.iter().rev().map(|item| match item {
        BodyItem::Binding(binding) => (&binding.kind.value, false),
        BodyItem::Expression(expression) => (expression, true),
    }));
}
