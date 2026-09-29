//! Bounded canonical representation measurement for checked value types.

use std::collections::HashMap;

use mal_syntax::{diagnostic::Diagnostic, source::Span};

use super::super::ast::{SharedTypeId, Type};

const MAX_REPRESENTATION_UNITS: usize = 65_536;
const MAX_REPRESENTATION_DEPTH: usize = 64;

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
                    Type::Opaque { representation, .. } => {
                        pending.push(Representation::Type(representation));
                    }
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
    Product(Option<SharedTypeId>, usize),
    Sum(Option<SharedTypeId>, usize),
    Function(Option<SharedTypeId>),
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
    id: Option<SharedTypeId>,
    measure: RepresentationMeasure,
    cache: &mut HashMap<SharedTypeId, RepresentationMeasure>,
) {
    if let Some(id) = id {
        cache.insert(id, measure);
    }
}
