//! C type syntax.

use proc_macro::{Delimiter, TokenStream, TokenTree};

use crate::shared::Cursor;

use super::support::*;

pub(super) fn c_type_syntax(input: TokenStream) -> Result<TokenStream, String> {
    let mut input = Cursor::new(input);
    let ty = c_type_tokens(&mut input)?;
    if input.is_empty() {
        Ok(ty)
    } else {
        Err("unexpected token after C type".into())
    }
}

pub(super) fn c_type_tokens(input: &mut Cursor) -> Result<TokenStream, String> {
    if input.eat_punct('*') {
        let is_const = match input.next() {
            Some(TokenTree::Ident(qualifier)) if qualifier.to_string() == "const" => true,
            Some(TokenTree::Ident(qualifier)) if qualifier.to_string() == "mut" => false,
            _ => return Err("expected `const` or `mut` after `*` in C type".into()),
        };
        let pointee = c_type_tokens(input)?;
        return Ok(rust_method_call(
            pointee,
            if is_const {
                "const_pointee_pointer"
            } else {
                "pointer"
            },
        ));
    }

    match input.next() {
        Some(TokenTree::Ident(identifier)) if identifier.to_string() == "Struct" => {
            if !input.eat_punct('<') {
                return Err("expected `<` after `Struct`".into());
            }
            let name = match input.next() {
                Some(TokenTree::Ident(name)) => format!("{:?}", name.to_string()).parse().unwrap(),
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                    group.stream()
                }
                _ => return Err("expected C struct tag".into()),
            };
            if !input.eat_punct('>') {
                return Err("expected `>` after C struct tag".into());
            }
            Ok(rust_call(
                "crate::backend::c::syntax::TypeName::structure",
                [name],
            ))
        }
        Some(TokenTree::Ident(identifier)) => Ok(rust_call(
            "crate::backend::c::syntax::TypeName::named",
            [
                format!("{:?}", identifier.to_string().trim_start_matches("r#"))
                    .parse()
                    .unwrap(),
            ],
        )),
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => Ok(rust_call(
            "crate::backend::c::syntax::TypeName::from",
            [group.stream()],
        )),
        _ => Err("expected C type".into()),
    }
}
