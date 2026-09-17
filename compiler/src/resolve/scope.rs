use crate::ast;
use crate::diagnostic::Diagnostic;

use super::ast::{
    ExternalOperationId, TypeBinding, TypeId, TypeReference, ValueBinding, ValueId, ValueOwner,
};
use super::{ExternalBinding, Resolver};

impl Resolver {
    pub(super) fn predeclare_unit_names(
        &mut self,
        program: &ast::Program,
    ) -> Result<(), Diagnostic> {
        for item in &program.items {
            match &item.kind {
                ast::TopItem::TypeAlias { name, .. }
                | ast::TopItem::GenericTypeAlias { name, .. }
                | ast::TopItem::ExternalType { name } => {
                    if self.types.contains_key(&name.text) {
                        return Err(self.duplicate(name, "type"));
                    }
                    let binding = TypeBinding {
                        id: TypeId(self.next_type),
                        name: name.clone(),
                    };
                    self.next_type += 1;
                    self.types.insert(name.text.clone(), binding);
                }
                ast::TopItem::ExternalOperation { name, .. } => {
                    if self.externals.contains_key(&name.text)
                        || self.value_scopes[0].contains_key(&name.text)
                    {
                        return Err(self.duplicate(name, "top-level value"));
                    }
                    let value = self.allocate_value_binding(name, ValueOwner::TopLevel);
                    let binding = ExternalBinding {
                        id: ExternalOperationId(self.next_external),
                        binding: value.clone(),
                        lambda_id: self.allocate_lambda(),
                    };
                    self.next_external += 1;
                    self.value_scopes[0].insert(name.text.clone(), value);
                    self.externals.insert(name.text.clone(), binding);
                }
                ast::TopItem::Binding(_) | ast::TopItem::GenericBinding { .. } => {}
            }
        }
        Ok(())
    }

    pub(super) fn declare_pattern(
        &mut self,
        pattern: &ast::Node<ast::Pattern>,
        owner: ValueOwner,
    ) -> Result<ast::Node<super::ast::Pattern>, Diagnostic> {
        let kind = match &pattern.kind {
            ast::Pattern::Name(name) => {
                super::ast::Pattern::Binding(self.declare_value(name, owner)?)
            }
            ast::Pattern::Wildcard => super::ast::Pattern::Wildcard,
            ast::Pattern::Product(elements) => super::ast::Pattern::Product(
                elements
                    .iter()
                    .map(|element| self.declare_pattern(element, owner))
                    .collect::<Result<_, _>>()?,
            ),
        };
        Ok(ast::Node::new(kind, pattern.span))
    }

    pub(super) fn declare_value(
        &mut self,
        name: &ast::Name,
        owner: ValueOwner,
    ) -> Result<ValueBinding, Diagnostic> {
        if self
            .value_scopes
            .last()
            .expect("value scope")
            .contains_key(&name.text)
            || (owner == ValueOwner::TopLevel && self.externals.contains_key(&name.text))
        {
            return Err(self.duplicate(name, "value"));
        }
        let binding = self.allocate_value_binding(name, owner);
        self.value_scopes
            .last_mut()
            .expect("value scope")
            .insert(name.text.clone(), binding.clone());
        Ok(binding)
    }

    pub(super) fn allocate_value_binding(
        &mut self,
        name: &ast::Name,
        owner: ValueOwner,
    ) -> ValueBinding {
        let binding = ValueBinding {
            id: ValueId(self.next_value),
            name: name.clone(),
            owner,
        };
        self.next_value += 1;
        binding
    }

    pub(super) fn type_binding(&self, name: &ast::Name) -> Result<TypeBinding, Diagnostic> {
        self.types
            .get(&name.text)
            .cloned()
            .ok_or_else(|| self.unknown(name, "type"))
    }

    pub(super) fn type_reference(&self, name: &ast::Name) -> Result<TypeReference, Diagnostic> {
        Ok(TypeReference {
            id: self.type_binding(name)?.id,
            name: name.clone(),
        })
    }

    pub(super) fn lookup_value(&self, text: &str) -> Option<ValueBinding> {
        self.value_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(text).cloned())
    }

    pub(super) fn push_scope(&mut self) {
        self.value_scopes.push(std::collections::HashMap::new());
    }

    pub(super) fn pop_scope(&mut self) {
        self.value_scopes.pop().expect("a nested value scope");
    }

    pub(super) fn add_predefined_type(&mut self, text: &str, id: TypeId) {
        self.types.insert(
            text.into(),
            TypeBinding {
                id,
                name: self.synthetic_name(text),
            },
        );
    }

    pub(super) fn add_predefined_value(&mut self, text: &str, id: ValueId) {
        let name = self.synthetic_name(text);
        self.value_scopes[0].insert(
            text.into(),
            ValueBinding {
                id,
                name,
                owner: ValueOwner::Predefined,
            },
        );
    }

    fn synthetic_name(&self, text: &str) -> ast::Name {
        ast::Name {
            text: text.into(),
            span: self.synthetic_span,
        }
    }

    fn duplicate(&self, name: &ast::Name, category: &str) -> Diagnostic {
        Diagnostic::error(format!("duplicate {category} `{}`", name.text))
            .with_primary(name.span, "already declared in this scope")
    }

    pub(super) fn unknown(&self, name: &ast::Name, category: &str) -> Diagnostic {
        Diagnostic::error(format!("unknown {category} `{}`", name.text))
            .with_primary(name.span, format!("this {category} is not in scope"))
    }
}
