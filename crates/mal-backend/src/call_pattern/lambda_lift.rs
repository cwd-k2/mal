//! Lambda lifting for locally created closures used only as direct callees.

use std::collections::{HashMap, HashSet};

use mal_frontend::check::ast::Type;

use crate::anf::ast::ValueId;
use crate::closure::ast::{
    Atom, AtomKind, Binding, Block, FunctionId, Operation, Parameter, Pattern, Program, Reference,
};

use super::ids::Identities;
use super::walk::{self, Visitor};

mod operation;

#[derive(Clone)]
struct Creator {
    function: FunctionId,
    captures: Vec<Atom>,
}

#[derive(Default)]
struct Uses {
    direct: bool,
    other: bool,
}

pub(super) fn direct_closures(program: &mut Program) {
    let mut origins = HashMap::new();
    let mut creators = HashMap::new();
    let mut disqualified = HashSet::new();
    for binding in &program.bindings {
        collect_definitions(
            &binding.value,
            &mut origins,
            &mut creators,
            &mut disqualified,
        );
    }
    for function in &program.functions {
        collect_definitions(
            &function.body,
            &mut origins,
            &mut creators,
            &mut disqualified,
        );
        for join in &function.joins {
            collect_definitions(&join.body, &mut origins, &mut creators, &mut disqualified);
        }
    }

    let mut uses = creators
        .keys()
        .map(|creator| (*creator, Uses::default()))
        .collect::<HashMap<_, _>>();
    for binding in &program.bindings {
        collect_uses(&binding.value, &origins, &mut uses, &mut disqualified);
    }
    for function in &program.functions {
        collect_uses(&function.body, &origins, &mut uses, &mut disqualified);
        for join in &function.joins {
            collect_uses(&join.body, &origins, &mut uses, &mut disqualified);
        }
    }

    let mut functions = creators
        .iter()
        .filter_map(|(creator, value)| {
            let uses = &uses[creator];
            (!value.captures.is_empty() && uses.direct && !uses.other).then_some(value.function)
        })
        .collect::<HashSet<_>>();
    functions.retain(|function| {
        !disqualified.contains(function)
            && creators
                .iter()
                .filter(|(_, creator)| creator.function == *function)
                .all(|(creator, _)| {
                    let uses = &uses[creator];
                    uses.direct && !uses.other
                })
    });
    if functions.is_empty() {
        return;
    }

    let mut ids = Identities::after(program);
    let lifted = lift_functions(program, &functions, &mut ids);
    let closure_types = origins
        .iter()
        .filter_map(|(binding, creator)| {
            let function = creators.get(creator)?.function;
            lifted
                .get(&function)
                .map(|shape| (*binding, shape.closure_type.clone()))
        })
        .collect::<HashMap<_, _>>();
    for binding in &mut program.bindings {
        rewrite_block(
            &mut binding.value,
            &origins,
            &creators,
            &lifted,
            &closure_types,
            &mut ids,
        );
    }
    for function in &mut program.functions {
        rewrite_block(
            &mut function.body,
            &origins,
            &creators,
            &lifted,
            &closure_types,
            &mut ids,
        );
        for join in &mut function.joins {
            rewrite_block(
                &mut join.body,
                &origins,
                &creators,
                &lifted,
                &closure_types,
                &mut ids,
            );
        }
    }
}

fn collect_definitions(
    block: &Block,
    origins: &mut HashMap<ValueId, ValueId>,
    creators: &mut HashMap<ValueId, Creator>,
    disqualified: &mut HashSet<FunctionId>,
) {
    for binding in &block.bindings {
        match (&binding.pattern, &binding.operation) {
            (Pattern::Binding { id, .. }, Operation::MakeClosure { function, captures }) => {
                origins.insert(*id, *id);
                creators.insert(
                    *id,
                    Creator {
                        function: *function,
                        captures: captures.clone(),
                    },
                );
            }
            (
                Pattern::Binding { id, .. },
                Operation::Atom(Atom {
                    kind: AtomKind::Reference(Reference::Binding(source)),
                    ..
                }),
            ) if origins.contains_key(source) => {
                origins.insert(*id, origins[source]);
            }
            (_, Operation::MakeClosure { function, captures }) if !captures.is_empty() => {
                disqualified.insert(*function);
            }
            _ => {}
        }
        operation::for_nested_blocks(&binding.operation, |nested| {
            collect_definitions(nested, origins, creators, disqualified)
        });
    }
}

fn collect_uses(
    block: &Block,
    origins: &HashMap<ValueId, ValueId>,
    uses: &mut HashMap<ValueId, Uses>,
    disqualified: &mut HashSet<FunctionId>,
) {
    atom_use(&block.result, false, origins, uses, disqualified);
    for binding in &block.bindings {
        let alias = matches!(
            (&binding.pattern, &binding.operation),
            (
                Pattern::Binding { id, .. },
                Operation::Atom(Atom {
                    kind: AtomKind::Reference(Reference::Binding(source)),
                    ..
                })
            ) if origins.get(id) == origins.get(source) && origins.contains_key(id)
        );
        if !alias {
            operation_uses(&binding.operation, origins, uses, disqualified);
        }
    }
}

fn operation_uses(
    operation: &Operation,
    origins: &HashMap<ValueId, ValueId>,
    uses: &mut HashMap<ValueId, Uses>,
    disqualified: &mut HashSet<FunctionId>,
) {
    match operation {
        Operation::Call { callee, argument } => {
            atom_use(callee, true, origins, uses, disqualified);
            atom_use(argument, false, origins, uses, disqualified);
        }
        Operation::Case { scrutinee, arms } => {
            atom_use(scrutinee, false, origins, uses, disqualified);
            for arm in arms {
                collect_uses(&arm.value, origins, uses, disqualified);
            }
        }
        Operation::PrimitiveBranch {
            left,
            right,
            otherwise,
            then,
            ..
        } => {
            atom_use(left, false, origins, uses, disqualified);
            atom_use(right, false, origins, uses, disqualified);
            collect_uses(otherwise, origins, uses, disqualified);
            collect_uses(then, origins, uses, disqualified);
        }
        other => operation::for_atoms(other, |atom| {
            atom_use(atom, false, origins, uses, disqualified)
        }),
    }
}

fn atom_use(
    atom: &Atom,
    direct: bool,
    origins: &HashMap<ValueId, ValueId>,
    uses: &mut HashMap<ValueId, Uses>,
    disqualified: &mut HashSet<FunctionId>,
) {
    match atom.kind {
        AtomKind::Reference(Reference::Binding(binding)) => {
            if let Some(use_) = origins
                .get(&binding)
                .and_then(|creator| uses.get_mut(creator))
            {
                if direct {
                    use_.direct = true;
                } else {
                    use_.other = true;
                }
            }
        }
        AtomKind::Reference(Reference::SelfClosure(function)) => {
            disqualified.insert(function);
        }
        _ => {}
    }
}

struct LiftedShape {
    parameter_type: Type,
    closure_type: Type,
}

fn lift_functions(
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

fn rewrite_block(
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
            let mut elements = creator.captures.clone();
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
