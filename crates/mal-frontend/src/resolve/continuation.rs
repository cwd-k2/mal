use mal_syntax::ast;
use mal_syntax::diagnostic::Diagnostic;

use super::Resolver;
use super::ast::{Branch, Continuation};

impl Resolver {
    /// Resolves the continuations of a continuation application. Only a sum elimination (two or more
    /// continuations) turns a lambda literal into a branch of the enclosing invocation.
    pub(super) fn resolve_continuations(
        &mut self,
        continuations: &[ast::Node<ast::Expression>],
    ) -> Result<Vec<Continuation>, Diagnostic> {
        let is_sum_elimination = continuations.len() >= 2;
        continuations
            .iter()
            .map(|continuation| {
                if is_sum_elimination
                    && self.current_lambda.is_some()
                    && let Some(lambda) = lambda_literal(continuation)
                {
                    return self
                        .resolve_branch(lambda, continuation.span)
                        .map(Continuation::Branch);
                }
                Ok(Continuation::Function(
                    self.resolve_expression(continuation)?,
                ))
            })
            .collect()
    }

    fn resolve_branch(
        &mut self,
        lambda: &ast::Lambda,
        span: mal_syntax::source::Span,
    ) -> Result<Branch, Diagnostic> {
        let owner = self.local_owner(span)?;
        self.push_scope();
        let result = (|| {
            let parameter = lambda
                .parameter
                .as_ref()
                .map(|parameter| self.declare_pattern(parameter, owner))
                .transpose()?
                .map(Box::new);
            let body = self.resolve_expression_block_contents(&lambda.body)?;
            Ok(Branch {
                parameter,
                body,
                span,
            })
        })();
        self.pop_scope();
        result
    }
}

/// Parentheses carry no meaning, so a parenthesized lambda literal is still a lambda literal.
fn lambda_literal(expression: &ast::Node<ast::Expression>) -> Option<&ast::Lambda> {
    let mut current = expression;
    while let ast::Expression::Parenthesized(inner) = &current.kind {
        current = inner;
    }
    match &current.kind {
        ast::Expression::Lambda(lambda) => Some(lambda),
        _ => None,
    }
}
