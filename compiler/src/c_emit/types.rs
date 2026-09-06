use crate::c_emit::syntax::{AggregateDefinition, AggregateField, Declaration};
use crate::check::ast::Type;
use crate::closure::ast::{self as closure, Atom, Operation, Pattern, TopLevelPattern};
use crate::core::ast::ProgramInterface;

mod host;

#[derive(Default)]
pub(super) struct TypeRegistry {
    aggregates: Vec<Type>,
    uses_float32: bool,
    uses_float64: bool,
}

#[derive(Default)]
pub(super) struct HostTypes {
    types: Vec<Type>,
    opaque_names: Vec<String>,
}

impl TypeRegistry {
    pub(super) fn collect_program_body(&mut self, program: &closure::Program) {
        for binding in &program.bindings {
            self.collect_top_pattern(&binding.pattern);
            self.collect_block(&binding.value);
        }
        for function in &program.functions {
            for field in &function.environment {
                self.collect(&field.ty);
            }
            self.collect(&function.parameter.ty);
            self.collect_block(&function.body);
        }
    }

    pub(super) fn c_type(&self, ty: &Type) -> String {
        if is_bool(ty) {
            return "MalType_Bool".into();
        }
        match ty {
            Type::Unit => "MalType_Unit".into(),
            Type::Int8 => "MalType_Int8".into(),
            Type::Int16 => "MalType_Int16".into(),
            Type::Int32 => "MalType_Int32".into(),
            Type::Int64 => "MalType_Int64".into(),
            Type::UInt8 => "MalType_UInt8".into(),
            Type::UInt16 => "MalType_UInt16".into(),
            Type::UInt32 => "MalType_UInt32".into(),
            Type::UInt64 => "MalType_UInt64".into(),
            Type::Float32 => "MalType_Float32".into(),
            Type::Float64 => "MalType_Float64".into(),
            Type::Symbol => "MalType_Symbol".into(),
            Type::Ptr => "MalType_Ptr".into(),
            Type::External { name, .. } => format!("MalType_{name}"),
            Type::Product(_) => format!("MalRepr_Product_{}", self.index(ty)),
            Type::Sum(_) => format!("MalRepr_Sum_{}", self.index(ty)),
            Type::Function { .. } => format!("MalRepr_Closure_{}", self.index(ty)),
        }
    }

    pub(super) fn uses_float(&self) -> bool {
        self.uses_float32 || self.uses_float64
    }

    pub(super) fn uses_float32(&self) -> bool {
        self.uses_float32
    }

    pub(super) fn uses_float64(&self) -> bool {
        self.uses_float64
    }

    pub(super) fn source_declarations(&self, host: &HostTypes) -> String {
        self.declarations(host, false)
    }

    fn declarations(&self, host: &HostTypes, public: bool) -> String {
        let mut output = String::new();
        for (index, ty) in self.aggregates.iter().enumerate() {
            if host.contains(ty) != public {
                continue;
            }
            let kind = match ty {
                Type::Product(_) => "MalRepr_Product",
                Type::Sum(_) => "MalRepr_Sum",
                Type::Function { .. } => "MalRepr_Closure",
                Type::External { .. }
                | Type::Unit
                | Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::Float32
                | Type::Float64
                | Type::Symbol
                | Type::Ptr => unreachable!(),
            };
            output.push_str(
                &Declaration::new(format!("typedef struct {kind}_{index} {kind}_{index}")).render(),
            );
        }
        if !output.is_empty() {
            output.push('\n');
        }
        for (index, ty) in self.aggregates.iter().enumerate() {
            if host.contains(ty) != public {
                continue;
            }
            match ty {
                Type::Product(elements) => {
                    let fields = elements.iter().enumerate().map(|(element_index, element)| {
                        AggregateField::declaration(format!(
                            "{} field_{element_index}",
                            self.c_type(element)
                        ))
                    });
                    output.push_str(
                        &AggregateDefinition::new(
                            format!("struct MalRepr_Product_{index}"),
                            fields,
                            None,
                        )
                        .render(),
                    );
                    output.push('\n');
                }
                Type::Sum(members) => {
                    let variants = members.iter().enumerate().map(|(member_index, member)| {
                        AggregateField::declaration(format!(
                            "{} variant_{member_index}",
                            self.c_type(member)
                        ))
                    });
                    output.push_str(
                        &AggregateDefinition::new(
                            format!("struct MalRepr_Sum_{index}"),
                            [
                                AggregateField::declaration("uint32_t tag"),
                                AggregateField::aggregate("union", variants, "payload"),
                            ],
                            None,
                        )
                        .render(),
                    );
                    output.push('\n');
                }
                Type::Function { parameter, result } => {
                    output.push_str(
                        &AggregateDefinition::new(
                            format!("struct MalRepr_Closure_{index}"),
                            [
                                AggregateField::declaration(format!(
                                    "{} (*call)(MalContext *, const void *, {})",
                                    self.c_type(result),
                                    self.c_type(parameter)
                                )),
                                AggregateField::declaration("const void *environment"),
                            ],
                            None,
                        )
                        .render(),
                    );
                    output.push('\n');
                }
                Type::External { .. }
                | Type::Unit
                | Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::Float32
                | Type::Float64
                | Type::Symbol
                | Type::Ptr => unreachable!(),
            }
        }
        output
    }

    fn collect(&mut self, ty: &Type) {
        if is_bool(ty) {
            return;
        }
        match ty {
            Type::Float32 => {
                self.uses_float32 = true;
                return;
            }
            Type::Float64 => {
                self.uses_float64 = true;
                return;
            }
            _ => {}
        }
        match ty {
            Type::Product(elements) | Type::Sum(elements) => {
                for element in elements {
                    self.collect(element);
                }
            }
            Type::Function { parameter, result } => {
                self.collect(parameter);
                self.collect(result);
            }
            Type::External { .. }
            | Type::Unit
            | Type::Int8
            | Type::Int16
            | Type::Int32
            | Type::Int64
            | Type::UInt8
            | Type::UInt16
            | Type::UInt32
            | Type::UInt64
            | Type::Float32
            | Type::Float64
            | Type::Symbol
            | Type::Ptr => return,
        }
        if !self.aggregates.contains(ty) {
            self.aggregates.push(ty.clone());
        }
    }

    fn index(&self, ty: &Type) -> usize {
        self.aggregates
            .iter()
            .position(|candidate| candidate == ty)
            .expect("all emitted types are collected before rendering")
    }
}

impl HostTypes {
    pub(super) fn collect(interface: &ProgramInterface, registry: &mut TypeRegistry) -> Self {
        let mut host = Self::default();
        host.opaque_names.extend(
            interface
                .external_types
                .iter()
                .map(|external| external.name.clone()),
        );
        for external in &interface.externals {
            host.collect_type(&external.parameter, registry);
            host.collect_type(&external.result, registry);
        }
        host
    }

    fn collect_type(&mut self, ty: &Type, registry: &mut TypeRegistry) {
        if !self.types.contains(ty) {
            self.types.push(ty.clone());
        }
        if is_bool(ty) {
            return;
        }
        match ty {
            Type::Product(elements) | Type::Sum(elements) => {
                for element in elements {
                    self.collect_type(element, registry);
                }
                registry.collect(ty);
            }
            Type::Function { .. } => {
                unreachable!("type checking excludes functions from extern signatures")
            }
            _ => {}
        }
    }

    pub(super) fn contains(&self, ty: &Type) -> bool {
        self.types.contains(ty)
    }
}

impl TypeRegistry {
    fn collect_top_pattern(&mut self, pattern: &TopLevelPattern) {
        match pattern {
            TopLevelPattern::Binding { ty, .. } | TopLevelPattern::Wildcard { ty, .. } => {
                self.collect(ty);
            }
            TopLevelPattern::Product { ty, .. } => self.collect(ty),
        }
    }

    fn collect_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Binding { ty, .. }
            | Pattern::Wildcard { ty, .. }
            | Pattern::Product { ty, .. } => self.collect(ty),
        }
    }

    fn collect_atom(&mut self, atom: &Atom) {
        self.collect(&atom.ty);
    }

    fn collect_block(&mut self, block: &closure::Block) {
        for binding in &block.bindings {
            self.collect_pattern(&binding.pattern);
            self.collect_operation(&binding.operation);
        }
        self.collect_atom(&block.result);
    }

    fn collect_operation(&mut self, operation: &Operation) {
        match operation {
            Operation::Atom(atom) => self.collect_atom(atom),
            Operation::Product(elements) => {
                for element in elements {
                    self.collect_atom(element);
                }
            }
            Operation::MakeClosure { captures, .. } => {
                for capture in captures {
                    self.collect_atom(capture);
                }
            }
            Operation::Call { callee, argument } => {
                self.collect_atom(callee);
                self.collect_atom(argument);
            }
            Operation::SymbolLength { value } => self.collect_atom(value),
            Operation::SymbolAt { argument } => self.collect_atom(argument),
            Operation::Memory { argument, .. } => self.collect_atom(argument),
            Operation::ExternalCall { argument, .. }
            | Operation::NumericConversion { operand: argument }
            | Operation::SumInjection {
                value: argument, ..
            }
            | Operation::PrimitiveUnary {
                operand: argument, ..
            } => self.collect_atom(argument),
            Operation::Case { scrutinee, arms } => {
                self.collect_atom(scrutinee);
                for arm in arms {
                    self.collect_pattern(&arm.pattern);
                    self.collect_block(&arm.value);
                }
            }
            Operation::PrimitiveBranch {
                left,
                right,
                otherwise,
                then,
                ..
            } => {
                self.collect_atom(left);
                self.collect_atom(right);
                self.collect_block(otherwise);
                self.collect_block(then);
            }
            Operation::PrimitiveBinary { left, right, .. } => {
                self.collect_atom(left);
                self.collect_atom(right);
            }
        }
    }
}

pub(super) fn is_bool(ty: &Type) -> bool {
    matches!(ty, Type::Sum(members) if members == &[Type::Unit, Type::Unit])
}
