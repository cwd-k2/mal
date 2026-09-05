use crate::check::ast::Type;
use crate::closure::ast::{self as closure, Atom, Operation, Pattern, TopLevelPattern};

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
                .external_types
                .iter()
                .map(|external| external.name.clone()),
        );
        for external in &program.externals {
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
        match ty {
            Type::Unit => "MalUnit".into(),
            Type::Int8 => "int8_t".into(),
            Type::Int16 => "int16_t".into(),
            Type::Int32 => "int32_t".into(),
            Type::Int64 => "int64_t".into(),
            Type::UInt8 => "uint8_t".into(),
            Type::UInt16 => "uint16_t".into(),
            Type::UInt32 => "uint32_t".into(),
            Type::UInt64 => "uint64_t".into(),
            Type::Float32 => "float".into(),
            Type::Float64 => "double".into(),
            Type::String => "MalString".into(),
            Type::Ptr => "MalPtr".into(),
            Type::External { name, .. } => format!("MalOpaque_{name}"),
            Type::Product(_) => format!("MalProduct_{}", self.index(ty)),
            Type::Sum(_) => format!("MalSum_{}", self.index(ty)),
            Type::Function { .. } => format!("MalClosure_{}", self.index(ty)),
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

    pub(super) fn header_declarations(&self) -> String {
        let mut output = String::new();
        for name in &self.opaque_names {
            c_line!(
                &mut output,
                0,
                "typedef struct {{ uintptr_t bits; }} MalOpaque_{name};"
            );
        }
        if !self.opaque_names.is_empty() {
            output.push('\n');
        }
        output.push_str(&self.declarations(true));
        output
    }

    fn declarations(&self, public: bool) -> String {
        let mut output = String::new();
        for (index, ty) in self.aggregates.iter().enumerate() {
            if self.is_public(ty) != public {
                continue;
            }
            let kind = match ty {
                Type::Product(_) => "MalProduct",
                Type::Sum(_) => "MalSum",
                Type::Function { .. } => "MalClosure",
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
                | Type::String
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
                    c_line!(&mut output, 0, "struct MalProduct_{index} {{");
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
                    c_line!(&mut output, 0, "struct MalSum_{index} {{");
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
                    c_line!(&mut output, 0, "struct MalClosure_{index} {{");
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
                | Type::String
                | Type::Ptr => unreachable!(),
            }
        }
        output
    }

    fn collect_public(&mut self, ty: &Type) {
        match ty {
            Type::Product(elements) | Type::Sum(elements) => {
                for element in elements {
                    self.collect_public(element);
                }
                self.collect(ty);
                if !self.public.contains(ty) {
                    self.public.push(ty.clone());
                }
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

    fn collect(&mut self, ty: &Type) {
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
            | Type::String
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
            Operation::StringLength { value } => self.collect_atom(value),
            Operation::StringAt { argument } => self.collect_atom(argument),
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
