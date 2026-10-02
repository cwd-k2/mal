//! Core-to-ANF lowering with explicit left-to-right evaluation and fresh intermediate identities.

use crate::core::ast as core;

pub(crate) mod ast;
mod expression;

use self::ast::{
    Atom, AtomKind, Binding, Block, Capture, CaseArm, Lambda, Operation, Parameter, Pattern,
    Program, TopLevelBinding, TopLevelPattern, ValueId,
};

/// Linearizes nested core expressions while preserving the language's evaluation order and existing identities.
pub(crate) fn lower(program: &core::Program) -> Program {
    Lowerer::new().lower_program(program)
}

struct Lowerer {
    next_temporary: u32,
}

impl Lowerer {
    fn new() -> Self {
        Self { next_temporary: 0 }
    }

    fn lower_program(&mut self, program: &core::Program) -> Program {
        Program {
            interface: program.interface.clone(),
            bindings: program
                .bindings
                .iter()
                .map(|binding| self.lower_top_level_binding(binding))
                .collect(),
            entry: program.entry.map(|entry| ast::EntryPoint {
                binding: self.core_id(entry.binding),
                parameter: entry.parameter,
            }),
            span: program.span,
        }
    }

    fn lower_top_level_binding(&mut self, binding: &core::TopLevelBinding) -> TopLevelBinding {
        TopLevelBinding {
            pattern: self.lower_top_level_pattern(&binding.pattern),
            value: self.lower_expression(&binding.value),
            span: binding.span,
        }
    }

    fn lower_top_level_pattern(&self, pattern: &core::TopLevelPattern) -> TopLevelPattern {
        match pattern {
            core::TopLevelPattern::Binding { id, name, ty } => TopLevelPattern::Binding {
                id: self.core_id(*id),
                name: name.clone(),
                ty: ty.clone(),
            },
            core::TopLevelPattern::Wildcard { ty, span } => TopLevelPattern::Wildcard {
                ty: ty.clone(),
                span: *span,
            },
            core::TopLevelPattern::Product { elements, ty, span } => TopLevelPattern::Product {
                elements: elements
                    .iter()
                    .map(|element| self.lower_top_level_pattern(element))
                    .collect(),
                ty: ty.clone(),
                span: *span,
            },
        }
    }

    fn lower_let_chain(
        &mut self,
        binding: &core::Binding,
        body: &core::Expression,
        expression: &core::Expression,
    ) -> Block {
        let mut bindings = Vec::new();
        let mut current_binding = binding;
        let mut current_body = body;
        loop {
            let mut value = self.lower_expression(&current_binding.value);
            bindings.append(&mut value.bindings);
            bindings.push(Binding {
                pattern: self.lower_pattern(&current_binding.pattern),
                operation: Operation::Atom(value.result),
                span: current_binding.span,
            });
            match &current_body.kind {
                core::ExpressionKind::Let { binding, body } => {
                    current_binding = binding;
                    current_body = body;
                }
                _ => break,
            }
        }
        let mut lowered_body = self.lower_expression(current_body);
        bindings.append(&mut lowered_body.bindings);
        Block {
            bindings,
            result: lowered_body.result,
            span: expression.span,
        }
    }

    fn lower_case_arm(&mut self, arm: &core::CaseArm) -> CaseArm {
        CaseArm {
            index: arm.index,
            pattern: self.lower_pattern(&arm.pattern),
            value: self.lower_expression(&arm.value),
            span: arm.span,
        }
    }

    fn lower_join(&mut self, join: &core::Join) -> ast::Join {
        ast::Join {
            parameter: self.lower_pattern(&join.parameter),
            body: self.lower_expression(&join.body),
            span: join.span,
        }
    }

    fn lower_pattern(&self, pattern: &core::Pattern) -> Pattern {
        match pattern {
            core::Pattern::Binding { id, ty } => Pattern::Binding {
                id: self.core_id(*id),
                ty: ty.clone(),
            },
            core::Pattern::Wildcard { ty, span } => Pattern::Wildcard {
                ty: ty.clone(),
                span: *span,
            },
            core::Pattern::Product { elements, ty, span } => Pattern::Product {
                elements: elements
                    .iter()
                    .map(|element| self.lower_pattern(element))
                    .collect(),
                ty: ty.clone(),
                span: *span,
            },
        }
    }

    fn lower_operand(&mut self, expression: &core::Expression) -> (ExpressionBuilder, Atom) {
        let block = self.lower_expression(expression);
        (ExpressionBuilder::from(block.bindings), block.result)
    }

    fn atom_block(&self, expression: &core::Expression, kind: AtomKind) -> Block {
        Block {
            bindings: Vec::new(),
            result: Atom {
                kind,
                ty: expression.ty.clone(),
                span: expression.span,
            },
            span: expression.span,
        }
    }

    fn operation_block(&mut self, expression: &core::Expression, operation: Operation) -> Block {
        ExpressionBuilder::default().finish(self, expression, operation)
    }

    fn bind_operation(
        &mut self,
        bindings: &mut Vec<Binding>,
        expression: &core::Expression,
        operation: Operation,
    ) -> Atom {
        let id = ValueId::Temporary(self.next_temporary);
        self.next_temporary += 1;
        bindings.push(Binding {
            pattern: Pattern::Binding {
                id,
                ty: expression.ty.clone(),
            },
            operation,
            span: expression.span,
        });
        Atom {
            kind: AtomKind::Reference(id),
            ty: expression.ty.clone(),
            span: expression.span,
        }
    }

    fn core_id(&self, id: core::ValueId) -> ValueId {
        ValueId::Core(id)
    }
}

#[derive(Default)]
struct ExpressionBuilder {
    bindings: Vec<Binding>,
}

impl ExpressionBuilder {
    fn from(bindings: Vec<Binding>) -> Self {
        Self { bindings }
    }

    fn append(&mut self, lowerer: &mut Lowerer, expression: &core::Expression) -> Atom {
        let block = lowerer.lower_expression(expression);
        self.bindings.extend(block.bindings);
        block.result
    }

    fn finish(
        mut self,
        lowerer: &mut Lowerer,
        expression: &core::Expression,
        operation: Operation,
    ) -> Block {
        let result = lowerer.bind_operation(&mut self.bindings, expression, operation);
        Block {
            bindings: self.bindings,
            result,
            span: expression.span,
        }
    }
}
