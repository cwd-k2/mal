use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::closure::ast::{self as closure, AtomKind, FunctionId, Pattern, Reference};

pub(super) fn stable_symbol_slots(function: &closure::Function) -> Option<Vec<ValueId>> {
    let slots = direct_tail_parameter_slots(function)?;
    Some(
        slots
            .iter()
            .enumerate()
            .filter_map(|(slot_index, slot)| {
                (slot.ty == &Type::Symbol
                    && tail_calls_carry_slot(
                        &function.body,
                        function.id,
                        slots.len(),
                        slot_index,
                        slot.id,
                    ))
                .then_some(slot.id)
            })
            .collect(),
    )
}

struct TailParameterSlot<'a> {
    id: ValueId,
    ty: &'a Type,
}

fn direct_tail_parameter_slots(function: &closure::Function) -> Option<Vec<TailParameterSlot<'_>>> {
    if !super::super::super::has_direct_tail_call(&function.body, function.id) {
        return None;
    }
    let parameter = function.parameter.binding?;
    let first = function.body.bindings.first()?;
    let closure::Operation::Atom(closure::Atom {
        kind: AtomKind::Reference(Reference::Binding(source)),
        ..
    }) = &first.operation
    else {
        return None;
    };
    if *source != parameter {
        return None;
    }
    let Pattern::Product { elements, .. } = &first.pattern else {
        return None;
    };
    let slots = elements
        .iter()
        .map(|pattern| match pattern {
            Pattern::Binding { id, ty } => Some(TailParameterSlot { id: *id, ty }),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    tail_calls_have_flat_products(&function.body, function.id, slots.len()).then_some(slots)
}

fn tail_calls_have_flat_products(
    block: &closure::Block,
    function: FunctionId,
    arity: usize,
) -> bool {
    let Some(tail) = returned_tail_binding(block) else {
        return true;
    };
    match &tail.operation {
        closure::Operation::Call { callee, argument }
            if matches!(
                callee.kind,
                AtomKind::Reference(Reference::SelfClosure(target)) if target == function
            ) =>
        {
            tail_product_elements(block, argument).is_some_and(|elements| elements.len() == arity)
        }
        closure::Operation::Case { arms, .. } => arms
            .iter()
            .all(|arm| tail_calls_have_flat_products(&arm.value, function, arity)),
        closure::Operation::PrimitiveBranch {
            otherwise, then, ..
        } => {
            tail_calls_have_flat_products(otherwise, function, arity)
                && tail_calls_have_flat_products(then, function, arity)
        }
        _ => true,
    }
}

fn tail_calls_carry_slot(
    block: &closure::Block,
    function: FunctionId,
    arity: usize,
    slot_index: usize,
    slot: ValueId,
) -> bool {
    let Some(tail) = returned_tail_binding(block) else {
        return true;
    };
    match &tail.operation {
        closure::Operation::Call { callee, argument }
            if matches!(
                callee.kind,
                AtomKind::Reference(Reference::SelfClosure(target)) if target == function
            ) =>
        {
            tail_product_elements(block, argument).is_some_and(|elements| {
                elements.len() == arity
                    && matches!(
                        elements[slot_index].kind,
                        AtomKind::Reference(Reference::Binding(id)) if id == slot
                    )
            })
        }
        closure::Operation::Case { arms, .. } => arms
            .iter()
            .all(|arm| tail_calls_carry_slot(&arm.value, function, arity, slot_index, slot)),
        closure::Operation::PrimitiveBranch {
            otherwise, then, ..
        } => {
            tail_calls_carry_slot(otherwise, function, arity, slot_index, slot)
                && tail_calls_carry_slot(then, function, arity, slot_index, slot)
        }
        _ => true,
    }
}

fn returned_tail_binding(block: &closure::Block) -> Option<&closure::Binding> {
    block.bindings.last().filter(|binding| {
        matches!(
            (&block.result.kind, &binding.pattern),
            (
                AtomKind::Reference(Reference::Binding(result)),
                Pattern::Binding { id, .. }
            ) if result == id
        )
    })
}

fn tail_product_elements<'a>(
    block: &'a closure::Block,
    argument: &closure::Atom,
) -> Option<&'a [closure::Atom]> {
    let AtomKind::Reference(Reference::Binding(argument_id)) = argument.kind else {
        return None;
    };
    let product_index = block.bindings.len().checked_sub(2)?;
    match block.bindings.get(product_index) {
        Some(closure::Binding {
            pattern: Pattern::Binding { id, .. },
            operation: closure::Operation::Product(elements),
            ..
        }) if *id == argument_id => Some(elements),
        _ => None,
    }
}
