use mal_syntax::diagnostic::Diagnostic;

use super::ast::{self, Binding, Pattern, Type};

pub(super) fn entry_point(binding: &Binding) -> Result<Option<ast::EntryPoint>, Diagnostic> {
    let Pattern::Binding { binding: name, ty } = &binding.pattern else {
        return match product_bound_main(&binding.pattern) {
            Some(name) => Err(
                Diagnostic::error("invalid entry point binding").with_primary(
                    name.name.span,
                    "bind `main` by its own name, not in a product",
                ),
            ),
            None => Ok(None),
        };
    };
    if name.name.text != "main" {
        return Ok(None);
    }
    let Type::Function { parameter, result } = ty else {
        return Err(Diagnostic::error("invalid entry point type").with_primary(
            name.name.span,
            "expected `Unit -> Int32` or `Buffer<Symbol> -> Int32`",
        ));
    };
    let parameter = if **parameter == Type::Unit {
        ast::EntryParameter::Unit
    } else if **parameter == ast::EntryParameter::process_arguments_type() {
        ast::EntryParameter::ProcessArguments
    } else {
        return Err(Diagnostic::error("invalid entry point type").with_primary(
            name.name.span,
            "expected `Unit -> Int32` or `Buffer<Symbol> -> Int32`",
        ));
    };
    if **result != Type::Int32 {
        return Err(Diagnostic::error("invalid entry point type").with_primary(
            name.name.span,
            "expected `Unit -> Int32` or `Buffer<Symbol> -> Int32`",
        ));
    }
    Ok(Some(ast::EntryPoint {
        binding: name.id,
        parameter,
    }))
}

fn product_bound_main(pattern: &Pattern) -> Option<&crate::resolve::ast::ValueBinding> {
    let mut pending = vec![pattern];
    while let Some(pattern) = pending.pop() {
        match pattern {
            Pattern::Binding { binding, .. } if binding.name.text == "main" => {
                return Some(binding);
            }
            Pattern::Product { elements, .. } => pending.extend(elements.iter().rev()),
            Pattern::Binding { .. } | Pattern::Wildcard { .. } => {}
        }
    }
    None
}
