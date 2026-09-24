use std::collections::HashMap;

use crate::resolve::ast::{self as resolved, TypeBinding, ValueBinding};
use mal_syntax::ast;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::{FileId, SourceGraph, Span};

use super::{ExternalBinding, Resolver};

#[derive(Clone, Default)]
struct Exports {
    types: HashMap<String, TypeBinding>,
    externals: HashMap<String, ExternalBinding>,
    values: HashMap<String, ValueBinding>,
}

pub(super) fn resolve(
    graph: &SourceGraph,
    programs: &[ast::Program],
) -> Result<resolved::Program, Diagnostic> {
    if programs.len() != graph.files().len() {
        return Err(Diagnostic::error("source graph and parsed programs differ"));
    }
    let root_span = programs[graph.root().index() as usize].span;
    let mut resolver = FileResolver {
        graph,
        programs,
        resolver: Resolver::new(root_span),
        exports: vec![None; programs.len()],
        items: Vec::new(),
    };
    resolver.resolve_files(graph.root())?;
    Ok(resolved::Program {
        items: resolver.items,
        span: root_span,
    })
}

struct FileResolver<'a> {
    graph: &'a SourceGraph,
    programs: &'a [ast::Program],
    resolver: Resolver,
    exports: Vec<Option<Exports>>,
    items: Vec<ast::Node<resolved::TopItem>>,
}

impl FileResolver<'_> {
    fn resolve_files(&mut self, root: FileId) -> Result<(), Diagnostic> {
        let mut states = vec![0_u8; self.programs.len()];
        states[root.index() as usize] = 1;
        let mut pending = vec![(root, 0_usize)];

        while let Some((file, next_requirement)) = pending.last_mut() {
            let requirements = self.graph.requirements(*file);
            if let Some(requirement) = requirements.get(*next_requirement) {
                *next_requirement += 1;
                match states[requirement.target.index() as usize] {
                    0 => {
                        states[requirement.target.index() as usize] = 1;
                        pending.push((requirement.target, 0));
                    }
                    1 => {
                        return Err(Diagnostic::error("cyclic source graph")
                            .with_primary(requirement.span, "this dependency is still active"));
                    }
                    _ => {}
                }
                continue;
            }

            let (file, _) = pending.pop().expect("pending file exists");
            self.resolve_file(file)?;
            states[file.index() as usize] = 2;
        }
        Ok(())
    }

    fn resolve_file(&mut self, file: FileId) -> Result<(), Diagnostic> {
        let requirements = self.graph.requirements(file);
        let program = &self.programs[file.index() as usize];
        self.resolver.begin_file(program.span);
        for requirement in requirements {
            let exports = self.exports[requirement.target.index() as usize]
                .as_ref()
                .expect("dependency is resolved before its importer")
                .clone();
            self.import(&exports, requirement.span)?;
        }
        if file != self.graph.root() {
            reject_dependency_main(program)?;
        }
        self.resolver.predeclare_unit_names(program)?;

        let mut exports = Exports::default();
        for item in &program.items {
            let resolved = self.resolver.resolve_top_item(item)?;
            collect_exports(&mut exports, &resolved.kind);
            self.items.push(resolved);
        }
        self.exports[file.index() as usize] = Some(exports);
        Ok(())
    }

    fn import(&mut self, exports: &Exports, span: Span) -> Result<(), Diagnostic> {
        for (name, binding) in &exports.types {
            if self
                .resolver
                .types
                .insert(name.clone(), binding.clone())
                .is_some()
            {
                return Err(import_conflict(span, "type", name));
            }
        }
        for (name, binding) in &exports.externals {
            if self.resolver.externals.contains_key(name)
                || self.resolver.value_scopes[0].contains_key(name)
            {
                return Err(import_conflict(span, "top-level value", name));
            }
            self.resolver
                .externals
                .insert(name.clone(), binding.clone());
            self.resolver.value_scopes[0].insert(name.clone(), binding.binding.clone());
        }
        for (name, binding) in &exports.values {
            if self.resolver.externals.contains_key(name)
                || self.resolver.value_scopes[0].contains_key(name)
            {
                return Err(import_conflict(span, "top-level value", name));
            }
            self.resolver.value_scopes[0].insert(name.clone(), binding.clone());
        }
        Ok(())
    }
}

fn collect_exports(exports: &mut Exports, item: &resolved::TopItem) {
    match item {
        resolved::TopItem::TypeAlias { binding, .. }
        | resolved::TopItem::GenericTypeAlias { binding, .. }
        | resolved::TopItem::ExternalType { binding }
            if is_public(&binding.name.text) =>
        {
            exports
                .types
                .insert(binding.name.text.clone(), binding.clone());
        }
        resolved::TopItem::ExternalOperation {
            id,
            binding,
            lambda_id,
            ..
        } if is_public(&binding.name.text) => {
            exports.externals.insert(
                binding.name.text.clone(),
                ExternalBinding {
                    id: *id,
                    binding: binding.clone(),
                    lambda_id: *lambda_id,
                },
            );
        }
        resolved::TopItem::Binding(binding) => collect_pattern_exports(exports, &binding.pattern),
        resolved::TopItem::GenericBinding { binding, .. } if is_public(&binding.name.text) => {
            exports
                .values
                .insert(binding.name.text.clone(), binding.clone());
        }
        _ => {}
    }
}

fn collect_pattern_exports(exports: &mut Exports, pattern: &ast::Node<resolved::Pattern>) {
    let mut pending = vec![pattern];
    while let Some(pattern) = pending.pop() {
        match &pattern.kind {
            resolved::Pattern::Binding(binding) if is_public(&binding.name.text) => {
                exports
                    .values
                    .insert(binding.name.text.clone(), binding.clone());
            }
            resolved::Pattern::Product(elements) => pending.extend(elements.iter().rev()),
            _ => {}
        }
    }
}

fn reject_dependency_main(program: &ast::Program) -> Result<(), Diagnostic> {
    for item in &program.items {
        if let ast::TopItem::Binding(binding) = &item.kind
            && let Some(name) = pattern_name(&binding.pattern, "main")
        {
            return Err(Diagnostic::error("`main` declared outside the root file")
                .with_primary(name.span, "the entry point belongs in the root file"));
        }
    }
    Ok(())
}

fn pattern_name<'a>(pattern: &'a ast::Node<ast::Pattern>, expected: &str) -> Option<&'a ast::Name> {
    let mut pending = vec![pattern];
    while let Some(pattern) = pending.pop() {
        match &pattern.kind {
            ast::Pattern::Name(name) if name.text == expected => return Some(name),
            ast::Pattern::Product(elements) => pending.extend(elements.iter().rev()),
            _ => {}
        }
    }
    None
}

fn is_public(name: &str) -> bool {
    !name.starts_with('_')
}

fn import_conflict(span: Span, category: &str, name: &str) -> Diagnostic {
    Diagnostic::error(format!("duplicate imported {category} `{name}`"))
        .with_primary(span, "this requirement introduces a conflicting name")
}
