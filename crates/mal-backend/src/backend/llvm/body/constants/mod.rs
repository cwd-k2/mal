//! Closed top-level values, evaluated once into LLVM constants that every function reads.

use std::collections::HashMap;

use crate::anf::ast::ValueId;
use crate::closure::ast::{AtomKind, Pattern, Reference, TopLevelPattern};

use mal_frontend::check::ast::Type;

use super::types::Types;
use crate::backend::llvm::syntax::{
    BinaryOperator, CastOperator, Constant as LlvmConstant, TypedConstant, UnaryOperator,
    llvm_constant, llvm_typed_constant,
};

mod evaluation;
mod pattern;

pub(super) struct TopLevelConstants {
    values: HashMap<ValueId, Constant>,
    globals: Vec<crate::backend::llvm::syntax::GlobalDefinition>,
    types: Types,
}

#[derive(Clone)]
pub(super) struct Constant {
    pub(super) ty: Type,
    kind: ConstantKind,
}

#[derive(Clone)]
enum ConstantKind {
    Value(LlvmConstant),
    Product(Vec<Constant>),
    Sum { index: usize, value: Box<Constant> },
}

impl TopLevelConstants {
    pub(super) fn new(execution: &crate::execution::Program, types: Types) -> Option<Self> {
        let mut constants = Self {
            values: HashMap::new(),
            globals: Vec::new(),
            types,
        };
        for binding in &execution.lowered.bindings {
            let mut locals = HashMap::new();
            for local in &binding.value.bindings {
                let ty = local.pattern.ty();
                let value = constants.operation(&local.operation, ty, &locals)?;
                constants.bind_local_pattern(&local.pattern, value, &mut locals)?;
            }
            let value = constants.atom(&binding.value.result, &locals)?;
            constants.bind_top_pattern(&binding.pattern, value)?;
        }
        Some(constants)
    }

    pub(super) fn globals(&self) -> &[crate::backend::llvm::syntax::GlobalDefinition] {
        &self.globals
    }

    pub(super) fn get(&self, id: ValueId) -> Option<&Constant> {
        self.values.get(&id)
    }
}

impl Constant {
    fn llvm(&self) -> Option<&LlvmConstant> {
        let ConstantKind::Value(value) = &self.kind else {
            return None;
        };
        Some(value)
    }

    fn typed(&self, types: Types) -> Option<TypedConstant> {
        llvm_typed_constant! {
            ({ types.value(&self.ty)?.llvm }, { self.llvm()?.clone() })
        }
    }

    pub(super) fn product(&self) -> Option<&[Constant]> {
        let ConstantKind::Product(elements) = &self.kind else {
            return None;
        };
        Some(elements)
    }

    pub(super) fn sum(&self) -> Option<(usize, &Constant)> {
        let ConstantKind::Sum { index, value } = &self.kind else {
            return None;
        };
        Some((*index, value))
    }

    pub(super) fn value(&self) -> Option<String> {
        self.llvm().map(LlvmConstant::render)
    }
}
