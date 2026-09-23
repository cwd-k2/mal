use crate::ast::{BinaryOperator, UnaryOperator};
use crate::check::ast as checked;
use crate::resolve::ast::{FALSE_VALUE, TRUE_VALUE};
use crate::source::Span;
use std::collections::HashMap;

pub mod ast;
mod bool;
mod buffer;
mod completion;
mod expression;
mod external;
mod interface;
mod lambda;
mod pattern;
mod primitive;

use self::bool::bool_type;
pub use self::interface::lower_interface;
use self::primitive::lower_binary_primitive;

use self::ast::{
    Binding, Capture, CaseArm, Expression, ExpressionKind, Lambda, LambdaKind, Parameter, Pattern,
    Program, TopLevelBinding, UnaryPrimitive, ValueId,
};

pub fn lower(program: &checked::MonomorphicProgram) -> Program {
    Lowerer::new().lower_program(program.program())
}

struct Lowerer {
    next_temporary: u32,
    next_lambda: u32,
    joins: Vec<ast::Join>,
    result_targets: HashMap<crate::resolve::ast::ValueId, ast::JoinId>,
}

impl Lowerer {
    fn new() -> Self {
        Self {
            next_temporary: 0,
            next_lambda: 0,
            joins: Vec::new(),
            result_targets: HashMap::new(),
        }
    }

    fn lower_program(&mut self, program: &checked::Program) -> Program {
        self.next_lambda = crate::check::next_lambda_identity(program);
        let mut bindings = program
            .items
            .iter()
            .filter_map(|item| match &item.kind {
                checked::TopItem::ExternalOperation {
                    id,
                    binding,
                    lambda_id,
                    parameter,
                    result,
                    ..
                } => Some(self.lower_external_operation(
                    *id, binding, *lambda_id, parameter, result, item.span,
                )),
                _ => None,
            })
            .collect::<Vec<_>>();
        bindings.extend(program.items.iter().filter_map(|item| match &item.kind {
            checked::TopItem::Binding(binding) => Some(self.lower_top_level_binding(binding)),
            _ => None,
        }));
        Program {
            interface: lower_interface(program),
            bindings,
            entry: program.entry.map(|entry| ast::EntryPoint {
                binding: ValueId::Source(entry.binding),
                parameter: entry.parameter,
            }),
            span: program.span,
        }
    }

    fn lower_top_level_binding(&mut self, binding: &checked::Binding) -> TopLevelBinding {
        let pattern = self.lower_top_level_pattern(&binding.pattern);
        TopLevelBinding {
            pattern,
            value: self.lower_expression(&binding.value),
            span: binding.span,
        }
    }

    fn lower_binding(&mut self, binding: &checked::Binding) -> Binding {
        Binding {
            pattern: self.lower_pattern(&binding.pattern),
            value: self.lower_expression(&binding.value),
            span: binding.span,
        }
    }

    fn case(
        &self,
        scrutinee: Expression,
        arms: Vec<CaseArm>,
        ty: checked::Type,
        span: Span,
    ) -> Expression {
        Expression {
            kind: ExpressionKind::Case {
                scrutinee: Box::new(scrutinee),
                arms,
            },
            ty,
            span,
        }
    }

    fn wildcard_arm(&self, index: usize, value: Expression, span: Span) -> CaseArm {
        CaseArm {
            index,
            pattern: Pattern::Wildcard {
                ty: checked::Type::Unit,
                span,
            },
            value,
            span,
        }
    }

    fn reference(&self, id: ValueId, ty: checked::Type, span: Span) -> Expression {
        Expression {
            kind: ExpressionKind::Reference(id),
            ty,
            span,
        }
    }

    fn temporary_let(
        &self,
        id: ValueId,
        value: Expression,
        body: Expression,
        span: Span,
    ) -> Expression {
        let ty = body.ty.clone();
        Expression {
            kind: ExpressionKind::Let {
                binding: Box::new(Binding {
                    pattern: Pattern::Binding {
                        id,
                        ty: value.ty.clone(),
                    },
                    value,
                    span,
                }),
                body: Box::new(body),
            },
            ty,
            span,
        }
    }

    fn temporary(&mut self) -> ValueId {
        let id = ValueId::Temporary(self.next_temporary);
        self.next_temporary += 1;
        id
    }
}
