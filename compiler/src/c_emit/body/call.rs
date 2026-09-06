use crate::check::ast::Type;
use crate::closure::ast::{self as closure, Binding, Block, FunctionId, Operation, Pattern};

const MAX_DIRECT_PARAMETERS: usize = 16;

pub(super) fn has_direct_product_entry(ty: &Type) -> bool {
    matches!(ty, Type::Product(_)) && flattened_product_types(ty).len() <= MAX_DIRECT_PARAMETERS
}

pub(super) fn flattened_product_types(ty: &Type) -> Vec<&Type> {
    match ty {
        Type::Product(elements) => elements.iter().flat_map(flattened_product_types).collect(),
        _ => vec![ty],
    }
}

pub(super) fn flattened_product_values(
    ty: &Type,
    value: crate::c_emit::syntax::Expr,
) -> Vec<crate::c_emit::syntax::Expr> {
    match ty {
        Type::Product(elements) => elements
            .iter()
            .enumerate()
            .flat_map(|(index, element)| {
                flattened_product_values(element, value.clone().field(format!("field_{index}")))
            })
            .collect(),
        _ => vec![value],
    }
}

pub(super) fn has_direct_tail_call(block: &Block, function: FunctionId) -> bool {
    let Some(binding) = tail_binding(block) else {
        return false;
    };
    match &binding.operation {
        Operation::Call { callee, .. } => matches!(
            callee.kind,
            closure::AtomKind::Reference(closure::Reference::SelfClosure(id)) if id == function
        ),
        Operation::Case { arms, .. } => arms
            .iter()
            .any(|arm| has_direct_tail_call(&arm.value, function)),
        Operation::PrimitiveBranch {
            otherwise, then, ..
        } => has_direct_tail_call(otherwise, function) || has_direct_tail_call(then, function),
        _ => false,
    }
}

fn tail_binding(block: &Block) -> Option<&Binding> {
    block.bindings.last().filter(|binding| {
        matches!(
            (&block.result.kind, &binding.pattern),
            (
                closure::AtomKind::Reference(closure::Reference::Binding(result)),
                Pattern::Binding { id, .. }
            ) if result == id
        )
    })
}
