//! Lifting the selected functions: captures become leading parameters and closure construction is rewritten.

use super::*;

pub(in crate::call_pattern) struct LiftedShape {
    pub(in crate::call_pattern) parameter_type: Type,
    pub(in crate::call_pattern) closure_type: Type,
}

pub(in crate::call_pattern) fn lift_functions(
    program: &mut Program,
    selected: &HashSet<FunctionId>,
    ids: &mut Identities,
) -> HashMap<FunctionId, LiftedShape> {
    let mut lifted = HashMap::new();
    for function in &mut program.functions {
        if !selected.contains(&function.id) {
            continue;
        }
        let capture_types = function
            .captures
            .iter()
            .map(|capture| capture.ty.clone())
            .collect::<Vec<_>>();
        let mut fields = capture_types.clone();
        fields.push(function.parameter.ty.clone());
        let parameter_type = Type::Product(fields.into());
        let parameter = ids.value();
        let capture_bindings = capture_types
            .iter()
            .map(|_| ids.value())
            .collect::<Vec<_>>();
        let old_parameter = function.parameter.clone();
        let span = old_parameter.span;
        let mut elements = capture_bindings
            .iter()
            .zip(&capture_types)
            .map(|(id, ty)| Pattern::Binding {
                id: *id,
                ty: ty.clone(),
            })
            .collect::<Vec<_>>();
        elements.push(match old_parameter.binding {
            Some(id) => Pattern::Binding {
                id,
                ty: old_parameter.ty.clone(),
            },
            None => Pattern::Wildcard {
                ty: old_parameter.ty.clone(),
                span,
            },
        });
        function.body.bindings.insert(
            0,
            Binding {
                pattern: Pattern::Product {
                    elements,
                    ty: parameter_type.clone(),
                    span,
                },
                operation: Operation::Atom(Atom {
                    id: ids.atom(),
                    kind: AtomKind::Reference(Reference::Binding(parameter)),
                    ty: parameter_type.clone(),
                    span,
                }),
                span,
            },
        );
        function.parameter = Parameter {
            binding: Some(parameter),
            ty: parameter_type.clone(),
            span,
        };
        walk::function(
            function,
            &mut ReplaceCaptures {
                bindings: &capture_bindings,
            },
        );
        function.captures.clear();
        lifted.insert(
            function.id,
            LiftedShape {
                parameter_type: parameter_type.clone(),
                closure_type: Type::Function {
                    parameter: parameter_type.into(),
                    result: function.body.result.ty.clone().into(),
                },
            },
        );
    }
    lifted
}

struct ReplaceCaptures<'a> {
    bindings: &'a [ValueId],
}

impl Visitor for ReplaceCaptures<'_> {
    fn atom(&mut self, atom: &mut Atom) {
        if let AtomKind::Reference(Reference::Capture(index)) = atom.kind {
            atom.kind = AtomKind::Reference(Reference::Binding(self.bindings[index]));
        }
    }
}

pub(super) fn rewrite_block(
    block: &mut Block,
    origins: &HashMap<ValueId, ValueId>,
    creators: &HashMap<ValueId, Creator>,
    lifted: &HashMap<FunctionId, LiftedShape>,
    closure_types: &HashMap<ValueId, Type>,
    ids: &mut Identities,
) {
    let mut rewritten = Vec::with_capacity(block.bindings.len());
    for mut binding in std::mem::take(&mut block.bindings) {
        rewrite_nested(
            &mut binding.operation,
            origins,
            creators,
            lifted,
            closure_types,
            ids,
        );
        rewrite_operation_atoms(&mut binding.operation, closure_types);
        rewrite_pattern_type(&mut binding.pattern, closure_types);
        if let Operation::MakeClosure { function, captures } = &mut binding.operation
            && lifted.contains_key(function)
        {
            captures.clear();
        }
        if let Operation::Call { callee, argument } = &mut binding.operation
            && let AtomKind::Reference(Reference::Binding(callee_binding)) = callee.kind
            && let Some(creator_id) = origins.get(&callee_binding)
            && let Some(creator) = creators.get(creator_id)
            && let Some(shape) = lifted.get(&creator.function)
        {
            let argument_id = ids.value();
            let mut elements = ids.copy_atoms(&creator.captures);
            elements.push(argument.clone());
            rewritten.push(Binding {
                pattern: Pattern::Binding {
                    id: argument_id,
                    ty: shape.parameter_type.clone(),
                },
                operation: Operation::Product(elements),
                span: argument.span,
            });
            *argument = Atom {
                id: ids.atom(),
                kind: AtomKind::Reference(Reference::Binding(argument_id)),
                ty: shape.parameter_type.clone(),
                span: argument.span,
            };
        }
        rewritten.push(binding);
    }
    block.bindings = rewritten;
    rewrite_atom_type(&mut block.result, closure_types);
}

fn rewrite_nested(
    operation: &mut Operation,
    origins: &HashMap<ValueId, ValueId>,
    creators: &HashMap<ValueId, Creator>,
    lifted: &HashMap<FunctionId, LiftedShape>,
    closure_types: &HashMap<ValueId, Type>,
    ids: &mut Identities,
) {
    match operation {
        Operation::Case { arms, .. } => {
            for arm in arms {
                rewrite_block(
                    &mut arm.value,
                    origins,
                    creators,
                    lifted,
                    closure_types,
                    ids,
                );
            }
        }
        Operation::PrimitiveBranch {
            otherwise, then, ..
        } => {
            rewrite_block(otherwise, origins, creators, lifted, closure_types, ids);
            rewrite_block(then, origins, creators, lifted, closure_types, ids);
        }
        _ => {}
    }
}

fn rewrite_pattern_type(pattern: &mut Pattern, closure_types: &HashMap<ValueId, Type>) {
    match pattern {
        Pattern::Binding { id, ty } => {
            if let Some(replacement) = closure_types.get(id) {
                *ty = replacement.clone();
            }
        }
        Pattern::Product { elements, .. } => {
            for element in elements {
                rewrite_pattern_type(element, closure_types);
            }
        }
        Pattern::Wildcard { .. } => {}
    }
}

fn rewrite_operation_atoms(operation: &mut Operation, closure_types: &HashMap<ValueId, Type>) {
    operation::for_atoms_mut(operation, |atom| rewrite_atom_type(atom, closure_types));
}

fn rewrite_atom_type(atom: &mut Atom, closure_types: &HashMap<ValueId, Type>) {
    if let AtomKind::Reference(Reference::Binding(binding)) = atom.kind
        && let Some(replacement) = closure_types.get(&binding)
    {
        atom.ty = replacement.clone();
    }
}
