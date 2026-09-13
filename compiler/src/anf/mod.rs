use crate::core::ast as core;

pub mod ast;

use self::ast::{
    Atom, AtomKind, Binding, Block, Capture, CaseArm, Lambda, Operation, Parameter, Pattern,
    Program, TopLevelBinding, TopLevelPattern, ValueId,
};

pub fn lower(program: &core::Program) -> Program {
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
            span: program.span,
        }
    }

    fn lower_top_level_binding(&mut self, binding: &core::TopLevelBinding) -> TopLevelBinding {
        TopLevelBinding {
            pattern: match &binding.pattern {
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
            },
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

    fn lower_expression(&mut self, expression: &core::Expression) -> Block {
        match &expression.kind {
            core::ExpressionKind::Reference(id) => {
                self.atom_block(expression, AtomKind::Reference(self.core_id(*id)))
            }
            core::ExpressionKind::Integer(value) => {
                self.atom_block(expression, AtomKind::Integer(*value))
            }
            core::ExpressionKind::Float(bits) => {
                self.atom_block(expression, AtomKind::Float(*bits))
            }
            core::ExpressionKind::Symbol(value) => {
                self.atom_block(expression, AtomKind::Symbol(value.clone()))
            }
            core::ExpressionKind::StorageSize(ty) => {
                self.atom_block(expression, AtomKind::StorageSize(ty.clone()))
            }
            core::ExpressionKind::Unit => self.atom_block(expression, AtomKind::Unit),
            core::ExpressionKind::Product(elements) => {
                let mut builder = ExpressionBuilder::default();
                let elements = elements
                    .iter()
                    .map(|element| builder.append(self, element))
                    .collect();
                builder.finish(self, expression, Operation::Product(elements))
            }
            core::ExpressionKind::Let { binding, body } => {
                self.lower_let_chain(binding, body, expression)
            }
            core::ExpressionKind::Lambda(lambda) => {
                let lambda = Lambda {
                    id: lambda.id,
                    self_binding: lambda.self_binding.map(|id| self.core_id(id)),
                    captures: lambda
                        .captures
                        .iter()
                        .map(|capture| Capture {
                            source: self.core_id(capture.source),
                            binding: self.core_id(capture.binding),
                            ty: capture.ty.clone(),
                        })
                        .collect(),
                    parameter: Parameter {
                        binding: lambda.parameter.binding.map(|id| self.core_id(id)),
                        ty: lambda.parameter.ty.clone(),
                        span: lambda.parameter.span,
                    },
                    body: self.lower_expression(&lambda.body),
                };
                self.operation_block(expression, Operation::Lambda(lambda))
            }
            core::ExpressionKind::Call { callee, argument } => {
                let (mut builder, argument) = self.lower_operand(argument);
                let callee = builder.append(self, callee);
                builder.finish(self, expression, Operation::Call { callee, argument })
            }
            core::ExpressionKind::SymbolLength { value } => {
                let (builder, value) = self.lower_operand(value);
                builder.finish(self, expression, Operation::SymbolLength { value })
            }
            core::ExpressionKind::SymbolAt { argument } => {
                let (builder, argument) = self.lower_operand(argument);
                builder.finish(self, expression, Operation::SymbolAt { argument })
            }
            core::ExpressionKind::MemoryFunction { primitive } => self.operation_block(
                expression,
                Operation::MemoryFunction {
                    primitive: *primitive,
                },
            ),
            core::ExpressionKind::Memory {
                primitive,
                argument,
            } => {
                let (builder, argument) = self.lower_operand(argument);
                builder.finish(
                    self,
                    expression,
                    Operation::Memory {
                        primitive: *primitive,
                        argument,
                    },
                )
            }
            core::ExpressionKind::ExternalCall { id, argument } => {
                let (builder, argument) = self.lower_operand(argument);
                builder.finish(
                    self,
                    expression,
                    Operation::ExternalCall { id: *id, argument },
                )
            }
            core::ExpressionKind::NumericConversion { value } => {
                let (builder, operand) = self.lower_operand(value);
                builder.finish(self, expression, Operation::NumericConversion { operand })
            }
            core::ExpressionKind::SumInjection { index, value } => {
                let (builder, value) = self.lower_operand(value);
                builder.finish(
                    self,
                    expression,
                    Operation::SumInjection {
                        index: *index,
                        value,
                    },
                )
            }
            core::ExpressionKind::Case { scrutinee, arms } => {
                let (builder, scrutinee) = self.lower_operand(scrutinee);
                let arms = arms.iter().map(|arm| self.lower_case_arm(arm)).collect();
                builder.finish(self, expression, Operation::Case { scrutinee, arms })
            }
            core::ExpressionKind::PrimitiveBranch {
                operator,
                left,
                right,
                otherwise,
                then,
            } => {
                let (mut builder, left) = self.lower_operand(left);
                let right = builder.append(self, right);
                let otherwise = self.lower_expression(otherwise);
                let then = self.lower_expression(then);
                builder.finish(
                    self,
                    expression,
                    Operation::PrimitiveBranch {
                        operator: *operator,
                        left,
                        right,
                        otherwise: Box::new(otherwise),
                        then: Box::new(then),
                    },
                )
            }
            core::ExpressionKind::PrimitiveUnary { operator, operand } => {
                let (builder, operand) = self.lower_operand(operand);
                builder.finish(
                    self,
                    expression,
                    Operation::PrimitiveUnary {
                        operator: *operator,
                        operand,
                    },
                )
            }
            core::ExpressionKind::PrimitiveBinary {
                operator,
                left,
                right,
            } => {
                let (mut builder, left) = self.lower_operand(left);
                let right = builder.append(self, right);
                builder.finish(
                    self,
                    expression,
                    Operation::PrimitiveBinary {
                        operator: *operator,
                        left,
                        right,
                    },
                )
            }
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
