//! Binding evaluated constants to the names a pattern introduces.

use super::*;

impl TopLevelConstants {
    pub(super) fn bind_local_pattern(
        &self,
        pattern: &Pattern,
        value: Constant,
        values: &mut HashMap<ValueId, Constant>,
    ) -> Option<()> {
        bind_pattern(pattern, value, values)
    }

    pub(super) fn bind_top_pattern(
        &mut self,
        pattern: &TopLevelPattern,
        value: Constant,
    ) -> Option<()> {
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
