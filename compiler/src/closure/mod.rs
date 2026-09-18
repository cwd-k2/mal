use std::collections::HashMap;

use crate::anf::ast as anf;

pub mod ast;

use self::ast::{
    Atom, AtomId, AtomKind, Binding, Block, CaptureField, Function, FunctionId, FunctionKind,
    Operation, Parameter, Pattern, Program, Reference, TopLevelBinding, TopLevelPattern,
};

pub fn convert(program: &anf::Program) -> Program {
    Converter::new().convert_program(program)
}

struct Converter {
    functions: Vec<Function>,
    next_atom: usize,
}

impl Converter {
    fn new() -> Self {
        Self {
            functions: Vec::new(),
            next_atom: 0,
        }
    }

    fn convert_program(mut self, program: &anf::Program) -> Program {
        let empty_environment = HashMap::new();
        let bindings: Vec<_> = program
            .bindings
            .iter()
            .map(|binding| self.convert_top_level_binding(binding, &empty_environment))
            .collect();
        let entry = program.entry.map(|entry| {
            let binding = bindings
                .iter()
                .find(|binding| {
                    matches!(binding.pattern, TopLevelPattern::Binding { id, .. } if id == entry.binding)
                })
                .expect("entry identity names a reachable top-level binding");
            ast::EntryPoint {
                function: top_level_function(binding)
                    .expect("checked entry binding lowers to a capture-free closure"),
                parameter: entry.parameter,
            }
        });
        Program {
            interface: program.interface.clone(),
            bindings,
            functions: self.functions,
            entry,
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
            anf::Operation::Goto { target, value } => Operation::Goto {
                target: *target,
                value: self.convert_atom(value, environment),
            },
            anf::Operation::Lambda(lambda) => {
                self.lift_function(lambda);
                let function = FunctionId::Lambda(lambda.id);
                match &lambda.kind {
                    crate::core::ast::LambdaKind::Ordinary => Operation::MakeClosure {
                        function,
                        captures: lambda
                            .captures
                            .iter()
                            .map(|capture| {
                                self.reference_atom(
                                    capture.source,
                                    capture.ty.clone(),
                                    span,
                                    environment,
                                )
                            })
                            .collect(),
                    },
                    crate::core::ast::LambdaKind::PackedCapability { .. } => {
                        let [capture] = lambda.captures.as_slice() else {
                            unreachable!("Packed capability must capture exactly one builder");
                        };
                        debug_assert_eq!(capture.ty, crate::check::ast::Type::Address);
                        Operation::MakePackedCapability {
                            function,
                            builder: self.reference_atom(
                                capture.source,
                                capture.ty.clone(),
                                span,
                                environment,
                            ),
                        }
                    }
                }
            }
            anf::Operation::Call { callee, argument } => Operation::Call {
                callee: self.convert_atom(callee, environment),
                argument: self.convert_atom(argument, environment),
            },
            anf::Operation::SymbolLength { value } => Operation::SymbolLength {
                value: self.convert_atom(value, environment),
            },
            anf::Operation::SymbolAt { argument } => Operation::SymbolAt {
                argument: self.convert_atom(argument, environment),
            },
            anf::Operation::Memory {
                primitive,
                operands,
            } => Operation::Memory {
                primitive: *primitive,
                operands: operands
                    .iter()
                    .map(|operand| self.convert_atom(operand, environment))
                    .collect(),
            },
            anf::Operation::PackedBuilder {
                operation,
                element,
                argument,
            } => Operation::PackedBuilder {
                operation: *operation,
                element: element.clone(),
                argument: self.convert_atom(argument, environment),
            },
            anf::Operation::ExternalCall { id, argument } => Operation::ExternalCall {
                id: *id,
                argument: self.convert_atom(argument, environment),
            },
            anf::Operation::NumericConversion { operand } => Operation::NumericConversion {
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
            anf::Operation::PrimitiveBranch {
                operator,
                left,
                right,
                otherwise,
                then,
            } => Operation::PrimitiveBranch {
                operator: *operator,
                left: self.convert_atom(left, environment),
                right: self.convert_atom(right, environment),
                otherwise: Box::new(self.convert_block(otherwise, environment)),
                then: Box::new(self.convert_block(then, environment)),
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
        if let Some(existing) = self
            .functions
            .iter()
            .find(|function| function.id == FunctionId::Lambda(lambda.id))
        {
            debug_assert_eq!(existing.parameter.ty, lambda.parameter.ty);
            debug_assert_eq!(existing.body.result.ty, lambda.body.result.ty);
            debug_assert_eq!(existing.kind, Self::function_kind(lambda));
            return;
        }
        let mut environment = match &lambda.kind {
            crate::core::ast::LambdaKind::Ordinary => lambda
                .captures
                .iter()
                .enumerate()
                .map(|(index, capture)| (capture.binding, Reference::Capture(index)))
                .collect::<HashMap<_, _>>(),
            crate::core::ast::LambdaKind::PackedCapability { .. } => {
                let [capture] = lambda.captures.as_slice() else {
                    unreachable!("Packed capability must capture exactly one builder");
                };
                HashMap::from([(capture.binding, Reference::PackedBuilder)])
            }
        };
        if let Some(self_binding) = lambda.self_binding {
            environment.insert(
                self_binding,
                Reference::SelfClosure(FunctionId::Lambda(lambda.id)),
            );
        }
        let body = self.convert_block(&lambda.body, &environment);
        let joins = lambda
            .joins
            .iter()
            .map(|join| ast::Join {
                parameter: self.convert_pattern(&join.parameter),
                body: self.convert_block(&join.body, &environment),
                span: join.span,
            })
            .collect();
        self.functions.push(Function {
            id: FunctionId::Lambda(lambda.id),
            kind: Self::function_kind(lambda),
            parameter: Parameter {
                binding: lambda.parameter.binding,
                ty: lambda.parameter.ty.clone(),
                span: lambda.parameter.span,
            },
            body,
            joins,
        });
    }

    fn function_kind(lambda: &anf::Lambda) -> FunctionKind {
        match &lambda.kind {
            crate::core::ast::LambdaKind::Ordinary => FunctionKind::Ordinary {
                captures: lambda
                    .captures
                    .iter()
                    .map(|capture| CaptureField {
                        ty: capture.ty.clone(),
                    })
                    .collect(),
            },
            crate::core::ast::LambdaKind::PackedCapability { operation, element } => {
                FunctionKind::PackedCapability {
                    operation: *operation,
                    element: element.clone(),
                }
            }
        }
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
        &mut self,
        atom: &anf::Atom,
        environment: &HashMap<anf::ValueId, Reference>,
    ) -> Atom {
        let kind = match &atom.kind {
            anf::AtomKind::Reference(id) => self.reference_kind(*id, environment),
            anf::AtomKind::Integer(value) => AtomKind::Integer(*value),
            anf::AtomKind::Float(bits) => AtomKind::Float(*bits),
            anf::AtomKind::Symbol(value) => AtomKind::Symbol(value.clone()),
            anf::AtomKind::StorageSize(ty) => AtomKind::StorageSize(ty.clone()),
            anf::AtomKind::Unit => AtomKind::Unit,
        };
        Atom {
            id: self.atom_id(),
            kind,
            ty: atom.ty.clone(),
            span: atom.span,
        }
    }

    fn reference_atom(
        &mut self,
        id: anf::ValueId,
        ty: crate::check::ast::Type,
        span: crate::source::Span,
        environment: &HashMap<anf::ValueId, Reference>,
    ) -> Atom {
        Atom {
            id: self.atom_id(),
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

    fn atom_id(&mut self) -> AtomId {
        let id = AtomId(self.next_atom);
        self.next_atom += 1;
        id
    }
}

fn top_level_function(binding: &TopLevelBinding) -> Option<FunctionId> {
    let AtomKind::Reference(Reference::Binding(result)) = binding.value.result.kind else {
        return None;
    };
    binding.value.bindings.iter().find_map(|binding| {
        let Pattern::Binding { id, .. } = binding.pattern else {
            return None;
        };
        match &binding.operation {
            Operation::MakeClosure { function, captures }
                if id == result && captures.is_empty() =>
            {
                Some(*function)
            }
            _ => None,
        }
    })
}
