use mal_syntax::ast::{BodyItem, Expression, ExpressionBlock, Node, Program, TopItem};
use mal_syntax::lexer::{Lexed, TokenKind};
use mal_syntax::source::SourceFile;

use super::layout::BlockLayout;

pub(super) struct ControlLayout {
    aligned: Vec<bool>,
    inline: Vec<bool>,
    completed_before: Vec<usize>,
    sum_continuations: Vec<bool>,
    sum_break_before: Vec<bool>,
}

impl ControlLayout {
    pub(super) fn new(
        source: &SourceFile,
        lexed: &Lexed,
        program: &Program,
        blocks: &BlockLayout,
    ) -> Self {
        let mut layout = Self {
            aligned: vec![false; lexed.tokens.len()],
            inline: vec![false; lexed.tokens.len()],
            completed_before: vec![0; lexed.tokens.len()],
            sum_continuations: vec![false; lexed.tokens.len()],
            sum_break_before: vec![false; lexed.tokens.len()],
        };
        for item in &program.items {
            match &item.kind {
                TopItem::Binding(binding) => layout.mark_expression(
                    source,
                    lexed,
                    blocks,
                    &binding.value,
                    ExpressionPosition::Root,
                ),
                TopItem::GenericBinding { value, .. } => {
                    layout.mark_expression(source, lexed, blocks, value, ExpressionPosition::Root);
                }
                _ => {}
            }
        }
        layout
    }

    pub(super) fn is_aligned(&self, token_index: usize) -> bool {
        self.aligned[token_index]
    }

    pub(super) fn is_inline(&self, token_index: usize) -> bool {
        self.inline[token_index]
    }

    pub(super) fn completed_before(&self, token_index: usize) -> usize {
        self.completed_before[token_index]
    }

    pub(super) fn is_sum_continuation(&self, token_index: usize) -> bool {
        self.sum_continuations[token_index]
    }

    pub(super) fn should_break_before_sum_token(&self, token_index: usize) -> bool {
        self.sum_break_before[token_index]
    }

    fn mark_expression(
        &mut self,
        source: &SourceFile,
        lexed: &Lexed,
        blocks: &BlockLayout,
        expression: &Node<Expression>,
        position: ExpressionPosition,
    ) {
        let mut pending = vec![(expression, position)];
        while let Some((expression, position)) = pending.pop() {
            match &expression.kind {
                Expression::Parenthesized(inner) => pending.push((inner, position)),
                Expression::Product(fields) => {
                    pending.extend(
                        fields
                            .iter()
                            .rev()
                            .map(|field| (field, ExpressionPosition::Embedded)),
                    );
                }
                Expression::Lambda(lambda) => push_block(&mut pending, &lambda.body),
                Expression::Block(block) => push_block(&mut pending, block),
                Expression::ResultBlock { body, .. } => push_block(&mut pending, body),
                Expression::Call { callee, arguments } => {
                    pending.extend(
                        arguments
                            .iter()
                            .rev()
                            .map(|argument| (argument, ExpressionPosition::Embedded)),
                    );
                    pending.push((callee, ExpressionPosition::Embedded));
                }
                Expression::ContinuationApplication {
                    value,
                    continuations,
                } => {
                    if continuations.len() >= 2
                        && let Some(bracket) = sum_bracket(lexed, expression, value)
                        && !is_inline_sum_continuation(
                            source,
                            lexed,
                            blocks,
                            bracket,
                            expression.span.end(),
                        )
                    {
                        self.mark_sum_continuation(lexed, expression, continuations, bracket);
                    }
                    pending.extend(
                        continuations
                            .iter()
                            .rev()
                            .map(|continuation| (continuation, ExpressionPosition::Embedded)),
                    );
                    pending.push((value, ExpressionPosition::Embedded));
                }
                Expression::Conversion { value, .. } => {
                    pending.push((value, ExpressionPosition::Embedded));
                }
                Expression::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    if let Some(index) = self.mark_control_start(lexed, expression, position, true)
                    {
                        self.inline[index] = matches!(position, ExpressionPosition::Embedded)
                            && is_inline_if(
                                source,
                                lexed,
                                blocks,
                                expression,
                                condition,
                                (then_branch, else_branch),
                            );
                    }
                    push_block(&mut pending, else_branch);
                    push_block(&mut pending, then_branch);
                    pending.push((condition, ExpressionPosition::Embedded));
                }
                Expression::When { condition, body } => {
                    self.mark_control_start(lexed, expression, position, false);
                    push_block(&mut pending, body);
                    pending.push((condition, ExpressionPosition::Embedded));
                }
                Expression::Unary { operand, .. } => {
                    pending.push((operand, ExpressionPosition::Embedded));
                }
                Expression::Binary { left, right, .. } => {
                    pending.push((right, ExpressionPosition::Embedded));
                    pending.push((left, ExpressionPosition::Embedded));
                }
                Expression::Name(_)
                | Expression::GenericName { .. }
                | Expression::Integer(_)
                | Expression::Float(_)
                | Expression::Byte(_)
                | Expression::Symbol(_)
                | Expression::Unit => {}
            }
        }
    }

    /// Puts each continuation of a sum elimination on its own line.
    fn mark_sum_continuation(
        &mut self,
        lexed: &Lexed,
        expression: &Node<Expression>,
        continuations: &[Node<Expression>],
        bracket: usize,
    ) {
        self.sum_continuations[bracket] = true;
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
        position: ExpressionPosition,
        tracks_completion: bool,
    ) -> Option<usize> {
        if tracks_completion {
            let next = lexed
                .tokens
                .partition_point(|token| token.span.end() <= expression.span.end());
            if let Some(count) = self.completed_before.get_mut(next) {
                *count += 1;
            }
        }
        if let Ok(index) = lexed
            .tokens
            .binary_search_by_key(&expression.span.start(), |token| token.span.start())
        {
            self.aligned[index] = matches!(position, ExpressionPosition::Block);
            return Some(index);
        }
        None
    }
}

#[derive(Clone, Copy)]
enum ExpressionPosition {
    Root,
    Block,
    Embedded,
}

fn push_block<'a>(
    pending: &mut Vec<(&'a Node<Expression>, ExpressionPosition)>,
    block: &'a ExpressionBlock,
) {
    pending.push((
        &block.result,
        if block.span == block.result.span {
            ExpressionPosition::Root
        } else {
            ExpressionPosition::Block
        },
    ));
    pending.extend(block.items.iter().rev().map(|item| match item {
        BodyItem::Binding(binding) => (&binding.kind.value, ExpressionPosition::Root),
        BodyItem::Expression(expression) => (expression, ExpressionPosition::Block),
    }));
}

fn compact_branch(
    source: &SourceFile,
    lexed: &Lexed,
    blocks: &BlockLayout,
    branch: &ExpressionBlock,
) -> bool {
    let token = lexed
        .tokens
        .binary_search_by_key(&branch.span.start(), |token| token.span.start())
        .ok();
    if token.is_some_and(|index| matches!(lexed.tokens[index].kind, TokenKind::LeftBrace)) {
        return token.is_some_and(|index| blocks.is_compact(index));
    }
    branch.items.is_empty()
        && source
            .location(branch.span.start())
            .zip(source.location(branch.span.end()))
            .is_some_and(|(start, end)| start.line == end.line)
}

/// The token index of the `[` that opens the continuation list after `value`.
fn sum_bracket(
    lexed: &Lexed,
    expression: &Node<Expression>,
    value: &Node<Expression>,
) -> Option<usize> {
    let start = lexed
        .tokens
        .partition_point(|token| token.span.start() < value.span.end());
    lexed.tokens[start..]
        .iter()
        .take_while(|token| token.span.end() <= expression.span.end())
        .position(|token| matches!(token.kind, TokenKind::LeftBracket))
        .map(|offset| start + offset)
}

/// The source keeps the whole bracketed list on one line and none of its blocks needs to expand.
fn is_inline_sum_continuation(
    source: &SourceFile,
    lexed: &Lexed,
    blocks: &BlockLayout,
    bracket: usize,
    end: usize,
) -> bool {
    source
        .location(lexed.tokens[bracket].span.start())
        .zip(source.location(end))
        .is_some_and(|(start, end)| start.line == end.line)
        && lexed.tokens[bracket..]
            .iter()
            .take_while(|token| token.span.end() <= end)
            .enumerate()
            .all(|(offset, token)| {
                !matches!(token.kind, TokenKind::LeftBrace) || blocks.is_compact(bracket + offset)
            })
}

fn is_inline_if(
    source: &SourceFile,
    lexed: &Lexed,
    blocks: &BlockLayout,
    expression: &Node<Expression>,
    condition: &Node<Expression>,
    branches: (&ExpressionBlock, &ExpressionBlock),
) -> bool {
    source
        .location(expression.span.start())
        .zip(source.location(expression.span.end()))
        .is_some_and(|(start, end)| start.line == end.line)
        && source
            .location(expression.span.start())
            .zip(source.location(condition.span.end()))
            .is_some_and(|(start, end)| start.line == end.line)
        && compact_branch(source, lexed, blocks, branches.0)
        && compact_branch(source, lexed, blocks, branches.1)
}
