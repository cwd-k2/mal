use mal_frontend::check::ast::Type;

use crate::closure::ast::{
    Atom, AtomKind, Binding, Block, CaptureField, Operation, Pattern, Program, Reference,
};

use super::super::super::{ids::Identities, lambda_lift};
use super::Use;

pub(in crate::call_pattern::parameter_lift) fn prepare_function(
    program: &mut Program,
    nested: &Use,
    capture_types: &[Type],
    shape: &lambda_lift::LiftedShape,
    ids: &mut Identities,
) {
    let function = program
        .functions
        .iter_mut()
        .find(|function| function.id == nested.function)
        .expect("admitted nested callback function");
    let original_capture_count = function.captures.len();
    let outer_capture_type = shape.closure_type.clone();
    function.captures[nested.capture].ty = outer_capture_type.clone();
    function
        .captures
        .extend(capture_types.iter().cloned().map(|ty| CaptureField { ty }));
    rewrite_calls(
        function,
        nested,
        original_capture_count,
        &outer_capture_type,
        shape,
        capture_types,
        ids,
    );
}

pub(in crate::call_pattern::parameter_lift) fn extend_creator(
    operation: &mut Operation,
    nested: &Use,
    capture_bindings: &[(crate::anf::ast::ValueId, Type)],
    ids: &mut Identities,
) {
    let Operation::MakeClosure { function, captures } = operation else {
        return;
    };
    if *function != nested.function
        || !captures
            .get(nested.capture)
            .is_some_and(|atom| nested.capture_atoms.contains(&atom.id))
    {
        return;
    }
    let capture_span = captures[nested.capture].span;
    captures.extend(capture_bindings.iter().map(|(binding, ty)| Atom {
        id: ids.atom(),
        kind: AtomKind::Reference(Reference::Binding(*binding)),
        ty: ty.clone(),
        span: capture_span,
    }));
}

#[allow(clippy::too_many_arguments)]
fn rewrite_calls(
    function: &mut crate::closure::ast::Function,
    nested: &Use,
    original_capture_count: usize,
    outer_capture_type: &Type,
    shape: &lambda_lift::LiftedShape,
    capture_types: &[Type],
    ids: &mut Identities,
) {
    rewrite_block(
        &mut function.body,
        nested,
        original_capture_count,
        outer_capture_type,
        shape,
        capture_types,
        ids,
    );
    for join in &mut function.joins {
        rewrite_block(
            &mut join.body,
            nested,
            original_capture_count,
            outer_capture_type,
            shape,
            capture_types,
            ids,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn rewrite_block(
    block: &mut Block,
    nested: &Use,
    original_capture_count: usize,
    outer_capture_type: &Type,
    shape: &lambda_lift::LiftedShape,
    capture_types: &[Type],
    ids: &mut Identities,
) {
    let mut output = Vec::with_capacity(block.bindings.len());
    for mut binding in std::mem::take(&mut block.bindings) {
        match &mut binding.operation {
            Operation::Case { arms, .. } => {
                for arm in arms {
                    rewrite_block(
                        &mut arm.value,
                        nested,
                        original_capture_count,
                        outer_capture_type,
                        shape,
                        capture_types,
                        ids,
                    );
                }
            }
            Operation::PrimitiveBranch {
                otherwise, then, ..
            } => {
                rewrite_block(
                    otherwise,
                    nested,
                    original_capture_count,
                    outer_capture_type,
                    shape,
                    capture_types,
                    ids,
                );
                rewrite_block(
                    then,
                    nested,
                    original_capture_count,
                    outer_capture_type,
                    shape,
                    capture_types,
                    ids,
                );
            }
            _ => {}
        }
        rewrite_atoms(&mut binding.operation, |atom| {
            if nested_atom(atom, nested) {
                atom.ty = shape.closure_type.clone();
            } else if atom.kind == AtomKind::Reference(Reference::Capture(nested.capture)) {
                atom.ty = outer_capture_type.clone();
            }
        });
        rewrite_alias_pattern(&mut binding.pattern, nested, &shape.closure_type);
        if let Operation::Call { callee, argument } = &mut binding.operation
            && nested_atom(callee, nested)
        {
            let packed = ids.value();
            let mut elements = capture_types
                .iter()
                .enumerate()
                .map(|(index, ty)| Atom {
                    id: ids.atom(),
                    kind: AtomKind::Reference(Reference::Capture(original_capture_count + index)),
                    ty: ty.clone(),
                    span: argument.span,
                })
                .collect::<Vec<_>>();
            elements.push(argument.clone());
            output.push(Binding {
                pattern: Pattern::Binding {
                    id: packed,
                    ty: shape.parameter_type.clone(),
                },
                operation: Operation::Product(elements),
                span: argument.span,
            });
            *argument = Atom {
                id: ids.atom(),
                kind: AtomKind::Reference(Reference::Binding(packed)),
                ty: shape.parameter_type.clone(),
                span: argument.span,
            };
        }
        output.push(binding);
    }
    block.bindings = output;
    if nested_atom(&block.result, nested) {
        block.result.ty = shape.closure_type.clone();
    } else if block.result.kind == AtomKind::Reference(Reference::Capture(nested.capture)) {
        block.result.ty = outer_capture_type.clone();
    }
}

fn nested_atom(atom: &Atom, nested: &Use) -> bool {
    atom.kind == AtomKind::Reference(Reference::Capture(nested.capture))
        || atom
            .binding()
            .is_some_and(|binding| nested.aliases.contains(&binding))
}

fn rewrite_alias_pattern(pattern: &mut Pattern, nested: &Use, replacement: &Type) {
    match pattern {
        Pattern::Binding { id, ty } if nested.aliases.contains(id) => *ty = replacement.clone(),
        Pattern::Product { elements, ty, .. } => {
            for element in elements.iter_mut() {
                rewrite_alias_pattern(element, nested, replacement);
            }
            *ty = Type::Product(
                elements
                    .iter()
                    .map(Pattern::ty)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into(),
            );
        }
        _ => {}
    }
}

fn rewrite_atoms(operation: &mut Operation, mut visit: impl FnMut(&mut Atom)) {
    match operation {
        Operation::Atom(atom)
        | Operation::Goto { value: atom, .. }
        | Operation::ExternalCall { argument: atom, .. }
        | Operation::NumericConversion { operand: atom }
        | Operation::SumInjection { value: atom, .. }
        | Operation::PrimitiveUnary { operand: atom, .. } => visit(atom),
        Operation::MakeClosure { captures, .. }
        | Operation::Product(captures)
        | Operation::Memory {
            operands: captures, ..
        }
        | Operation::Symbol {
            operands: captures, ..
        }
        | Operation::Buffer {
            operands: captures, ..
        } => captures.iter_mut().for_each(visit),
        Operation::Call { callee, argument } => {
            visit(callee);
            visit(argument);
        }
        Operation::Case { scrutinee, .. } => visit(scrutinee),
        Operation::PrimitiveBranch { left, right, .. }
        | Operation::PrimitiveBinary { left, right, .. } => {
            visit(left);
            visit(right);
        }
    }
}
