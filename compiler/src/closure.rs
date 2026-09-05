use std::collections::HashMap;

use crate::anf::ast as anf;

pub mod ast;

use self::ast::{
    Atom, AtomKind, Binding, Block, EnvironmentField, ExternalOperation, ExternalType, Function,
    Operation, Parameter, Pattern, Program, Reference, TopLevelBinding, TopLevelPattern,
};

pub fn convert(program: &anf::Program) -> Program {
    Converter::new().convert_program(program)
}

struct Converter {
    functions: Vec<Function>,
}

impl Converter {
    fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }

    fn convert_program(mut self, program: &anf::Program) -> Program {
        let empty_environment = HashMap::new();
        let externals = program
            .externals
            .iter()
            .map(|external| ExternalOperation {
                id: external.id,
                name: external.name.clone(),
                parameter: external.parameter.clone(),
                result: external.result.clone(),
                span: external.span,
            })
            .collect();
        let bindings = program
            .bindings
            .iter()
            .map(|binding| self.convert_top_level_binding(binding, &empty_environment))
            .collect();
        Program {
            external_types: program
                .external_types
                .iter()
                .map(|external| ExternalType {
                    name: external.name.clone(),
                })
                .collect(),
            externals,
            bindings,
            functions: self.functions,
            span: program.span,
        }
    }

    fn convert_top_level_binding(
        &mut self,
        binding: &anf::TopLevelBinding,
        environment: &HashMap<anf::ValueId, Reference>,
    ) -> TopLevelBinding {
        TopLevelBinding {
            pattern: match &binding.pattern {
                anf::TopLevelPattern::Binding { id, name, ty } => TopLevelPattern::Binding {
                    id: *id,
                    name: name.clone(),
                    ty: ty.clone(),
                },
                anf::TopLevelPattern::Wildcard { ty, span } => TopLevelPattern::Wildcard {
                    ty: ty.clone(),
                    span: *span,
                },
                anf::TopLevelPattern::Product { elements, ty, span } => TopLevelPattern::Product {
                    elements: elements
                        .iter()
                        .map(|element| self.convert_top_level_pattern(element))
                        .collect(),
                    ty: ty.clone(),
                    span: *span,
                },
            },
            value: self.convert_block(&binding.value, environment),
            span: binding.span,
        }
    }

    fn convert_top_level_pattern(&self, pattern: &anf::TopLevelPattern) -> TopLevelPattern {
        match pattern {
            anf::TopLevelPattern::Binding { id, name, ty } => TopLevelPattern::Binding {
                id: *id,
                name: name.clone(),
                ty: ty.clone(),
            },
            anf::TopLevelPattern::Wildcard { ty, span } => TopLevelPattern::Wildcard {
                ty: ty.clone(),
                span: *span,
            },
            anf::TopLevelPattern::Product { elements, ty, span } => TopLevelPattern::Product {
                elements: elements
                    .iter()
                    .map(|element| self.convert_top_level_pattern(element))
                    .collect(),
                ty: ty.clone(),
                span: *span,
            },
        }
    }

    fn convert_block(
        &mut self,
        block: &anf::Block,
        environment: &HashMap<anf::ValueId, Reference>,
    ) -> Block {
        Block {
            bindings: block
                .bindings
                .iter()
                .map(|binding| self.convert_binding(binding, environment))
                .collect(),
            result: self.convert_atom(&block.result, environment),
            span: block.span,
        }
    }

    fn convert_binding(
        &mut self,
        binding: &anf::Binding,
        environment: &HashMap<anf::ValueId, Reference>,
    ) -> Binding {
        Binding {
            pattern: self.convert_pattern(&binding.pattern),
            operation: self.convert_operation(&binding.operation, binding.span, environment),
            span: binding.span,
        }
    }

    fn convert_operation(
        &mut self,
        operation: &anf::Operation,
        span: crate::source::Span,
        environment: &HashMap<anf::ValueId, Reference>,
    ) -> Operation {
        match operation {
            anf::Operation::Atom(atom) => Operation::Atom(self.convert_atom(atom, environment)),
            anf::Operation::Lambda(lambda) => {
                let captures = lambda
                    .captures
                    .iter()
                    .map(|capture| {
                        self.reference_atom(capture.source, capture.ty.clone(), span, environment)
                    })
                    .collect();
                self.lift_function(lambda);
                Operation::MakeClosure {
                    function: lambda.id,
                    captures,
                }
            }
            anf::Operation::Call { callee, argument } => Operation::Call {
                callee: self.convert_atom(callee, environment),
                argument: self.convert_atom(argument, environment),
            },
            anf::Operation::StringLength { value } => Operation::StringLength {
                value: self.convert_atom(value, environment),
            },
            anf::Operation::StringAt { argument } => Operation::StringAt {
                argument: self.convert_atom(argument, environment),
            },
            anf::Operation::ExternalCall { id, argument } => Operation::ExternalCall {
                id: *id,
                argument: self.convert_atom(argument, environment),
            },
            anf::Operation::IntegerConversion { operand } => Operation::IntegerConversion {
                operand: self.convert_atom(operand, environment),
            },
            anf::Operation::Product(elements) => Operation::Product(
                elements
                    .iter()
                    .map(|element| self.convert_atom(element, environment))
                    .collect(),
            ),
            anf::Operation::SumInjection { index, value } => Operation::SumInjection {
                index: *index,
                value: self.convert_atom(value, environment),
            },
            anf::Operation::Case { scrutinee, arms } => Operation::Case {
                scrutinee: self.convert_atom(scrutinee, environment),
                arms: arms
                    .iter()
                    .map(|arm| self.convert_case_arm(arm, environment))
                    .collect(),
            },
            anf::Operation::PrimitiveUnary { operator, operand } => Operation::PrimitiveUnary {
                operator: *operator,
                operand: self.convert_atom(operand, environment),
            },
            anf::Operation::PrimitiveBinary {
                operator,
                left,
                right,
            } => Operation::PrimitiveBinary {
                operator: *operator,
                left: self.convert_atom(left, environment),
                right: self.convert_atom(right, environment),
            },
        }
    }

    fn lift_function(&mut self, lambda: &anf::Lambda) {
        let mut environment = lambda
            .captures
            .iter()
            .enumerate()
            .map(|(index, capture)| (capture.binding, Reference::EnvironmentField(index)))
            .collect::<HashMap<_, _>>();
        if let Some(self_binding) = lambda.self_binding {
            environment.insert(self_binding, Reference::SelfClosure(lambda.id));
        }
        let body = self.convert_block(&lambda.body, &environment);
        self.functions.push(Function {
            id: lambda.id,
            environment: lambda
                .captures
                .iter()
                .map(|capture| EnvironmentField {
                    ty: capture.ty.clone(),
                })
                .collect(),
            parameter: Parameter {
                binding: lambda.parameter.binding,
                ty: lambda.parameter.ty.clone(),
                span: lambda.parameter.span,
            },
            body,
        });
    }

    fn convert_case_arm(
        &mut self,
        arm: &anf::CaseArm,
        environment: &HashMap<anf::ValueId, Reference>,
    ) -> ast::CaseArm {
        ast::CaseArm {
            index: arm.index,
            pattern: self.convert_pattern(&arm.pattern),
            value: self.convert_block(&arm.value, environment),
            span: arm.span,
        }
    }

    fn convert_pattern(&self, pattern: &anf::Pattern) -> Pattern {
        match pattern {
            anf::Pattern::Binding { id, ty } => Pattern::Binding {
                id: *id,
                ty: ty.clone(),
            },
            anf::Pattern::Wildcard { ty, span } => Pattern::Wildcard {
                ty: ty.clone(),
                span: *span,
            },
            anf::Pattern::Product { elements, ty, span } => Pattern::Product {
                elements: elements
                    .iter()
                    .map(|element| self.convert_pattern(element))
                    .collect(),
                ty: ty.clone(),
                span: *span,
            },
        }
    }

    fn convert_atom(
        &self,
        atom: &anf::Atom,
        environment: &HashMap<anf::ValueId, Reference>,
    ) -> Atom {
        let kind = match &atom.kind {
            anf::AtomKind::Reference(id) => self.reference_kind(*id, environment),
            anf::AtomKind::Integer(value) => AtomKind::Integer(*value),
            anf::AtomKind::Float(bits) => AtomKind::Float(*bits),
            anf::AtomKind::String(value) => AtomKind::String(value.clone()),
            anf::AtomKind::Unit => AtomKind::Unit,
        };
        Atom {
            kind,
            ty: atom.ty.clone(),
            span: atom.span,
        }
    }

    fn reference_atom(
        &self,
        id: anf::ValueId,
        ty: crate::check::ast::Type,
        span: crate::source::Span,
        environment: &HashMap<anf::ValueId, Reference>,
    ) -> Atom {
        Atom {
            kind: self.reference_kind(id, environment),
            ty,
            span,
        }
    }

    fn reference_kind(
        &self,
        id: anf::ValueId,
        environment: &HashMap<anf::ValueId, Reference>,
    ) -> AtomKind {
        match environment.get(&id) {
            Some(reference) => AtomKind::Reference(*reference),
            None => AtomKind::Reference(Reference::Binding(id)),
        }
    }
}
