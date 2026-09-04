use std::fmt::Write;

use crate::check::ast::Type;
use crate::closure::ast::{self as closure, Atom, Operation, Pattern, TopLevelPattern};

#[derive(Default)]
pub(super) struct TypeRegistry {
    aggregates: Vec<Type>,
}

impl TypeRegistry {
    pub(super) fn collect_program(&mut self, program: &closure::Program) {
        for external in &program.externals {
            self.collect(&external.parameter);
            self.collect(&external.result);
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
            Type::Int32 => "int32_t".into(),
            Type::Sum(_) => format!("MalSum_{}", self.index(ty)),
            Type::Function { .. } => format!("MalClosure_{}", self.index(ty)),
        }
    }

    pub(super) fn declarations(&self) -> String {
        let mut output = String::new();
        for (index, ty) in self.aggregates.iter().enumerate() {
            let kind = match ty {
                Type::Sum(_) => "MalSum",
                Type::Function { .. } => "MalClosure",
                Type::Unit | Type::Int32 => unreachable!(),
            };
            writeln!(output, "typedef struct {kind}_{index} {kind}_{index};").unwrap();
        }
        if !self.aggregates.is_empty() {
            output.push('\n');
        }
        for (index, ty) in self.aggregates.iter().enumerate() {
            match ty {
                Type::Sum(members) => {
                    writeln!(output, "struct MalSum_{index} {{").unwrap();
                    output.push_str("    uint32_t tag;\n    union {\n");
                    for (member_index, member) in members.iter().enumerate() {
                        writeln!(
                            output,
                            "        {} variant_{member_index};",
                            self.c_type(member)
                        )
                        .unwrap();
                    }
                    output.push_str("    } payload;\n};\n\n");
                }
                Type::Function { parameter, result } => {
                    writeln!(output, "struct MalClosure_{index} {{").unwrap();
                    writeln!(
                        output,
                        "    {} (*call)(MalContext *, const void *, {});",
                        self.c_type(result),
                        self.c_type(parameter)
                    )
                    .unwrap();
                    output.push_str("    const void *environment;\n};\n\n");
                }
                Type::Unit | Type::Int32 => unreachable!(),
            }
        }
        output
    }

    fn collect(&mut self, ty: &Type) {
        match ty {
            Type::Sum(members) => {
                for member in members {
                    self.collect(member);
                }
            }
            Type::Function { parameter, result } => {
                self.collect(parameter);
                self.collect(result);
            }
            Type::Unit | Type::Int32 => return,
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
        }
    }

    fn collect_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Binding { ty, .. } | Pattern::Wildcard { ty, .. } => self.collect(ty),
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
            Operation::MakeClosure { captures, .. } => {
                for capture in captures {
                    self.collect_atom(capture);
                }
            }
            Operation::Call { callee, argument } => {
                self.collect_atom(callee);
                self.collect_atom(argument);
            }
            Operation::ExternalCall { argument, .. }
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
