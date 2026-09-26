use std::collections::HashMap;

use crate::anf::ast::ValueId;
use crate::closure::ast::{AtomKind, FunctionId, Pattern, Reference, TopLevelPattern};

use mal_frontend::check::ast::Type;

use super::Slot;
use super::types::Types;
use crate::backend::llvm::syntax::{
    BinaryOperator, CastOperator, Constant as LlvmConstant, TypedConstant, UnaryOperator,
    llvm_constant, llvm_type, llvm_typed_constant,
};

pub(super) fn main_function(execution: &crate::execution::Program) -> Option<(FunctionId, Type)> {
    let entry = execution.lowered.entry?;
    let parameter = match entry.parameter {
        mal_frontend::check::ast::EntryParameter::Unit => Type::Unit,
        mal_frontend::check::ast::EntryParameter::ProcessArguments => {
            mal_frontend::check::ast::EntryParameter::process_arguments_type()
        }
    };
    Some((entry.function, parameter))
}

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

    fn operation(
        &mut self,
        operation: &crate::closure::ast::Operation,
        result_type: &Type,
        values: &HashMap<ValueId, Constant>,
    ) -> Option<Constant> {
        use crate::closure::ast::Operation;

        let value = match operation {
            Operation::Atom(atom) => self.atom(atom, values)?,
            Operation::MakeClosure { function, captures } if captures.is_empty() => Constant {
                ty: result_type.clone(),
                kind: ConstantKind::Value(llvm_constant!(structure [
                    (typed llvm_type!(ptr) =>
                        (atom format!("@{}", super::function_name(*function)))),
                    (typed llvm_type!(ptr) => (atom "null")),
                ])?),
            },
            Operation::NumericConversion { operand } => {
                let operand = self.atom(operand, values)?;
                numeric_conversion(operand, result_type, self.types.clone())?
            }
            Operation::Product(elements) => {
                let Type::Product(element_types) = result_type else {
                    return None;
                };
                if elements.len() != element_types.len() {
                    return None;
                }
                let elements = elements
                    .iter()
                    .zip(element_types.iter())
                    .map(|(atom, expected)| {
                        let value = self.atom(atom, values)?;
                        (value.ty == *expected).then_some(value)
                    })
                    .collect::<Option<Vec<_>>>()?;
                Constant {
                    ty: result_type.clone(),
                    kind: ConstantKind::Product(elements),
                }
            }
            Operation::SumInjection { index, value } => {
                let Type::Sum(members) = result_type else {
                    return None;
                };
                let member = members.get(*index)?;
                let value = self.atom(value, values)?;
                if value.ty != *member {
                    return None;
                }
                let kind = if super::types::is_bool(result_type) {
                    match index {
                        0 => ConstantKind::Value(llvm_constant!(atom "false")?),
                        1 => ConstantKind::Value(llvm_constant!(atom "true")?),
                        _ => return None,
                    }
                } else {
                    ConstantKind::Sum {
                        index: *index,
                        value: Box::new(value),
                    }
                };
                Constant {
                    ty: result_type.clone(),
                    kind,
                }
            }
            Operation::PrimitiveUnary { operator, operand } => {
                let operand = self.atom(operand, values)?;
                let scalar = super::scalar::scalar_type(&operand.ty, self.types.index_size())?;
                let value = match operator {
                    crate::core::ast::UnaryPrimitive::Negate if scalar.floating => {
                        llvm_constant!(unary UnaryOperator::FNeg;
                            { operand.typed(self.types.clone())? }
                        )?
                    }
                    crate::core::ast::UnaryPrimitive::Negate => llvm_constant!(binary
                        BinaryOperator::Sub;
                        (typed scalar.llvm_type() => (atom 0));
                        { operand.typed(self.types.clone())? }
                    )?,
                    _ => return None,
                };
                Constant {
                    ty: operand.ty,
                    kind: ConstantKind::Value(value),
                }
            }
            Operation::PrimitiveBinary {
                operator,
                left,
                right,
            } => {
                let left = self.atom(left, values)?;
                let right = self.atom(right, values)?;
                if left.ty != right.ty {
                    return None;
                }
                let scalar = super::scalar::scalar_type(&left.ty, self.types.index_size())?;
                let instruction = super::scalar::arithmetic_instruction(*operator, scalar)?;
                let value = llvm_constant!(binary instruction;
                    { left.typed(self.types.clone())? };
                    { right.typed(self.types.clone())? }
                )?;
                Constant {
                    ty: left.ty,
                    kind: ConstantKind::Value(value),
                }
            }
            _ => return None,
        };
        (value.ty == *result_type).then_some(value)
    }

    fn atom(
        &mut self,
        atom: &crate::closure::ast::Atom,
        values: &HashMap<ValueId, Constant>,
    ) -> Option<Constant> {
        let value = match &atom.kind {
            AtomKind::Integer(value) => llvm_constant!(atom super::scalar::integer_literal(
                &atom.ty,
                *value,
                self.types.index_size(),
            )?)?,
            AtomKind::Float(bits) if atom.ty == Type::Float32 => llvm_constant!(atom format!(
                "0x{:016X}",
                (f32::from_bits(*bits as u32) as f64).to_bits()
            ))?,
            AtomKind::Float(bits) if atom.ty == Type::Float64 => {
                llvm_constant!(atom format!("0x{bits:016X}"))?
            }
            AtomKind::Symbol(bytes) if bytes.is_empty() => llvm_constant!(zero)?,
            AtomKind::Symbol(bytes) => {
                let name = format!("mal_top_symbol_{}", atom.id.0);
                self.globals
                    .push(super::symbol::literal_definition(&name, bytes)?);
                let address_constant = llvm_constant!(atom format!("@{name}"))?;
                let address = || {
                    llvm_typed_constant!(typed
                        llvm_type!(ptr) =>
                        { address_constant.clone() }
                    )
                };
                llvm_constant!(structure [
                    { address()? },
                    (typed llvm_type!(ptr) =>
                        (get_element_ptr llvm_type!(int(8_u16));
                            { address()? };
                            [(typed self.types.index_llvm_type() =>
                                (atom super::symbol::STATIC_OWNER_DATA_OFFSET))]
                        )
                    ),
                    (typed self.types.index_llvm_type() => (atom bytes.len())),
                ])?
            }
            AtomKind::Unit if atom.ty == Type::Unit => llvm_constant!(atom 0)?,
            AtomKind::Reference(Reference::Binding(id)) => return values.get(id).cloned(),
            _ => return None,
        };
        Some(Constant {
            ty: atom.ty.clone(),
            kind: ConstantKind::Value(value),
        })
    }

    fn bind_local_pattern(
        &self,
        pattern: &Pattern,
        value: Constant,
        values: &mut HashMap<ValueId, Constant>,
    ) -> Option<()> {
        bind_pattern(pattern, value, values)
    }

    fn bind_top_pattern(&mut self, pattern: &TopLevelPattern, value: Constant) -> Option<()> {
        bind_top_pattern(pattern, value, &mut self.values)
    }
}

fn bind_pattern(
    pattern: &Pattern,
    value: Constant,
    values: &mut HashMap<ValueId, Constant>,
) -> Option<()> {
    if *pattern.ty() != value.ty {
        return None;
    }
    match pattern {
        Pattern::Binding { id, .. } => {
            values.insert(*id, value);
        }
        Pattern::Wildcard { .. } => {}
        Pattern::Product { elements, .. } => {
            let ConstantKind::Product(fields) = value.kind else {
                return None;
            };
            if fields.len() != elements.len() {
                return None;
            }
            for (element, field) in elements.iter().zip(fields) {
                bind_pattern(element, field, values)?;
            }
        }
    }
    Some(())
}

fn bind_top_pattern(
    pattern: &TopLevelPattern,
    value: Constant,
    values: &mut HashMap<ValueId, Constant>,
) -> Option<()> {
    let ty = match pattern {
        TopLevelPattern::Binding { ty, .. }
        | TopLevelPattern::Wildcard { ty, .. }
        | TopLevelPattern::Product { ty, .. } => ty,
    };
    if *ty != value.ty {
        return None;
    }
    match pattern {
        TopLevelPattern::Binding { id, .. } => {
            values.insert(*id, value);
        }
        TopLevelPattern::Wildcard { .. } => {}
        TopLevelPattern::Product { elements, .. } => {
            let ConstantKind::Product(fields) = value.kind else {
                return None;
            };
            if fields.len() != elements.len() {
                return None;
            }
            for (element, field) in elements.iter().zip(fields) {
                bind_top_pattern(element, field, values)?;
            }
        }
    }
    Some(())
}

impl Constant {
    fn llvm(&self) -> Option<&LlvmConstant> {
        let ConstantKind::Value(value) = &self.kind else {
            return None;
        };
        Some(value)
    }

    fn typed(&self, types: Types) -> Option<TypedConstant> {
        llvm_typed_constant!(typed types.value(&self.ty)?.llvm => { self.llvm()?.clone() })
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

fn numeric_conversion(operand: Constant, result_type: &Type, types: Types) -> Option<Constant> {
    let source = super::scalar::scalar_type(&operand.ty, types.index_size())?;
    let target = super::scalar::scalar_type(result_type, types.index_size())?;
    let representation = if source.floating == target.floating && source.bits == target.bits {
        operand.llvm()?.clone()
    } else if !source.floating && !target.floating {
        let value = operand.llvm()?.integer_value()?;
        let modulus = 1_i128 << target.bits;
        let residue = value.rem_euclid(modulus);
        if target.signed && residue >= modulus / 2 {
            llvm_constant!(atom residue - modulus)?
        } else {
            llvm_constant!(atom residue)?
        }
    } else {
        let instruction = if source.floating && target.floating {
            if source.bits > target.bits {
                CastOperator::FPTrunc
            } else {
                CastOperator::FPExt
            }
        } else if source.floating {
            if target.signed {
                CastOperator::FPToSI
            } else {
                CastOperator::FPToUI
            }
        } else if target.floating {
            if source.signed {
                CastOperator::SIToFP
            } else {
                CastOperator::UIToFP
            }
        } else if source.bits > target.bits {
            CastOperator::Trunc
        } else if source.signed {
            CastOperator::SExt
        } else {
            CastOperator::ZExt
        };
        llvm_constant!(cast instruction;
            { operand.typed(types.clone())? };
            target.llvm_type()
        )?
    };
    Some(Constant {
        ty: result_type.clone(),
        kind: ConstantKind::Value(representation),
    })
}

pub(super) fn collect_pattern_slot(
    pattern: &Pattern,
    slots: &mut HashMap<ValueId, Slot>,
    types: Types,
) -> Option<()> {
    match pattern {
        Pattern::Binding { id, ty } if types.value(ty).is_some() => {
            insert_slot(slots, *id, ty.clone())
        }
        Pattern::Product { elements, .. } => {
            for element in elements {
                collect_pattern_slot(element, slots, types.clone())?;
            }
        }
        Pattern::Wildcard { .. } => {}
        _ => return None,
    }
    Some(())
}

pub(super) fn insert_slot(slots: &mut HashMap<ValueId, Slot>, id: ValueId, ty: Type) {
    if !slots.contains_key(&id) {
        slots.insert(
            id,
            Slot {
                index: slots.len(),
                ty,
            },
        );
    }
}
