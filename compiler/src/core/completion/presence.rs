use crate::check::ast as checked;

pub(super) fn contains_control(value: &checked::Expression) -> bool {
    let mut pending = vec![Presence::Expression(value)];
    while let Some(item) = pending.pop() {
        match item {
            Presence::Expression(value) => match &value.kind {
                checked::ExpressionKind::Parenthesized(inner) => {
                    pending.push(Presence::Expression(inner));
                }
                checked::ExpressionKind::Block(block) => {
                    pending.push(Presence::Block(block));
                }
                checked::ExpressionKind::ResultBlock { .. } => return true,
                checked::ExpressionKind::Product(elements) => {
                    pending.extend(elements.iter().rev().map(Presence::Expression));
                }
                checked::ExpressionKind::Call { callee, argument } => {
                    pending.push(Presence::Expression(argument));
                    pending.push(Presence::Expression(callee));
                }
                checked::ExpressionKind::PackedBuild {
                    build, callback, ..
                } => {
                    pending.push(Presence::Expression(callback));
                    match build {
                        checked::PackedBuild::Make { capacity } => {
                            pending.push(Presence::Expression(capacity));
                        }
                        checked::PackedBuild::Edit { source } => {
                            pending.push(Presence::Expression(source));
                        }
                    }
                }
                checked::ExpressionKind::RegionView {
                    range, callback, ..
                } => {
                    pending.push(Presence::Expression(callback));
                    pending.push(Presence::Expression(range));
                }
                checked::ExpressionKind::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    pending.push(Presence::Block(else_branch));
                    pending.push(Presence::Block(then_branch));
                    pending.push(Presence::Expression(condition));
                }
                checked::ExpressionKind::Unary { operand, .. }
                | checked::ExpressionKind::SymbolLength { value: operand }
                | checked::ExpressionKind::NumericConversion { value: operand }
                | checked::ExpressionKind::SumInjection { value: operand, .. } => {
                    pending.push(Presence::Expression(operand));
                }
                checked::ExpressionKind::Binary { left, right, .. } => {
                    pending.push(Presence::Expression(right));
                    pending.push(Presence::Expression(left));
                }
                checked::ExpressionKind::SymbolAt { argument } => {
                    pending.push(Presence::Expression(argument));
                }
                checked::ExpressionKind::Memory { operands, .. } => {
                    pending.extend(operands.iter().rev().map(Presence::Expression));
                }
                checked::ExpressionKind::SumElimination {
                    scrutinee,
                    continuations,
                } => {
                    pending.extend(continuations.iter().rev().map(Presence::Expression));
                    pending.push(Presence::Expression(scrutinee));
                }
                checked::ExpressionKind::Lambda(_)
                | checked::ExpressionKind::Reference(_)
                | checked::ExpressionKind::GenericReference { .. }
                | checked::ExpressionKind::Integer(_)
                | checked::ExpressionKind::Float(_)
                | checked::ExpressionKind::Symbol(_)
                | checked::ExpressionKind::StorageSize(_)
                | checked::ExpressionKind::Unit => {}
            },
            Presence::Block(block) => {
                pending.push(Presence::Completion(&block.result));
                pending.extend(block.items.iter().rev().map(|item| {
                    Presence::Expression(match item {
                        checked::BodyItem::Binding(binding) => &binding.value,
                        checked::BodyItem::Expression(value) => value,
                    })
                }));
            }
            Presence::Completion(checked::Completion::Value(value)) => {
                pending.push(Presence::Expression(value));
            }
            Presence::Completion(checked::Completion::Abrupt(_)) => return true,
        }
    }
    false
}

enum Presence<'a> {
    Expression(&'a checked::Expression),
    Block(&'a checked::ExpressionBlock),
    Completion(&'a checked::Completion),
}
