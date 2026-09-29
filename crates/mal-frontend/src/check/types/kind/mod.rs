use std::collections::HashMap;

use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;

use crate::check::ast::Kind;
use crate::resolve::ast::{self as resolved, TypeBinding, TypeExpression, TypeId};

use self::solver::{InferredKind, Solver};

mod admission;
mod expression;
mod graph;
mod scheme;
mod solver;

#[derive(Clone, Default)]
pub(in crate::check) struct Kinds {
    pub(super) declarations: HashMap<TypeId, Kind>,
}

struct Declaration<'a> {
    binding: &'a TypeBinding,
    parameters: &'a [TypeBinding],
    body: Option<&'a Node<TypeExpression>>,
    result_is_type: bool,
}

impl Kinds {
    pub(in crate::check) fn infer(program: &resolved::Program) -> Result<Self, Diagnostic> {
        let declarations = program
            .items
            .iter()
            .filter_map(|item| match &item.kind {
                resolved::TopItem::TypeAlias { binding, value } => Some(Declaration {
                    binding,
                    parameters: &[],
                    body: Some(value),
                    result_is_type: false,
                }),
                resolved::TopItem::GenericTypeAlias {
                    binding,
                    parameters,
                    value,
                } => Some(Declaration {
                    binding,
                    parameters,
                    body: Some(value),
                    result_is_type: false,
                }),
                resolved::TopItem::OpaqueType {
                    binding,
                    parameters,
                    representation,
                } => Some(Declaration {
                    binding,
                    parameters,
                    body: Some(representation),
                    result_is_type: true,
                }),
                resolved::TopItem::ExternalType { binding } => Some(Declaration {
                    binding,
                    parameters: &[],
                    body: None,
                    result_is_type: true,
                }),
                _ => None,
            })
            .collect::<Vec<_>>();
        let positions = declarations
            .iter()
            .enumerate()
            .map(|(index, declaration)| (declaration.binding.id, index))
            .collect::<HashMap<_, _>>();
        for declaration in &declarations {
            admission::parameters(declaration.parameters.len(), declaration.binding.name.span)?;
        }
        let graph = declarations
            .iter()
            .map(|declaration| {
                declaration
                    .body
                    .map(graph::type_references)
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|id| positions.get(&id).copied())
                    .collect()
            })
            .collect::<Vec<_>>();

        let mut kinds = Self::default();
        for component in graph::components(&graph) {
            kinds.infer_component(&declarations, &component)?;
        }
        Ok(kinds)
    }

    pub(super) fn declaration(&self, id: TypeId, next: &mut u32) -> Option<Kind> {
        self.declarations
            .get(&id)
            .map(|kind| scheme::freshen_all(std::slice::from_ref(kind), next).remove(0))
    }

    pub(in crate::check) fn parameters(
        &self,
        parameters: &[TypeBinding],
        expression: &Node<TypeExpression>,
        next: &mut u32,
    ) -> Result<Vec<Kind>, Diagnostic> {
        admission::parameters(parameters.len(), expression.span)?;
        let mut solver = Solver::default();
        let locals = parameters
            .iter()
            .map(|parameter| (parameter.id, solver.fresh()))
            .collect::<HashMap<_, _>>();
        let inferred = self.infer_expression(expression, &locals, &HashMap::new(), &mut solver)?;
        solver.unify(inferred, InferredKind::Type, expression.span)?;
        let kinds = solver.generalize_all(
            &parameters
                .iter()
                .map(|parameter| locals[&parameter.id].clone())
                .collect::<Vec<_>>(),
            expression.span,
        )?;
        Ok(scheme::freshen_all(&kinds, next))
    }

    fn infer_component(
        &mut self,
        declarations: &[Declaration<'_>],
        component: &[usize],
    ) -> Result<(), Diagnostic> {
        let mut solver = Solver::default();
        let mut heads = HashMap::new();
        let mut locals = HashMap::new();
        let mut results = HashMap::new();
        for index in component {
            let declaration = &declarations[*index];
            let parameters = declaration
                .parameters
                .iter()
                .map(|parameter| (parameter.id, solver.fresh()))
                .collect::<HashMap<_, _>>();
            let result = if declaration.result_is_type {
                InferredKind::Type
            } else {
                solver.fresh()
            };
            let head = declaration
                .parameters
                .iter()
                .rev()
                .fold(result.clone(), |result, parameter| {
                    Solver::function(parameters[&parameter.id].clone(), result)
                });
            heads.insert(declaration.binding.id, head);
            locals.insert(declaration.binding.id, parameters);
            results.insert(declaration.binding.id, result);
        }
        for index in component {
            let declaration = &declarations[*index];
            if let Some(body) = declaration.body {
                let inferred = self.infer_expression(
                    body,
                    &locals[&declaration.binding.id],
                    &heads,
                    &mut solver,
                )?;
                solver.unify(
                    inferred,
                    results[&declaration.binding.id].clone(),
                    body.span,
                )?;
            }
        }
        for index in component {
            let id = declarations[*index].binding.id;
            let span = declarations[*index].binding.name.span;
            self.declarations
                .insert(id, solver.generalize(heads[&id].clone(), span)?);
        }
        Ok(())
    }
}
