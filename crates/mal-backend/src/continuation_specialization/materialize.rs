//! Conversion of symbolic fusion results back into closure-program values.

use mal_syntax::source::Span;

use crate::closure::ast::{Atom, AtomKind, Binding, Block, CaseArm, Operation, Pattern, Reference};
use crate::closure::rewrite::Identities;

use super::symbolic::{Choice, EvalBlock, Value};

pub(super) fn value(
    symbolic: Value,
    output: &mut Vec<Binding>,
    ids: &mut Identities,
    fallback_span: Span,
) -> Option<Atom> {
    match symbolic {
        Value::Bound(id, ty) => Some(Atom {
            id: ids.atom(),
            kind: AtomKind::Reference(Reference::Binding(id)),
            ty,
            span: fallback_span,
        }),
        Value::Literal(kind, ty, span) => Some(Atom {
            id: ids.atom(),
            kind,
            ty,
            span,
        }),
        Value::Product(elements, ty, span) => {
            let elements = elements
                .into_iter()
                .map(|element| value(element, output, ids, fallback_span))
                .collect::<Option<Vec<_>>>()?;
            let id = ids.value();
            output.push(Binding {
                pattern: Pattern::Binding { id, ty: ty.clone() },
                operation: Operation::Product(elements),
                span,
            });
            Some(Atom {
                id: ids.atom(),
                kind: AtomKind::Reference(Reference::Binding(id)),
                ty,
                span,
            })
        }
        Value::Choice(choice) => choice_value(*choice, output, ids, fallback_span),
        Value::Closure { .. } => None,
    }
}

fn choice_value(
    choice: Choice,
    output: &mut Vec<Binding>,
    ids: &mut Identities,
    fallback_span: Span,
) -> Option<Atom> {
    let (operation, ty, span) = match choice {
        Choice::Case {
            scrutinee,
            arms,
            ty,
            span,
        } => {
            let arms = arms
                .into_iter()
                .map(|arm| {
                    Some(CaseArm {
                        index: arm.index,
                        pattern: arm.pattern,
                        value: finish(arm.value, ids, fallback_span)?,
                        span: arm.span,
                    })
                })
                .collect::<Option<Vec<_>>>()?;
            (Operation::Case { scrutinee, arms }, ty, span)
        }
        Choice::Branch {
            operator,
            left,
            right,
            otherwise,
            then,
            ty,
            span,
        } => (
            Operation::PrimitiveBranch {
                operator,
                left,
                right,
                otherwise: Box::new(finish(*otherwise, ids, fallback_span)?),
                then: Box::new(finish(*then, ids, fallback_span)?),
            },
            ty,
            span,
        ),
    };
    let id = ids.value();
    output.push(Binding {
        pattern: Pattern::Binding { id, ty: ty.clone() },
        operation,
        span,
    });
    Some(Atom {
        id: ids.atom(),
        kind: AtomKind::Reference(Reference::Binding(id)),
        ty,
        span,
    })
}

fn finish(mut block: EvalBlock, ids: &mut Identities, fallback_span: Span) -> Option<Block> {
    let result = value(block.value, &mut block.bindings, ids, fallback_span)?;
    Some(Block {
        bindings: block.bindings,
        result,
        span: block.span,
    })
}
