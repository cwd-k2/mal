//! Lambda lifting for non-recursive locally created closures whose aliases are used only as direct callees.

use std::collections::{HashMap, HashSet};

use mal_frontend::check::ast::Type;

use crate::anf::ast::ValueId;
use crate::closure::ast::{
    Atom, AtomKind, Binding, Block, FunctionId, Operation, Parameter, Pattern, Program, Reference,
};
use crate::closure::rewrite::{
    Identities,
    walk::{self, Visitor},
};

mod lift;
mod uses;

use lift::rewrite_block;
pub(super) use lift::{LiftedShape, lift_functions};
use uses::{Creator, Uses, collect_definitions, collect_uses};

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
