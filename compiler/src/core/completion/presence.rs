use crate::check::ast as checked;

pub(super) fn contains_control(value: &checked::Expression) -> bool {
    use checked::ExpressionKind;
    match &value.kind {
        ExpressionKind::Parenthesized(inner) => contains_control(inner),
        ExpressionKind::Product(elements) => elements.iter().any(contains_control),
        ExpressionKind::Call { callee, argument } => {
            contains_control(callee) || contains_control(argument)
        }
        ExpressionKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            contains_control(condition)
                || block_contains_control(then_branch)
                || block_contains_control(else_branch)
        }
        ExpressionKind::Unary { operand, .. }
        | ExpressionKind::SymbolLength { value: operand }
        | ExpressionKind::NumericConversion { value: operand }
        | ExpressionKind::SumInjection { value: operand, .. } => contains_control(operand),
        ExpressionKind::Binary { left, right, .. } => {
            contains_control(left) || contains_control(right)
        }
        ExpressionKind::SymbolAt { argument } | ExpressionKind::Memory { argument, .. } => {
            contains_control(argument)
        }
        ExpressionKind::SumElimination {
            scrutinee,
            continuations,
        } => contains_control(scrutinee) || continuations.iter().any(contains_control),
        ExpressionKind::Lambda(_)
        | ExpressionKind::Reference(_)
        | ExpressionKind::Integer(_)
        | ExpressionKind::Float(_)
        | ExpressionKind::Symbol(_)
        | ExpressionKind::StorageSize(_)
        | ExpressionKind::Unit
        | ExpressionKind::MemoryFunction { .. }
        | ExpressionKind::InjectionConstructor { .. } => false,
    }
}

fn block_contains_control(block: &checked::ExpressionBlock) -> bool {
    block.items.iter().any(|item| match item {
        checked::BodyItem::Binding(binding) => contains_control(&binding.value),
        checked::BodyItem::Expression(value) => contains_control(value),
    }) || match block.result.as_ref() {
        checked::Completion::Value(value) => contains_control(value),
        checked::Completion::Abrupt(_) => true,
    }
}
