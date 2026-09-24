use std::collections::{HashMap, HashSet};

use crate::diagnostic::Diagnostic;
use crate::resolve::ast::TypeId;
use crate::source::Span;

use super::{Type, type_name};

const MAX_REPRESENTATION_UNITS: usize = 65_536;
const MAX_REPRESENTATION_DEPTH: usize = 64;

pub(in crate::check) fn ensure_memory_representable(
    ty: &Type,
    span: Span,
) -> Result<(), Diagnostic> {
    if is_memory_representable(ty) {
        return Ok(());
    }
    let offending = first_nonrepresentable_type(ty).unwrap_or(ty);
    Err(
        Diagnostic::error("memory element type is not representable").with_primary(
            span,
            format!(
                "`{}` has no canonical memory representation",
                type_name(offending)
            ),
        ),
    )
}

fn first_nonrepresentable_type(ty: &Type) -> Option<&Type> {
    let mut pending = vec![ty];
    let mut visited = HashSet::new();
    while let Some(ty) = pending.pop() {
        if ty.shared_id().is_some_and(|id| !visited.insert(id)) {
            continue;
        }
        match ty {
            Type::Product(elements) | Type::Sum(elements) if !elements.is_empty() => {
                pending.extend(elements.iter().rev());
            }
            Type::Symbol
            | Type::External { .. }
            | Type::Function { .. }
            | Type::Buffer(_)
            | Type::Sum(_) => return Some(ty),
            _ => {}
        }
    }
    None
}

pub(in crate::check) fn is_memory_representable(ty: &Type) -> bool {
    let mut pending = vec![ty];
    let mut visited = HashSet::new();
    while let Some(ty) = pending.pop() {
        if ty.shared_id().is_some_and(|id| !visited.insert(id)) {
            continue;
        }
        match ty {
            Type::Unit
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
            | Type::Address
            | Type::ByteSize
            | Type::USize
            | Type::Parameter { .. } => {}
            Type::Product(elements) => pending.extend(elements.iter()),
            Type::Sum(members) if !members.is_empty() => pending.extend(members.iter()),
            Type::Symbol
            | Type::External { .. }
            | Type::Function { .. }
            | Type::Buffer(_)
            | Type::Sum(_) => return false,
        }
    }
    true
}

pub(in crate::check) fn representable_requirements(ty: &Type) -> HashSet<TypeId> {
    let mut requirements = HashSet::new();
    let mut pending = vec![(ty, false)];
    while let Some((ty, required)) = pending.pop() {
        match ty {
            Type::Parameter { id, .. } if required => {
                requirements.insert(*id);
            }
            Type::Buffer(element) => {
                pending.push((element, true));
            }
            Type::Product(elements) | Type::Sum(elements) => {
                pending.extend(elements.iter().map(|element| (element, required)));
            }
            Type::Function { parameter, result } => {
                pending.push((parameter, required));
                pending.push((result, required));
            }
            _ => {}
        }
    }
    requirements
}

pub(in crate::check) fn satisfies_representable_requirement(
    ty: &Type,
    available: &HashSet<TypeId>,
) -> bool {
    let mut pending = vec![ty];
    while let Some(ty) = pending.pop() {
        match ty {
            Type::Parameter { id, .. } => {
                if !available.contains(id) {
                    return false;
                }
            }
            Type::Product(elements) => pending.extend(elements.iter()),
            Type::Sum(members) if !members.is_empty() => pending.extend(members.iter()),
            Type::Unit
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
            | Type::Address
            | Type::ByteSize
            | Type::USize => {}
            _ => return false,
        }
    }
    true
}

pub(in crate::check) fn ensure_representable(ty: &Type, span: Span) -> Result<(), Diagnostic> {
    if representation_units(ty).is_none() {
        return Err(
            Diagnostic::error("type representation is too large").with_primary(
                span,
                format!(
                    "malc supports at most {MAX_REPRESENTATION_DEPTH} nested levels and {MAX_REPRESENTATION_UNITS} storage components"
                ),
            ),
        );
    }
    Ok(())
}

fn representation_units(ty: &Type) -> Option<usize> {
    let mut pending = vec![Representation::Type(ty)];
    let mut values = Vec::new();
    let mut cache = HashMap::new();
    while let Some(item) = pending.pop() {
        match item {
            Representation::Type(ty) => {
                if let Some(measure) = ty.shared_id().and_then(|id| cache.get(&id).copied()) {
                    values.push(measure);
                    continue;
                }
                match ty {
                    Type::Product(elements) => {
                        pending.push(Representation::Product(ty.shared_id(), elements.len()));
                        pending.extend(elements.iter().rev().map(Representation::Type));
                    }
                    Type::Sum(elements) => {
                        pending.push(Representation::Sum(ty.shared_id(), elements.len()));
                        pending.extend(elements.iter().rev().map(Representation::Type));
                    }
                    Type::Function { parameter, result } => {
                        pending.push(Representation::Function(ty.shared_id()));
                        pending.push(Representation::Type(result));
                        pending.push(Representation::Type(parameter));
                    }
                    _ => values.push(RepresentationMeasure { units: 1, depth: 0 }),
                }
            }
            Representation::Product(id, length) => {
                let children = take_measures(&mut values, length);
                let units = children
                    .iter()
                    .try_fold(0usize, |total, measure| total.checked_add(measure.units))?;
                let measure = composite_measure(units, &children)?;
                store_measure(id, measure, &mut cache);
                values.push(measure);
            }
            Representation::Sum(id, length) => {
                let children = take_measures(&mut values, length);
                let units = children
                    .iter()
                    .map(|measure| measure.units)
                    .max()
                    .unwrap_or(0)
                    .checked_add(1)?;
                let measure = composite_measure(units, &children)?;
                store_measure(id, measure, &mut cache);
                values.push(measure);
            }
            Representation::Function(id) => {
                let children = take_measures(&mut values, 2);
                let units = children
                    .iter()
                    .map(|measure| measure.units)
                    .max()
                    .unwrap_or(2)
                    .max(2);
                let depth = children
                    .iter()
                    .map(|measure| measure.depth)
                    .max()
                    .unwrap_or(0);
                let measure = (units <= MAX_REPRESENTATION_UNITS
                    && depth <= MAX_REPRESENTATION_DEPTH)
                    .then_some(RepresentationMeasure { units, depth })?;
                store_measure(id, measure, &mut cache);
                values.push(measure);
            }
        }
    }
    let [measure] = values.try_into().ok()?;
    Some(measure.units)
}

enum Representation<'a> {
    Type(&'a Type),
    Product(Option<super::super::ast::SharedTypeId>, usize),
    Sum(Option<super::super::ast::SharedTypeId>, usize),
    Function(Option<super::super::ast::SharedTypeId>),
}

#[derive(Clone, Copy)]
struct RepresentationMeasure {
    units: usize,
    depth: usize,
}

fn take_measures(
    values: &mut Vec<RepresentationMeasure>,
    length: usize,
) -> Vec<RepresentationMeasure> {
    values.split_off(
        values
            .len()
            .checked_sub(length)
            .expect("all child units exist"),
    )
}

fn composite_measure(
    units: usize,
    children: &[RepresentationMeasure],
) -> Option<RepresentationMeasure> {
    let depth = children
        .iter()
        .map(|measure| measure.depth)
        .max()
        .unwrap_or(0)
        .checked_add(1)?;
    (units <= MAX_REPRESENTATION_UNITS && depth <= MAX_REPRESENTATION_DEPTH)
        .then_some(RepresentationMeasure { units, depth })
}

fn store_measure(
    id: Option<super::super::ast::SharedTypeId>,
    measure: RepresentationMeasure,
    cache: &mut HashMap<super::super::ast::SharedTypeId, RepresentationMeasure>,
) {
    if let Some(id) = id {
        cache.insert(id, measure);
    }
}
