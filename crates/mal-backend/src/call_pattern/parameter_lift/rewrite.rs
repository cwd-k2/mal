use std::collections::{HashMap, HashSet};

use mal_frontend::check::ast::Type;

use crate::anf::ast::ValueId;
use crate::closure::ast::{
    Atom, AtomKind, Binding, Block, Operation, Pattern, Program, Reference, TopLevelPattern,
};

use super::super::{ids::Identities, lambda_lift};
use super::Candidate;

pub(super) fn apply(program: &mut Program, candidate: Candidate) {
    let mut ids = Identities::after(program);
    let lifted = lambda_lift::lift_functions(program, &HashSet::from([candidate.target]), &mut ids);
    let shape = &lifted[&candidate.target];

    let capture_bindings = candidate
        .capture_types
        .iter()
        .map(|ty| (ids.value(), ty.clone()))
        .collect::<Vec<_>>();
    if let Some(nested) = &candidate.nested {
        super::nested::prepare_function(program, nested, &candidate.capture_types, shape, &mut ids);
    }
    for function in &mut program.functions {
        if function.id == candidate.host {
            function.parameter.ty = candidate.host_parameter_type.clone();
            expand_callback_pattern(function, &candidate, &capture_bindings, shape);
        }
    }
    let mut types = HashMap::from([
        (candidate.parameter, candidate.host_parameter_type.clone()),
        (candidate.callback, shape.closure_type.clone()),
        (candidate.host_binding, candidate.host_closure_type.clone()),
    ]);
    rewrite_top_level_patterns(
        &mut program.bindings,
        candidate.host_binding,
        &candidate.host_closure_type,
    );
    for binding in &mut program.bindings {
        rewrite_block(
            &mut binding.value,
            &candidate,
            shape,
            &capture_bindings,
            &mut types,
            &mut ids,
        );
    }
    for function in &mut program.functions {
        rewrite_block(
            &mut function.body,
            &candidate,
            shape,
            &capture_bindings,
            &mut types,
            &mut ids,
        );
        for join in &mut function.joins {
            rewrite_block(
                &mut join.body,
                &candidate,
                shape,
                &capture_bindings,
                &mut types,
                &mut ids,
            );
        }
    }
}

fn expand_callback_pattern(
    function: &mut crate::closure::ast::Function,
    candidate: &Candidate,
    captures: &[(ValueId, Type)],
    shape: &lambda_lift::LiftedShape,
) {
    for index in 0..function.body.bindings.len() {
        let binding = &mut function.body.bindings[index];
        let span = binding.span;
        if !matches!(&binding.operation, Operation::Atom(atom) if atom.binding() == Some(candidate.parameter))
            || !replace_callback_pattern(&mut binding.pattern, candidate.callback, captures, span)
        {
            continue;
        }
        function.body.bindings.insert(
            index + 1,
            Binding {
                pattern: Pattern::Binding {
                    id: candidate.callback,
                    ty: shape.closure_type.clone(),
                },
                operation: Operation::MakeClosure {
                    function: candidate.target,
                    captures: Vec::new(),
                },
                span,
            },
        );
        return;
    }
}

fn replace_callback_pattern(
    pattern: &mut Pattern,
    callback: ValueId,
    captures: &[(ValueId, Type)],
    span: mal_syntax::source::Span,
) -> bool {
    match pattern {
        Pattern::Binding { id, .. } if *id == callback => {
            *pattern = Pattern::Product {
                elements: captures
                    .iter()
                    .map(|(id, ty)| Pattern::Binding {
                        id: *id,
                        ty: ty.clone(),
                    })
                    .collect(),
                ty: Type::Product(
                    captures
                        .iter()
                        .map(|(_, ty)| ty.clone())
                        .collect::<Vec<_>>()
                        .into(),
                ),
                span,
            };
            true
        }
        Pattern::Product { elements, .. } => elements
            .iter_mut()
            .any(|element| replace_callback_pattern(element, callback, captures, span)),
        _ => false,
    }
}

fn rewrite_block(
    block: &mut Block,
    candidate: &Candidate,
    shape: &lambda_lift::LiftedShape,
    capture_bindings: &[(ValueId, Type)],
    types: &mut HashMap<ValueId, Type>,
    ids: &mut Identities,
) {
    let mut output = Vec::with_capacity(block.bindings.len());
    for mut binding in std::mem::take(&mut block.bindings) {
        rewrite_nested(
            &mut binding.operation,
            candidate,
            shape,
            capture_bindings,
            types,
            ids,
        );
        rewrite_atoms(&mut binding.operation, |atom| {
            if let Some(ty) = atom.binding().and_then(|binding| types.get(&binding)) {
                atom.ty = ty.clone();
            } else if atom.kind == AtomKind::Reference(Reference::SelfClosure(candidate.host)) {
                atom.ty = candidate.host_closure_type.clone();
            }
        });
        if let Operation::MakeClosure { function, captures } = &mut binding.operation
            && *function == candidate.target
        {
            captures.clear();
            set_pattern_type(&mut binding.pattern, &shape.closure_type, types);
        }
        if let Operation::MakeClosure { function, .. } = &binding.operation
            && *function == candidate.host
        {
            set_pattern_type(&mut binding.pattern, &candidate.host_closure_type, types);
        }
        if let Some(nested) = &candidate.nested {
            super::nested::extend_creator(&mut binding.operation, nested, capture_bindings, ids);
        }

        let mut capture_products = Vec::new();
        rewrite_atoms(&mut binding.operation, |atom| {
            if let Some(captures) = candidate.replacements.get(&atom.id) {
                let id = ids.value();
                capture_products.push(Binding {
                    pattern: Pattern::Binding {
                        id,
                        ty: candidate.capture_type.clone(),
                    },
                    operation: Operation::Product(ids.copy_atoms(captures)),
                    span: atom.span,
                });
                *atom = Atom {
                    id: ids.atom(),
                    kind: AtomKind::Reference(Reference::Binding(id)),
                    ty: candidate.capture_type.clone(),
                    span: atom.span,
                };
            } else if candidate.forwarded.contains(&atom.id) {
                let id = ids.value();
                capture_products.push(Binding {
                    pattern: Pattern::Binding {
                        id,
                        ty: candidate.capture_type.clone(),
                    },
                    operation: Operation::Product(
                        capture_bindings
                            .iter()
                            .map(|(binding, ty)| Atom {
                                id: ids.atom(),
                                kind: AtomKind::Reference(Reference::Binding(*binding)),
                                ty: ty.clone(),
                                span: atom.span,
                            })
                            .collect(),
                    ),
                    span: atom.span,
                });
                *atom = Atom {
                    id: ids.atom(),
                    kind: AtomKind::Reference(Reference::Binding(id)),
                    ty: candidate.capture_type.clone(),
                    span: atom.span,
                };
            }
        });
        output.extend(capture_products);

        if let Operation::Call { callee, argument } = &mut binding.operation
            && candidate.direct_calls.contains(&callee.id)
        {
            let mut elements = capture_bindings
                .iter()
                .map(|(id, ty)| Atom {
                    id: ids.atom(),
                    kind: AtomKind::Reference(Reference::Binding(*id)),
                    ty: ty.clone(),
                    span: argument.span,
                })
                .collect::<Vec<_>>();
            elements.push(argument.clone());
            callee.ty = shape.closure_type.clone();
            let packed = ids.value();
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

        rewrite_pattern(&mut binding.pattern, types);
        if let Operation::Product(elements) = &binding.operation {
            let ty = Type::Product(
                elements
                    .iter()
                    .map(|element| element.ty.clone())
                    .collect::<Vec<_>>()
                    .into(),
            );
            set_pattern_type(&mut binding.pattern, &ty, types);
        }
        output.push(binding);
    }
    block.bindings = output;
    if let Some(ty) = block
        .result
        .binding()
        .and_then(|binding| types.get(&binding))
    {
        block.result.ty = ty.clone();
    }
}

fn rewrite_nested(
    operation: &mut Operation,
    candidate: &Candidate,
    shape: &lambda_lift::LiftedShape,
    capture_bindings: &[(ValueId, Type)],
    types: &mut HashMap<ValueId, Type>,
    ids: &mut Identities,
) {
    match operation {
        Operation::Case { arms, .. } => {
            for arm in arms {
                rewrite_block(
                    &mut arm.value,
                    candidate,
                    shape,
                    capture_bindings,
                    types,
                    ids,
                );
            }
        }
        Operation::PrimitiveBranch {
            otherwise, then, ..
        } => {
            rewrite_block(otherwise, candidate, shape, capture_bindings, types, ids);
            rewrite_block(then, candidate, shape, capture_bindings, types, ids);
        }
        _ => {}
    }
}

fn rewrite_pattern(pattern: &mut Pattern, types: &mut HashMap<ValueId, Type>) -> Type {
    match pattern {
        Pattern::Binding { id, ty } => {
            if let Some(replacement) = types.get(id) {
                *ty = replacement.clone();
            }
            ty.clone()
        }
        Pattern::Wildcard { ty, .. } => ty.clone(),
        Pattern::Product { elements, ty, .. } => {
            *ty = Type::Product(
                elements
                    .iter_mut()
                    .map(|element| rewrite_pattern(element, types))
                    .collect::<Vec<_>>()
                    .into(),
            );
            ty.clone()
        }
    }
}

fn set_pattern_type(pattern: &mut Pattern, replacement: &Type, types: &mut HashMap<ValueId, Type>) {
    match pattern {
        Pattern::Binding { id, ty } => {
            *ty = replacement.clone();
            types.insert(*id, replacement.clone());
        }
        Pattern::Wildcard { ty, .. } | Pattern::Product { ty, .. } => {
            *ty = replacement.clone();
        }
    }
}

fn rewrite_top_level_patterns(
    bindings: &mut [crate::closure::ast::TopLevelBinding],
    sought: ValueId,
    replacement: &Type,
) {
    fn rewrite(pattern: &mut TopLevelPattern, sought: ValueId, replacement: &Type) {
        match pattern {
            TopLevelPattern::Binding { id, ty, .. } if *id == sought => *ty = replacement.clone(),
            TopLevelPattern::Product { elements, .. } => {
                for element in elements {
                    rewrite(element, sought, replacement);
                }
            }
            _ => {}
        }
    }
    for binding in bindings {
        rewrite(&mut binding.pattern, sought, replacement);
    }
}

fn rewrite_atoms(operation: &mut Operation, mut visit: impl FnMut(&mut Atom)) {
    match operation {
        Operation::Atom(atom)
        | Operation::Goto { value: atom, .. }
        | Operation::SymbolLength { value: atom }
        | Operation::SymbolAt { argument: atom }
        | Operation::ExternalCall { argument: atom, .. }
        | Operation::NumericConversion { operand: atom }
        | Operation::SumInjection { value: atom, .. }
        | Operation::PrimitiveUnary { operand: atom, .. } => visit(atom),
        Operation::MakeClosure { captures, .. }
        | Operation::Product(captures)
        | Operation::Memory {
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
