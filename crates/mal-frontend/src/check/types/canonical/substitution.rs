//! Iterative generic substitution into canonical type terms.

use std::collections::HashMap;

use crate::check::ast::{Kind, Type};
use crate::check::types::term::Normalizer;
use crate::resolve::ast::TypeId;
use mal_syntax::{diagnostic::Diagnostic, source::Span};

pub(in crate::check) fn substitute_type(
    ty: &Type,
    substitutions: &HashMap<TypeId, Type>,
    span: Span,
) -> Result<Type, Diagnostic> {
    let mut pending = vec![Substitution::Type(ty)];
    let mut values = Vec::new();
    let mut normalizer = Normalizer::new(span);
    while let Some(substitution) = pending.pop() {
        match substitution {
            Substitution::Type(ty) => match ty {
                Type::Parameter { id, .. } => {
                    values.push(substitutions.get(id).cloned().unwrap_or_else(|| ty.clone()))
                }
                Type::Application {
                    constructor,
                    argument,
                    span,
                    ..
                } => {
                    pending.push(Substitution::Application(*span));
                    pending.push(Substitution::Type(argument));
                    pending.push(Substitution::Type(constructor));
                }
                Type::Abstraction {
                    parameter_kind,
                    body,
                } => {
                    pending.push(Substitution::Abstraction(parameter_kind.clone()));
                    pending.push(Substitution::Type(body));
                }
                Type::Buffer(element) => {
                    pending.push(Substitution::Buffer);
                    pending.push(Substitution::Type(element));
                }
                Type::Opaque {
                    id,
                    name,
                    arguments,
                    representation,
                    declaration_file,
                } => {
                    pending.push(Substitution::Opaque {
                        id: *id,
                        name: name.clone(),
                        arguments: arguments.len(),
                        declaration_file: *declaration_file,
                    });
                    pending.push(Substitution::Type(representation));
                    pending.extend(arguments.iter().rev().map(Substitution::Type));
                }
                Type::Product(elements) => {
                    pending.push(Substitution::Product(elements.len()));
                    pending.extend(elements.iter().rev().map(Substitution::Type));
                }
                Type::Sum(members) => {
                    pending.push(Substitution::Sum(members.len()));
                    pending.extend(members.iter().rev().map(Substitution::Type));
                }
                Type::Function { parameter, result } => {
                    pending.push(Substitution::Function);
                    pending.push(Substitution::Type(result));
                    pending.push(Substitution::Type(parameter));
                }
                _ => values.push(ty.clone()),
            },
            Substitution::Application(span) => {
                let argument = values.pop().expect("application has an argument");
                let constructor = values.pop().expect("application has a constructor");
                values.push(normalizer.apply(constructor, argument, span)?);
            }
            Substitution::Abstraction(parameter_kind) => {
                let body = values.pop().expect("abstraction has a body");
                values.push(normalizer.abstraction(parameter_kind, body)?);
            }
            Substitution::Buffer => {
                let element = values.pop().expect("Buffer has an element type");
                values.push(Type::Buffer(element.into()));
            }
            Substitution::Opaque {
                id,
                name,
                arguments,
                declaration_file,
            } => {
                let representation = values.pop().expect("opaque type has a representation");
                let arguments = take_last(&mut values, arguments);
                values.push(Type::Opaque {
                    id,
                    name,
                    arguments: arguments.into(),
                    representation: representation.into(),
                    declaration_file,
                });
            }
            Substitution::Product(length) => {
                let elements = take_last(&mut values, length);
                values.push(Type::Product(elements.into()));
            }
            Substitution::Sum(length) => {
                let members = take_last(&mut values, length);
                values.push(Type::Sum(members.into()));
            }
            Substitution::Function => {
                let result = values.pop().expect("function has a result type");
                let parameter = values.pop().expect("function has a parameter type");
                values.push(Type::Function {
                    parameter: parameter.into(),
                    result: result.into(),
                });
            }
        }
    }
    let [value] = values
        .try_into()
        .expect("one substitution root produces one type");
    Ok(value)
}

enum Substitution<'a> {
    Type(&'a Type),
    Application(Span),
    Abstraction(Kind),
    Buffer,
    Opaque {
        id: TypeId,
        name: std::sync::Arc<str>,
        arguments: usize,
        declaration_file: mal_syntax::source::FileId,
    },
    Product(usize),
    Sum(usize),
    Function,
}

fn take_last(values: &mut Vec<Type>, length: usize) -> Vec<Type> {
    values.split_off(values.len() - length)
}
