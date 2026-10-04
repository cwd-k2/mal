//! Resolution of type expressions, and the type parameters a declaration brings into scope.

use super::*;

impl Resolver {
    pub(super) fn resolve_type(
        &self,
        ty: &mal_syntax::ast::Node<mal_syntax::ast::TypeExpression>,
    ) -> Result<mal_syntax::ast::Node<ast::TypeExpression>, Diagnostic> {
        let kind = match &ty.kind {
            mal_syntax::ast::TypeExpression::Named(name) => {
                ast::TypeExpression::Named(self.type_reference(name)?)
            }
            mal_syntax::ast::TypeExpression::Application {
                constructor,
                arguments,
            } => ast::TypeExpression::Application {
                constructor: self.type_reference(constructor)?,
                arguments: arguments
                    .iter()
                    .map(|argument| self.resolve_type(argument))
                    .collect::<Result<_, _>>()?,
            },
            mal_syntax::ast::TypeExpression::Unit => ast::TypeExpression::Unit,
            mal_syntax::ast::TypeExpression::Parenthesized(inner) => {
                ast::TypeExpression::Parenthesized(Box::new(self.resolve_type(inner)?))
            }
            mal_syntax::ast::TypeExpression::Product(elements) => ast::TypeExpression::Product(
                elements
                    .iter()
                    .map(|element| self.resolve_type(element))
                    .collect::<Result<_, _>>()?,
            ),
            mal_syntax::ast::TypeExpression::Sum(members) => ast::TypeExpression::Sum(
                members
                    .iter()
                    .map(|member| self.resolve_type(member))
                    .collect::<Result<_, _>>()?,
            ),
            mal_syntax::ast::TypeExpression::Function { parameter, result } => {
                ast::TypeExpression::Function {
                    parameter: Box::new(self.resolve_type(parameter)?),
                    result: Box::new(self.resolve_type(result)?),
                }
            }
        };
        Ok(mal_syntax::ast::Node::new(kind, ty.span))
    }

    /// Brings the parameters of one declaration into type scope. A parameter may not reuse the name of a type, so
    /// every type name in a declaration means one thing.
    pub(super) fn push_type_parameters(
        &mut self,
        parameters: &[mal_syntax::ast::Name],
    ) -> Result<Vec<TypeBinding>, Diagnostic> {
        let mut bindings = Vec::with_capacity(parameters.len());
        let mut names = std::collections::HashSet::new();
        for name in parameters {
            if !names.insert(name.text.clone()) {
                self.pop_type_parameters(&bindings);
                return Err(Diagnostic::error("duplicate type parameter")
                    .with_primary(name.span, "this parameter is declared more than once"));
            }
            if self.types.contains_key(&name.text) {
                self.pop_type_parameters(&bindings);
                return Err(Diagnostic::error(format!(
                    "type parameter `{}` has the name of a type",
                    name.text
                ))
                .with_primary(name.span, "rename this parameter; it would hide the type"));
            }
            let binding = TypeBinding {
                id: ast::TypeId(self.next_type),
                name: name.clone(),
            };
            self.next_type += 1;
            self.types.insert(name.text.clone(), binding.clone());
            bindings.push(binding);
        }
        Ok(bindings)
    }

    pub(super) fn pop_type_parameters(&mut self, parameters: &[TypeBinding]) {
        for parameter in parameters {
            self.types.remove(&parameter.name.text);
        }
    }

    /// Resolves one declaration body with its type parameters in scope and restores the surrounding type scope on
    /// both success and failure.
    pub(super) fn with_type_parameters<T>(
        &mut self,
        parameters: &[mal_syntax::ast::Name],
        resolve: impl FnOnce(&mut Self) -> Result<T, Diagnostic>,
    ) -> Result<(Vec<TypeBinding>, T), Diagnostic> {
        let bindings = self.push_type_parameters(parameters)?;
        let result = resolve(self);
        self.pop_type_parameters(&bindings);
        result.map(|value| (bindings, value))
    }
}
