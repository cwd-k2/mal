//! Source type expansion into canonical kinded type terms.

use crate::resolve::ast::{
    self as resolved, ADDRESS_TYPE, BOOL_TYPE, BUFFER_TYPE, BYTE_SIZE_TYPE, FLOAT32_TYPE,
    FLOAT64_TYPE, INT8_TYPE, INT16_TYPE, INT32_TYPE, INT64_TYPE, SYMBOL_TYPE, TypeId, U_SIZE_TYPE,
    UINT8_TYPE, UINT16_TYPE, UINT32_TYPE, UINT64_TYPE, UNIT_TYPE,
};
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::super::{Checker, ast::Kind, ast::KindRequirement, ast::Type};
use super::{ensure_representable, term};

pub(in crate::check) fn require_type_argument_kinds(
    parameters: &[Kind],
    arguments: &mut [Type],
    span: Span,
) -> Result<Vec<KindRequirement>, Diagnostic> {
    term::normalize_argument_kinds(parameters, arguments, span)
}

pub(in crate::check) fn instantiate_signature_kinds(
    parameters: &[Kind],
    arguments: &[Type],
    ty: &Type,
    span: Span,
) -> Result<(Vec<Kind>, Type, Vec<KindRequirement>), Diagnostic> {
    term::instantiate_kinds(parameters, arguments, ty, span)
}

impl Checker {
    pub(in crate::check) fn expand_type(
        &mut self,
        ty: &Node<resolved::TypeExpression>,
    ) -> Result<Type, Diagnostic> {
        let expanded = self.expand([Expansion::Expression(
            ty.clone(),
            self.type_substitutions.clone(),
        )])?;
        ensure_value_type(&expanded, ty.span)?;
        ensure_representable(&expanded, ty.span)?;
        Ok(expanded)
    }

    pub(in crate::check) fn expand_type_term(
        &mut self,
        ty: &Node<resolved::TypeExpression>,
    ) -> Result<Type, Diagnostic> {
        self.expand([Expansion::Expression(
            ty.clone(),
            self.type_substitutions.clone(),
        )])
    }

    pub(in crate::check) fn expand_type_id(
        &mut self,
        id: TypeId,
        use_span: Span,
    ) -> Result<Type, Diagnostic> {
        let expanded = self.expand_term_id(id, use_span)?;
        ensure_value_type(&expanded, use_span)?;
        ensure_representable(&expanded, use_span)?;
        Ok(expanded)
    }

    pub(in crate::check) fn expand_term_id(
        &mut self,
        id: TypeId,
        use_span: Span,
    ) -> Result<Type, Diagnostic> {
        self.expand([Expansion::Reference(
            id,
            use_span,
            self.type_substitutions.clone(),
        )])
    }

    pub(super) fn expand_expression(
        &mut self,
        expression: Node<resolved::TypeExpression>,
        substitutions: std::sync::Arc<std::collections::HashMap<TypeId, Type>>,
    ) -> Result<Type, Diagnostic> {
        self.expand([Expansion::Expression(expression, substitutions)])
    }

    fn expand(&mut self, initial: impl IntoIterator<Item = Expansion>) -> Result<Type, Diagnostic> {
        let mut pending = initial.into_iter().collect::<Vec<_>>();
        let mut values = Vec::new();
        let span = match pending.last().expect("expansion has a root") {
            Expansion::Expression(expression, _) => expression.span,
            Expansion::Reference(_, span, _) => *span,
            _ => unreachable!("only source expansions may be roots"),
        };
        let mut normalizer = term::Normalizer::new(span);
        while let Some(expansion) = pending.pop() {
            match expansion {
                Expansion::Expression(expression, substitutions) => {
                    match expression.kind {
                        resolved::TypeExpression::Named(reference) => {
                            pending.push(Expansion::Reference(
                                reference.id,
                                reference.name.span,
                                substitutions,
                            ));
                        }
                        resolved::TypeExpression::Application {
                            constructor,
                            arguments,
                        } => {
                            let phantom = self
                                .generic_aliases
                                .get(&constructor.id)
                                .map(|definition| definition.used_parameters.clone())
                                .unwrap_or_default();
                            pending.push(Expansion::ApplyArguments {
                                arguments,
                                phantom,
                                substitutions: substitutions.clone(),
                                index: 0,
                            });
                            pending.push(Expansion::Reference(
                                constructor.id,
                                constructor.name.span,
                                substitutions,
                            ));
                        }
                        resolved::TypeExpression::Unit => values.push(Type::Unit),
                        resolved::TypeExpression::Parenthesized(inner) => {
                            pending.push(Expansion::Expression(*inner, substitutions));
                        }
                        resolved::TypeExpression::Product(elements) => {
                            pending.push(Expansion::Product(elements.len()));
                            pending.extend(elements.into_iter().rev().map(|element| {
                                Expansion::Expression(element, substitutions.clone())
                            }));
                        }
                        resolved::TypeExpression::Sum(members) => {
                            pending.push(Expansion::Sum(members.len()));
                            pending.extend(members.into_iter().rev().map(|member| {
                                Expansion::Expression(member, substitutions.clone())
                            }));
                        }
                        resolved::TypeExpression::Function { parameter, result } => {
                            pending.push(Expansion::Function);
                            pending.push(Expansion::Expression(*result, substitutions.clone()));
                            pending.push(Expansion::Expression(*parameter, substitutions));
                        }
                    }
                }
                Expansion::Reference(id, use_span, substitutions) => {
                    if let Some(ty) = substitutions.get(&id) {
                        values.push(ty.clone());
                    } else if let Some(ty) = predefined_type(id) {
                        values.push(ty);
                    } else if id == BUFFER_TYPE {
                        values.push(
                            normalizer.abstraction(
                                Kind::Type,
                                Type::Buffer(
                                    Type::Bound {
                                        index: 0,
                                        kind: Kind::Type,
                                    }
                                    .into(),
                                ),
                            )?,
                        );
                    } else if let Some(binding) = self.external_types.get(&id) {
                        values.push(Type::External {
                            id,
                            name: binding.name.text.clone(),
                        });
                    } else if let Some(expanded) = self.expanded_aliases.get(&id) {
                        values.push(expanded.clone());
                    } else if let Some(expanded) = self.expanded_generic_aliases.get(&id) {
                        values.push(term::freshen_kind_variables(
                            expanded,
                            &mut self.next_kind_variable,
                            &mut normalizer,
                        )?);
                    } else if let Some(definition) = self.generic_aliases.get(&id).cloned() {
                        if !self.expanding.insert(id) {
                            return Err(Diagnostic::error("recursive generic type alias")
                                .with_primary(use_span, "this application forms an alias cycle"));
                        }
                        let parameter_kinds = self.declaration_parameter_kinds(
                            id,
                            definition.parameters.len(),
                            use_span,
                        )?;
                        let bound = bound_substitutions(&definition.parameters, &parameter_kinds);
                        pending.push(Expansion::AliasAbstraction {
                            id,
                            parameter_kinds,
                        });
                        pending.push(Expansion::Expression(
                            definition.value,
                            std::sync::Arc::new(bound),
                        ));
                    } else if let Some(definition) = self.opaque_types.get(&id).cloned() {
                        if !self.expanding.insert(id) {
                            return Err(Diagnostic::error("recursive opaque representation")
                                .with_primary(use_span, "this representation forms a type cycle"));
                        }
                        let parameter_kinds = self.declaration_parameter_kinds(
                            id,
                            definition.parameters.len(),
                            use_span,
                        )?;
                        let bound = bound_substitutions(&definition.parameters, &parameter_kinds);
                        let arguments = definition
                            .parameters
                            .iter()
                            .map(|parameter| bound[&parameter.id].clone())
                            .collect();
                        pending.push(Expansion::OpaqueAbstraction {
                            id,
                            arguments,
                            parameter_kinds,
                        });
                        pending.push(Expansion::Expression(
                            definition.representation,
                            std::sync::Arc::new(bound),
                        ));
                    } else {
                        if !self.expanding.insert(id) {
                            return Err(Diagnostic::error("recursive type alias")
                                .with_primary(use_span, "this reference forms an alias cycle"));
                        }
                        let value = self
                            .aliases
                            .get(&id)
                            .expect("resolved type IDs must have a definition");
                        pending.push(Expansion::Alias(id));
                        pending.push(Expansion::Expression(value.clone(), substitutions));
                    }
                }
                Expansion::ApplyArguments {
                    arguments,
                    phantom,
                    substitutions,
                    index,
                } => {
                    if index == arguments.len() {
                        continue;
                    }
                    let span = arguments[index].span;
                    let state = Expansion::ApplyArguments {
                        arguments: arguments.clone(),
                        phantom: phantom.clone(),
                        substitutions: substitutions.clone(),
                        index: index + 1,
                    };
                    if phantom.get(index).is_some_and(|used| !used) {
                        let constructor = values
                            .pop()
                            .expect("type application has a constructor term");
                        let Kind::Function { parameter, .. } = constructor.kind() else {
                            return Err(Diagnostic::error("type does not accept arguments")
                                .with_primary(span, "this term already has kind `Type`"));
                        };
                        let ignored = Type::Bound {
                            index: usize::MAX,
                            kind: parameter.as_ref().clone(),
                        };
                        values.push(normalizer.apply(constructor, ignored, span)?);
                        pending.push(state);
                    } else {
                        pending.push(state);
                        pending.push(Expansion::ApplyOne(span));
                        pending.push(Expansion::Expression(
                            arguments[index].clone(),
                            substitutions,
                        ));
                    }
                }
                Expansion::ApplyOne(span) => {
                    let argument = values.pop().expect("type application has one argument");
                    let constructor = values
                        .pop()
                        .expect("type application has a constructor term");
                    values.push(normalizer.apply(constructor, argument, span)?);
                }
                Expansion::AliasAbstraction {
                    id,
                    parameter_kinds,
                } => {
                    let mut body = values.pop().expect("alias expansion produces one term");
                    for kind in parameter_kinds.into_iter().rev() {
                        body = normalizer.abstraction(kind, body)?;
                    }
                    self.expanded_generic_aliases.insert(id, body.clone());
                    values.push(body);
                    assert!(self.expanding.remove(&id));
                }
                Expansion::OpaqueAbstraction {
                    id,
                    arguments,
                    parameter_kinds,
                } => {
                    let representation = values
                        .pop()
                        .expect("opaque representation expansion produces a type");
                    let definition = self
                        .opaque_types
                        .get(&id)
                        .expect("opaque type exists when expansion finishes");
                    let mut body = Type::Opaque {
                        id,
                        name: definition.binding.name.text.clone().into(),
                        arguments: arguments.into(),
                        representation: representation.into(),
                        declaration_file: definition.binding.name.span.file(),
                    };
                    for kind in parameter_kinds.into_iter().rev() {
                        body = normalizer.abstraction(kind, body)?;
                    }
                    values.push(body);
                    assert!(self.expanding.remove(&id));
                }
                Expansion::Alias(id) => {
                    let expanded = values.last().expect("alias expansion must produce a type");
                    self.expanded_aliases.insert(id, expanded.clone());
                    assert!(self.expanding.remove(&id));
                }
                Expansion::Product(length) => {
                    let elements = take_last(&mut values, length);
                    values.push(Type::Product(elements.into()));
                }
                Expansion::Sum(length) => {
                    let members = take_last(&mut values, length);
                    values.push(Type::Sum(members.into()));
                }
                Expansion::Function => {
                    let [parameter, result] = take_last(&mut values, 2).try_into().unwrap();
                    values.push(Type::Function {
                        parameter: parameter.into(),
                        result: result.into(),
                    });
                }
            }
        }
        let [value] = values
            .try_into()
            .expect("one expansion must produce one type");
        Ok(value)
    }

    pub(super) fn declaration_parameter_kinds(
        &mut self,
        id: TypeId,
        count: usize,
        span: Span,
    ) -> Result<Vec<Kind>, Diagnostic> {
        let mut kind = self
            .kinds
            .declaration(id, &mut self.next_kind_variable)
            .expect("source type declaration has an inferred kind");
        let mut parameters = Vec::with_capacity(count);
        for _ in 0..count {
            let Kind::Function { parameter, result } = kind else {
                return Err(Diagnostic::error("type kind mismatch")
                    .with_primary(span, "declaration has fewer constructor parameters"));
            };
            parameters.push(parameter.as_ref().clone());
            kind = result.as_ref().clone();
        }
        Ok(parameters)
    }
}

fn bound_substitutions(
    parameters: &[resolved::TypeBinding],
    kinds: &[Kind],
) -> std::collections::HashMap<TypeId, Type> {
    let count = parameters.len();
    parameters
        .iter()
        .zip(kinds)
        .enumerate()
        .map(|(index, (parameter, kind))| {
            (
                parameter.id,
                Type::Bound {
                    index: count - index - 1,
                    kind: kind.clone(),
                },
            )
        })
        .collect()
}

pub(super) fn ensure_value_type(ty: &Type, span: Span) -> Result<(), Diagnostic> {
    if ty.kind() == Kind::Type {
        return Ok(());
    }
    Err(
        Diagnostic::error("type constructor used as a value type").with_primary(
            span,
            format!(
                "expected `Type` but found `{}`",
                term::kind_name(&ty.kind())
            ),
        ),
    )
}

enum Expansion {
    Expression(
        Node<resolved::TypeExpression>,
        std::sync::Arc<std::collections::HashMap<TypeId, Type>>,
    ),
    Reference(
        TypeId,
        Span,
        std::sync::Arc<std::collections::HashMap<TypeId, Type>>,
    ),
    Alias(TypeId),
    ApplyArguments {
        arguments: Vec<Node<resolved::TypeExpression>>,
        phantom: Vec<bool>,
        substitutions: std::sync::Arc<std::collections::HashMap<TypeId, Type>>,
        index: usize,
    },
    ApplyOne(Span),
    AliasAbstraction {
        id: TypeId,
        parameter_kinds: Vec<Kind>,
    },
    OpaqueAbstraction {
        id: TypeId,
        arguments: Vec<Type>,
        parameter_kinds: Vec<Kind>,
    },
    Product(usize),
    Sum(usize),
    Function,
}

pub(super) fn predefined_type(id: TypeId) -> Option<Type> {
    Some(match id {
        UNIT_TYPE => Type::Unit,
        INT8_TYPE => Type::Int8,
        INT16_TYPE => Type::Int16,
        INT32_TYPE => Type::Int32,
        INT64_TYPE => Type::Int64,
        UINT8_TYPE => Type::UInt8,
        UINT16_TYPE => Type::UInt16,
        UINT32_TYPE => Type::UInt32,
        UINT64_TYPE => Type::UInt64,
        FLOAT32_TYPE => Type::Float32,
        FLOAT64_TYPE => Type::Float64,
        BOOL_TYPE => Type::Sum(vec![Type::Unit, Type::Unit].into()),
        SYMBOL_TYPE => Type::Symbol,
        ADDRESS_TYPE => Type::Address,
        BYTE_SIZE_TYPE => Type::ByteSize,
        U_SIZE_TYPE => Type::USize,
        _ => return None,
    })
}

fn take_last(values: &mut Vec<Type>, length: usize) -> Vec<Type> {
    values.split_off(
        values
            .len()
            .checked_sub(length)
            .expect("composite expansion must have all children"),
    )
}
