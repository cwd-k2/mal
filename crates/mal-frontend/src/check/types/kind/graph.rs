use std::collections::HashSet;

use mal_syntax::ast::Node;

use crate::resolve::ast::{TypeExpression, TypeId};

pub(super) fn components(graph: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let mut finished = vec![false; graph.len()];
    let mut order = Vec::with_capacity(graph.len());
    for root in 0..graph.len() {
        if finished[root] {
            continue;
        }
        let mut active = HashSet::new();
        let mut stack = vec![(root, 0)];
        active.insert(root);
        while let Some((node, next)) = stack.last_mut() {
            if *next < graph[*node].len() {
                let target = graph[*node][*next];
                *next += 1;
                if !finished[target] && active.insert(target) {
                    stack.push((target, 0));
                }
            } else {
                let (node, _) = stack.pop().expect("active kind traversal has a node");
                active.remove(&node);
                if !finished[node] {
                    finished[node] = true;
                    order.push(node);
                }
            }
        }
    }

    let mut reverse = vec![Vec::new(); graph.len()];
    for (source, targets) in graph.iter().enumerate() {
        for target in targets {
            reverse[*target].push(source);
        }
    }
    let mut assigned = vec![false; graph.len()];
    let mut result = Vec::new();
    for root in order.into_iter().rev() {
        if assigned[root] {
            continue;
        }
        assigned[root] = true;
        let mut component = Vec::new();
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            component.push(node);
            for target in &reverse[node] {
                if !assigned[*target] {
                    assigned[*target] = true;
                    pending.push(*target);
                }
            }
        }
        result.push(component);
    }
    result.reverse();
    result
}

pub(super) fn type_references(root: &Node<TypeExpression>) -> HashSet<TypeId> {
    let mut references = HashSet::new();
    let mut pending = vec![root];
    while let Some(expression) = pending.pop() {
        match &expression.kind {
            TypeExpression::Named(reference) => {
                references.insert(reference.id);
            }
            TypeExpression::Application {
                constructor,
                arguments,
            } => {
                references.insert(constructor.id);
                pending.extend(arguments);
            }
            TypeExpression::Parenthesized(inner) => pending.push(inner),
            TypeExpression::Product(elements) | TypeExpression::Sum(elements) => {
                pending.extend(elements)
            }
            TypeExpression::Function { parameter, result } => {
                pending.push(parameter);
                pending.push(result);
            }
            TypeExpression::Unit => {}
        }
    }
    references
}
