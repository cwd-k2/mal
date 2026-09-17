use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::closure::ast::{AtomKind, FunctionId, Pattern, Reference, TopLevelPattern};
use crate::control::ast::{Program, StateId, Terminator};

use super::Slot;
use super::source_layout::SourceLayouts;
use super::types::Types;

pub(super) fn main_function(execution: &crate::execution::Program) -> Option<(FunctionId, Type)> {
    let binding = execution.lowered.bindings.iter().find(|binding| {
        matches!(&binding.pattern, TopLevelPattern::Binding { name, .. } if name == "main")
    })?;
    let TopLevelPattern::Binding { ty, .. } = &binding.pattern else {
        return None;
    };
    let Type::Function { parameter, result } = ty else {
        return None;
    };
    if **result != Type::Int32
        || (**parameter != Type::Unit
            && **parameter != Type::Product(vec![Type::USize, Type::Address].into()))
    {
        return None;
    }
    Some((closure_binding_function(binding)?, (**parameter).clone()))
}

pub(super) struct TopLevelConstants {
    values: HashMap<ValueId, Constant>,
    globals: String,
    types: Types,
    source_layouts: SourceLayouts,
}

#[derive(Clone)]
pub(super) struct Constant {
    pub(super) ty: Type,
    kind: ConstantKind,
}

#[derive(Clone)]
enum ConstantKind {
    Value(String),
    Product(Vec<Constant>),
    Sum { index: usize, value: Box<Constant> },
}

impl TopLevelConstants {
    pub(super) fn new(
        execution: &crate::execution::Program,
        types: Types,
        source_layouts: SourceLayouts,
    ) -> Option<Self> {
        let mut constants = Self {
            values: HashMap::new(),
            globals: String::new(),
            types,
            source_layouts,
        };
        for binding in &execution.lowered.bindings {
            let mut locals = HashMap::new();
            for local in &binding.value.bindings {
                let ty = pattern_type(&local.pattern);
                let value = constants.operation(&local.operation, ty, &locals)?;
                constants.bind_local_pattern(&local.pattern, value, &mut locals)?;
            }
            let value = constants.atom(&binding.value.result, &locals)?;
            constants.bind_top_pattern(&binding.pattern, value)?;
        }
        Some(constants)
    }

    pub(super) fn globals(&self) -> &str {
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
                kind: ConstantKind::Value(format!(
                    "{{ ptr @{}, ptr null }}",
                    super::function_name(*function)?
                )),
            },
            Operation::NumericConversion { operand } => {
                let operand = self.atom(operand, values)?;
                numeric_conversion(operand, result_type, self.types)?
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
                        0 => ConstantKind::Value("false".into()),
                        1 => ConstantKind::Value("true".into()),
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
                let representation = match operator {
                    crate::core::ast::UnaryPrimitive::Negate if scalar.floating => {
                        format!("fneg ({} {})", scalar.llvm, operand.representation()?)
                    }
                    crate::core::ast::UnaryPrimitive::Negate => {
                        format!(
                            "sub ({} 0, {} {})",
                            scalar.llvm,
                            scalar.llvm,
                            operand.representation()?
                        )
                    }
                    _ => return None,
                };
                Constant {
                    ty: operand.ty,
                    kind: ConstantKind::Value(representation),
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
        let representation = match &atom.kind {
            AtomKind::Integer(value) => {
                super::scalar::integer_literal(&atom.ty, *value, self.types.index_size())?
            }
            AtomKind::Float(bits) if atom.ty == Type::Float32 => {
                format!("0x{:016X}", (f32::from_bits(*bits as u32) as f64).to_bits())
            }
            AtomKind::Float(bits) if atom.ty == Type::Float64 => format!("0x{bits:016X}"),
            AtomKind::StorageSize(measured) if atom.ty == Type::ByteSize => {
                self.source_layouts.layout(measured)?.stride.to_string()
            }
            AtomKind::Symbol(bytes) if bytes.is_empty() => "null".into(),
            AtomKind::Symbol(bytes) => {
                let name = format!("mal_top_symbol_{}", atom.id.0);
                self.globals
                    .push_str(&super::symbol::literal_definition(&name, bytes));
                format!("@{name}")
            }
            AtomKind::Unit if atom.ty == Type::Unit => "0".into(),
            AtomKind::Reference(Reference::Binding(id)) => return values.get(id).cloned(),
            _ => return None,
        };
        Some(Constant {
            ty: atom.ty.clone(),
            kind: ConstantKind::Value(representation),
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

fn pattern_type(pattern: &Pattern) -> &Type {
    match pattern {
        Pattern::Binding { ty, .. }
        | Pattern::Wildcard { ty, .. }
        | Pattern::Product { ty, .. } => ty,
    }
}

fn bind_pattern(
    pattern: &Pattern,
    value: Constant,
    values: &mut HashMap<ValueId, Constant>,
) -> Option<()> {
    if *pattern_type(pattern) != value.ty {
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
    fn representation(&self) -> Option<&str> {
        let ConstantKind::Value(value) = &self.kind else {
            return None;
        };
        Some(value)
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

    pub(super) fn value(&self) -> Option<&str> {
        self.representation()
    }
}

fn numeric_conversion(operand: Constant, result_type: &Type, types: Types) -> Option<Constant> {
    let source = super::scalar::scalar_type(&operand.ty, types.index_size())?;
    let target = super::scalar::scalar_type(result_type, types.index_size())?;
    let representation = if source.floating == target.floating && source.bits == target.bits {
        operand.representation()?.into()
    } else if !source.floating && !target.floating {
        let value = operand.representation()?.parse::<i128>().ok()?;
        let modulus = 1_i128 << target.bits;
        let residue = value.rem_euclid(modulus);
        if target.signed && residue >= modulus / 2 {
            (residue - modulus).to_string()
        } else {
            residue.to_string()
        }
    } else {
        let instruction = if source.floating && target.floating {
            if source.bits > target.bits {
                "fptrunc"
            } else {
                "fpext"
            }
        } else if source.floating {
            if target.signed { "fptosi" } else { "fptoui" }
        } else if target.floating {
            if source.signed { "sitofp" } else { "uitofp" }
        } else if source.bits > target.bits {
            "trunc"
        } else if source.signed {
            "sext"
        } else {
            "zext"
        };
        format!(
            "{instruction} ({} {} to {})",
            source.llvm,
            operand.representation()?,
            target.llvm
        )
    };
    Some(Constant {
        ty: result_type.clone(),
        kind: ConstantKind::Value(representation),
    })
}

fn closure_binding_function(binding: &crate::closure::ast::TopLevelBinding) -> Option<FunctionId> {
    let AtomKind::Reference(Reference::Binding(result)) = binding.value.result.kind else {
        return None;
    };
    binding.value.bindings.iter().find_map(|binding| {
        let Pattern::Binding { id, .. } = binding.pattern else {
            return None;
        };
        match &binding.operation {
            crate::closure::ast::Operation::MakeClosure { function, captures }
                if id == result && captures.is_empty() =>
            {
                Some(*function)
            }
            _ => None,
        }
    })
}

pub(super) fn reachable_states(program: &Program, entry: StateId) -> Vec<StateId> {
    let mut pending = vec![entry];
    let mut seen = HashSet::new();
    let mut states = Vec::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        states.push(id);
        match &program.states[id.0].terminator {
            Terminator::Return(_) | Terminator::TailCall { .. } => {}
            Terminator::Goto(target) | Terminator::Jump { target, .. } => pending.push(*target),
            Terminator::Call { resume, .. } => pending.push(*resume),
            Terminator::Case { arms, .. } => pending.extend(arms.iter().map(|arm| arm.target)),
            Terminator::PrimitiveBranch {
                otherwise, then, ..
            } => pending.extend([*otherwise, *then]),
        }
    }
    states
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
                collect_pattern_slot(element, slots, types)?;
            }
        }
        Pattern::Wildcard { .. } => {}
        _ => return None,
    }
    Some(())
}

pub(super) fn collect_pattern_ids(pattern: &Pattern, ids: &mut Vec<ValueId>) {
    match pattern {
        Pattern::Binding { id, .. } => ids.push(*id),
        Pattern::Product { elements, .. } => {
            for element in elements {
                collect_pattern_ids(element, ids);
            }
        }
        Pattern::Wildcard { .. } => {}
    }
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

pub(super) fn pattern_value_type(pattern: &Pattern) -> Option<&Type> {
    match pattern {
        Pattern::Binding { ty, .. }
        | Pattern::Wildcard { ty, .. }
        | Pattern::Product { ty, .. } => Some(ty),
    }
}
