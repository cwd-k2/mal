use crate::check::ast::Type;
use crate::closure::ast::{self as closure, Atom, Operation, Pattern, TopLevelPattern};

mod host;

#[derive(Default)]
pub(super) struct TypeRegistry {
    aggregates: Vec<Type>,
    public: Vec<Type>,
    opaque_names: Vec<String>,
    uses_float32: bool,
    uses_float64: bool,
}

impl TypeRegistry {
    pub(super) fn collect_program(&mut self, program: &closure::Program) {
        self.opaque_names.extend(
            program
                .interface
                .external_types
                .iter()
                .map(|external| external.name.clone()),
        );
        for external in &program.interface.externals {
            self.collect_public(&external.parameter);
            self.collect_public(&external.result);
        }
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
            Type::Engram => "MalType_Engram".into(),
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

    pub(super) fn source_declarations(&self) -> String {
        self.declarations(false)
    }

    fn declarations(&self, public: bool) -> String {
        let mut output = String::new();
        for (index, ty) in self.aggregates.iter().enumerate() {
            if self.is_public(ty) != public {
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
                | Type::Engram
                | Type::Ptr => unreachable!(),
            };
            c_line!(
                &mut output,
                0,
                "typedef struct {kind}_{index} {kind}_{index};"
            );
        }
        if !output.is_empty() {
            output.push('\n');
        }
        for (index, ty) in self.aggregates.iter().enumerate() {
            if self.is_public(ty) != public {
                continue;
            }
            match ty {
                Type::Product(elements) => {
                    c_line!(&mut output, 0, "struct MalRepr_Product_{index} {{");
                    for (element_index, element) in elements.iter().enumerate() {
                        c_line!(
                            &mut output,
                            1,
                            "{} field_{element_index};",
                            self.c_type(element)
                        );
                    }
                    output.push_str("};\n\n");
                }
                Type::Sum(members) => {
                    c_line!(&mut output, 0, "struct MalRepr_Sum_{index} {{");
                    output.push_str("    uint32_t tag;\n    union {\n");
                    for (member_index, member) in members.iter().enumerate() {
                        c_line!(
                            &mut output,
                            2,
                            "{} variant_{member_index};",
                            self.c_type(member)
                        );
                    }
                    output.push_str("    } payload;\n};\n\n");
                }
                Type::Function { parameter, result } => {
                    c_line!(&mut output, 0, "struct MalRepr_Closure_{index} {{");
                    c_line!(
                        &mut output,
                        1,
                        "{} (*call)(MalContext *, const void *, {});",
                        self.c_type(result),
                        self.c_type(parameter)
                    );
                    output.push_str("    const void *environment;\n};\n\n");
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
                | Type::Engram
                | Type::Ptr => unreachable!(),
            }
        }
        output
    }

    fn collect_public(&mut self, ty: &Type) {
        if !self.public.contains(ty) {
            self.public.push(ty.clone());
        }
        if is_bool(ty) {
            return;
        }
        match ty {
            Type::Product(elements) | Type::Sum(elements) => {
                for element in elements {
                    self.collect_public(element);
                }
                self.collect(ty);
            }
            Type::Function { .. } => {
                unreachable!("type checking excludes functions from extern signatures")
            }
            _ => {}
        }
    }

    fn is_public(&self, ty: &Type) -> bool {
        self.public.contains(ty)
    }

    fn is_host_type(&self, ty: &Type) -> bool {
        self.public.contains(ty)
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
            | Type::Engram
            | Type::Ptr => return,
        }
        if !self.aggregates.contains(ty) {
            self.aggregates.push(ty.clone());
        }
    }

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
            Operation::EngramLength { value } => self.collect_atom(value),
            Operation::EngramAt { argument } => self.collect_atom(argument),
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

    fn index(&self, ty: &Type) -> usize {
        self.aggregates
            .iter()
            .position(|candidate| candidate == ty)
            .expect("all emitted types are collected before rendering")
    }
}

pub(super) fn is_bool(ty: &Type) -> bool {
    matches!(ty, Type::Sum(members) if members == &[Type::Unit, Type::Unit])
}
