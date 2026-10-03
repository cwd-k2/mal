//! Symbolic values and branch results carried while fusing an admitted slice.

use std::collections::HashMap;

use mal_frontend::check::ast::Type;
use mal_syntax::source::Span;

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomKind, Binding, FunctionId, Pattern};

#[derive(Clone, Default)]
pub(super) struct Environment {
    pub(super) values: HashMap<ValueId, Value>,
    pub(super) captures: Vec<Value>,
}

#[derive(Clone)]
pub(super) enum Value {
    Bound(ValueId, Type),
    Literal(AtomKind, Type, Span),
    Closure {
        function: FunctionId,
        captures: Vec<Value>,
        ty: Type,
    },
    Product(Vec<Value>, Type, Span),
    Choice(Box<Choice>),
}

impl Value {
    pub(super) fn ty(&self) -> &Type {
        match self {
            Self::Bound(_, ty)
            | Self::Literal(_, ty, _)
            | Self::Closure { ty, .. }
            | Self::Product(_, ty, _) => ty,
            Self::Choice(choice) => choice.ty(),
        }
    }
}

pub(super) fn transport(value: &Value, output: &mut Vec<Value>) -> Option<()> {
    match value {
        Value::Bound(..) | Value::Literal(..) => output.push(value.clone()),
        Value::Closure { captures, .. } => {
            for capture in captures {
                transport(capture, output)?;
            }
        }
        Value::Product(elements, ..) => {
            for element in elements {
                transport(element, output)?;
            }
        }
        Value::Choice(_) => return None,
    }
    Some(())
}

pub(super) fn rebuild_context(template: &[Value], inputs: &[Value]) -> Option<Vec<Value>> {
    fn rebuild(template: &Value, inputs: &mut impl Iterator<Item = Value>) -> Option<Value> {
        Some(match template {
            Value::Bound(..) | Value::Literal(..) => inputs.next()?,
            Value::Closure {
                function,
                captures,
                ty,
            } => Value::Closure {
                function: *function,
                captures: captures
                    .iter()
                    .map(|capture| rebuild(capture, inputs))
                    .collect::<Option<_>>()?,
                ty: ty.clone(),
            },
            Value::Product(elements, ty, span) => Value::Product(
                elements
                    .iter()
                    .map(|element| rebuild(element, inputs))
                    .collect::<Option<_>>()?,
                ty.clone(),
                *span,
            ),
            Value::Choice(_) => return None,
        })
    }

    let mut inputs = inputs.iter().cloned();
    let rebuilt = template
        .iter()
        .map(|value| rebuild(value, &mut inputs))
        .collect::<Option<Vec<_>>>()?;
    inputs.next().is_none().then_some(rebuilt)
}

pub(super) fn same_context(left: &[Value], right: &[Value]) -> bool {
    fn same(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Bound(_, left), Value::Bound(_, right))
            | (Value::Bound(_, left), Value::Literal(_, right, _))
            | (Value::Literal(_, left, _), Value::Bound(_, right))
            | (Value::Literal(_, left, _), Value::Literal(_, right, _)) => left == right,
            (
                Value::Closure {
                    function: left_function,
                    captures: left_captures,
                    ..
                },
                Value::Closure {
                    function: right_function,
                    captures: right_captures,
                    ..
                },
            ) => left_function == right_function && same_context(left_captures, right_captures),
            (Value::Product(left, left_ty, _), Value::Product(right, right_ty, _)) => {
                left_ty == right_ty && same_context(left, right)
            }
            _ => false,
        }
    }

    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| same(left, right))
}

#[derive(Clone)]
pub(super) enum Choice {
    Case {
        scrutinee: Atom,
        arms: Vec<ChoiceArm>,
        ty: Type,
        span: Span,
    },
    Branch {
        operator: crate::core::ast::BinaryPrimitive,
        left: Atom,
        right: Atom,
        otherwise: Box<EvalBlock>,
        then: Box<EvalBlock>,
        ty: Type,
        span: Span,
    },
}

impl Choice {
    pub(super) fn ty(&self) -> &Type {
        match self {
            Self::Case { ty, .. } | Self::Branch { ty, .. } => ty,
        }
    }
}

#[derive(Clone)]
pub(super) struct ChoiceArm {
    pub(super) index: usize,
    pub(super) pattern: Pattern,
    pub(super) value: EvalBlock,
    pub(super) span: Span,
}

#[derive(Clone)]
pub(super) struct EvalBlock {
    pub(super) bindings: Vec<Binding>,
    pub(super) value: Value,
    pub(super) span: Span,
}
