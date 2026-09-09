use std::collections::HashMap;

use crate::anf::ast::ValueId;
use crate::closure::ast::{self as closure, AtomKind, FunctionId, Pattern, Reference};

mod tail;

use tail::stable_symbol_slots;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit::body) struct SymbolAtCursorPlan {
    sites: HashMap<FunctionId, Vec<CursorSite>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CursorSite {
    product: ValueId,
    index: usize,
}

impl SymbolAtCursorPlan {
    pub(in crate::c_emit::body) fn new(program: &closure::Program) -> Self {
        let mut sites = HashMap::new();
        let mut next_index = 0;
        for function in &program.functions {
            let Some(stable_symbols) = stable_symbol_slots(function) else {
                continue;
            };
            let mut function_sites = Vec::new();
            collect_sites(
                &function.body,
                &stable_symbols,
                &mut next_index,
                &mut function_sites,
            );
            if !function_sites.is_empty() {
                sites.insert(function.id, function_sites);
            }
        }
        Self { sites }
    }

    pub(in crate::c_emit::body) fn sites(
        &self,
        function: FunctionId,
    ) -> impl Iterator<Item = (ValueId, usize)> + '_ {
        self.sites
            .get(&function)
            .into_iter()
            .flatten()
            .map(|site| (site.product, site.index))
    }

    pub(in crate::c_emit::body) fn is_valid(&self, program: &closure::Program) -> bool {
        self == &Self::new(program)
    }
}

fn collect_sites(
    block: &closure::Block,
    stable_symbols: &[ValueId],
    next_index: &mut usize,
    sites: &mut Vec<CursorSite>,
) {
    for pair in block.bindings.windows(2) {
        let [product, consumer] = pair else {
            unreachable!("windows have the requested length")
        };
        let (
            Pattern::Binding { id: product_id, .. },
            closure::Operation::Product(elements),
            closure::Operation::SymbolAt { argument },
        ) = (&product.pattern, &product.operation, &consumer.operation)
        else {
            continue;
        };
        if elements.len() != 2
            || !matches!(
                elements[0].kind,
                AtomKind::Reference(Reference::Binding(id)) if stable_symbols.contains(&id)
            )
            || !matches!(
                argument.kind,
                AtomKind::Reference(Reference::Binding(id)) if id == *product_id
            )
        {
            continue;
        }
        sites.push(CursorSite {
            product: *product_id,
            index: *next_index,
        });
        *next_index += 1;
    }
    for binding in &block.bindings {
        match &binding.operation {
            closure::Operation::Case { arms, .. } => {
                for arm in arms {
                    collect_sites(&arm.value, stable_symbols, next_index, sites);
                }
            }
            closure::Operation::PrimitiveBranch {
                otherwise, then, ..
            } => {
                collect_sites(otherwise, stable_symbols, next_index, sites);
                collect_sites(then, stable_symbols, next_index, sites);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
