//! Canonical type construction, alias expansion, cycle rejection, and bounded diagnostic display.

use crate::resolve::ast::{
    self as resolved, ADDRESS_TYPE, BOOL_TYPE, BUFFER_TYPE, BYTE_SIZE_TYPE, FLOAT32_TYPE,
    FLOAT64_TYPE, INT8_TYPE, INT16_TYPE, INT32_TYPE, INT64_TYPE, SYMBOL_TYPE, TypeId, U_SIZE_TYPE,
    UINT8_TYPE, UINT16_TYPE, UINT32_TYPE, UINT64_TYPE, UNIT_TYPE,
};
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::FileId;
use mal_syntax::source::Span;

use super::{Checker, ast::Type};

mod display;
mod properties;

pub(super) use display::type_name;
pub(super) use properties::{
    ensure_buffer_storable, ensure_representable, is_memory_representable,
    satisfies_representable_requirement, satisfies_storable_requirement, storable_requirements,
};

#[derive(Clone)]
pub(super) struct GenericAliasDefinition {
    pub(super) parameters: Vec<resolved::TypeBinding>,
    used_parameters: Vec<bool>,
    pub(super) value: Node<resolved::TypeExpression>,
}

#[derive(Clone)]
pub(super) struct OpaqueDefinition {
    pub(super) binding: resolved::TypeBinding,
    pub(super) parameters: Vec<resolved::TypeBinding>,
    pub(super) representation: Node<resolved::TypeExpression>,
}

impl Checker {
    pub(super) fn validate_generic_alias(
        &mut self,
        definition: &GenericAliasDefinition,
    ) -> Result<(), Diagnostic> {
        let substitutions = std::sync::Arc::new(
            definition
                .parameters
                .iter()
                .map(|parameter| {
                    (
                        parameter.id,
                        Type::Parameter {
                            id: parameter.id,
                            name: parameter.name.text.clone(),
                        },
                    )
                })
                .collect(),
        );
        let expanded = self.expand([Expansion::Expression(
            definition.value.clone(),
            substitutions,
        )])?;
        ensure_representable(&expanded, definition.value.span)
    }

    pub(super) fn validate_opaque(
        &mut self,
        definition: &OpaqueDefinition,
    ) -> Result<(), Diagnostic> {
        let substitutions = std::sync::Arc::new(
            definition
                .parameters
                .iter()
                .map(|parameter| {
                    (
                        parameter.id,
                        Type::Parameter {
                            id: parameter.id,
                            name: parameter.name.text.clone(),
                        },
                    )
                })
                .collect(),
        );
        let expanded = self.expand([Expansion::Expression(
            definition.representation.clone(),
            substitutions,
        )])?;
        ensure_representable(&expanded, definition.representation.span)
    }

    pub(super) fn collect_aliases(&mut self, program: &resolved::Program) {
        for item in &program.items {
            match &item.kind {
                resolved::TopItem::TypeAlias { binding, value } => {
                    self.aliases.insert(binding.id, value.clone());
                }
                resolved::TopItem::GenericTypeAlias {
                    binding,
                    parameters,
                    value,
                } => {
                    self.generic_aliases.insert(
                        binding.id,
                        GenericAliasDefinition {
                            parameters: parameters.clone(),
                            used_parameters: used_parameters(parameters, value),
                            value: value.clone(),
                        },
                    );
                }
                resolved::TopItem::OpaqueType {
                    binding,
                    parameters,
                    representation,
                } => {
                    self.opaque_types.insert(
                        binding.id,
                        OpaqueDefinition {
                            binding: binding.clone(),
                            parameters: parameters.clone(),
                            representation: representation.clone(),
                        },
                    );
                }
                resolved::TopItem::ExternalType { binding } => {
                    self.external_types.insert(binding.id, binding.clone());
                }
                _ => {}
            }
        }
    }

    pub(super) fn expand_type(
        &mut self,
        ty: &Node<resolved::TypeExpression>,
    ) -> Result<Type, Diagnostic> {
        let expanded = self.expand([Expansion::Expression(
            ty.clone(),
            self.type_substitutions.clone(),
        )])?;
        ensure_representable(&expanded, ty.span)?;
        Ok(expanded)
    }

    pub(super) fn expand_type_id(
        &mut self,
        id: TypeId,
        use_span: Span,
    ) -> Result<Type, Diagnostic> {
        let expanded = self.expand([Expansion::Reference(
            id,
            use_span,
            self.type_substitutions.clone(),
        )])?;
        ensure_representable(&expanded, use_span)?;
        Ok(expanded)
    }

    fn expand(&mut self, initial: impl IntoIterator<Item = Expansion>) -> Result<Type, Diagnostic> {
        let mut pending = initial.into_iter().collect::<Vec<_>>();
        let mut values = Vec::new();
        while let Some(expansion) = pending.pop() {
            match expansion {
                Expansion::Expression(expression, substitutions) => match expression.kind {
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
                        let expected = if constructor.id == BUFFER_TYPE {
                            1
                        } else if let Some(definition) = self.generic_aliases.get(&constructor.id) {
                            definition.parameters.len()
                        } else if let Some(definition) = self.opaque_types.get(&constructor.id) {
                            definition.parameters.len()
                        } else {
                            return Err(Diagnostic::error("type does not accept arguments")
                                .with_primary(
                                    constructor.name.span,
                                    "remove these type arguments",
                                ));
                        };
                        if arguments.len() != expected {
                            return Err(Diagnostic::error("generic type argument arity mismatch")
                                .with_primary(
                                    constructor.name.span,
                                    format!(
                                        "expected {expected} arguments but found {}",
                                        arguments.len()
                                    ),
                                ));
                        }
                        if constructor.id == BUFFER_TYPE {
                            pending.push(Expansion::Buffer(constructor.name.span));
                            pending.extend(arguments.into_iter().rev().map(|argument| {
                                Expansion::Expression(argument, substitutions.clone())
                            }));
                        } else if let Some(definition) = self.generic_aliases.get(&constructor.id) {
                            let used_parameters = definition
                                .parameters
                                .iter()
                                .zip(&definition.used_parameters)
                                .filter_map(|(parameter, used)| used.then_some(parameter.id))
                                .collect::<Vec<_>>();
                            let mut used_arguments = Vec::new();
                            for (argument, used) in
                                arguments.into_iter().zip(&definition.used_parameters)
                            {
                                if *used {
                                    used_arguments.push(argument);
                                } else {
                                    self.validate_phantom_type(&argument)?;
                                }
                            }
                            pending.push(Expansion::GenericAlias {
                                id: constructor.id,
                                span: constructor.name.span,
                                parameters: used_parameters,
                            });
                            pending.extend(used_arguments.into_iter().rev().map(|argument| {
                                Expansion::Expression(argument, substitutions.clone())
                            }));
                        } else {
                            let definition = self
                                .opaque_types
                                .get(&constructor.id)
                                .expect("opaque type was found above");
                            pending.push(Expansion::OpaqueStart {
                                id: constructor.id,
                                span: constructor.name.span,
                                argument_count: definition.parameters.len(),
                            });
                            pending.extend(arguments.into_iter().rev().map(|argument| {
                                Expansion::Expression(argument, substitutions.clone())
                            }));
                        }
                    }
                    resolved::TypeExpression::Unit => values.push(Type::Unit),
                    resolved::TypeExpression::Parenthesized(inner) => {
                        pending.push(Expansion::Expression(*inner, substitutions));
                    }
                    resolved::TypeExpression::Product(elements) => {
                        pending.push(Expansion::Product(elements.len()));
                        pending.extend(
                            elements.into_iter().rev().map(|element| {
                                Expansion::Expression(element, substitutions.clone())
                            }),
                        );
                    }
                    resolved::TypeExpression::Sum(members) => {
                        pending.push(Expansion::Sum(members.len()));
                        pending.extend(
                            members
                                .into_iter()
                                .rev()
                                .map(|member| Expansion::Expression(member, substitutions.clone())),
                        );
                    }
                    resolved::TypeExpression::Function { parameter, result } => {
                        pending.push(Expansion::Function);
                        pending.push(Expansion::Expression(*result, substitutions.clone()));
                        pending.push(Expansion::Expression(*parameter, substitutions));
                    }
                },
                Expansion::Reference(id, use_span, substitutions) => {
                    if let Some(ty) = substitutions.get(&id) {
                        values.push(ty.clone());
                    } else if let Some(ty) = predefined_type(id) {
                        values.push(ty);
                    } else if let Some(binding) = self.external_types.get(&id) {
                        values.push(Type::External {
                            id,
                            name: binding.name.text.clone(),
                        });
                    } else if let Some(expanded) = self.expanded_aliases.get(&id) {
                        values.push(expanded.clone());
                    } else if let Some(definition) = self.opaque_types.get(&id) {
                        if !definition.parameters.is_empty() {
                            return Err(Diagnostic::error("generic type requires arguments")
                                .with_primary(use_span, "supply the declared type arguments"));
                        }
                        pending.push(Expansion::OpaqueStart {
                            id,
                            span: use_span,
                            argument_count: 0,
                        });
                    } else if id == BUFFER_TYPE || self.generic_aliases.contains_key(&id) {
                        return Err(Diagnostic::error("generic type requires arguments")
                            .with_primary(use_span, "supply the declared type arguments"));
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
                Expansion::Buffer(span) => {
                    let element = values.pop().expect("Buffer application has one argument");
                    ensure_buffer_storable(&element, span)?;
                    values.push(Type::Buffer(element.into()));
                }
                Expansion::GenericAlias {
                    id,
                    span,
                    parameters,
                } => {
                    if !self.expanding.insert(id) {
                        return Err(Diagnostic::error("recursive generic type alias")
                            .with_primary(span, "this application forms an alias cycle"));
                    }
                    let arguments = take_last(&mut values, parameters.len());
                    let definition = self
                        .generic_aliases
                        .get(&id)
                        .expect("generic alias was found before its expansion completes");
                    let substitutions =
                        std::sync::Arc::new(parameters.into_iter().zip(arguments).collect());
                    pending.push(Expansion::FinishGenericAlias(id));
                    pending.push(Expansion::Expression(
                        definition.value.clone(),
                        substitutions,
                    ));
                }
                Expansion::FinishGenericAlias(id) => {
                    assert!(self.expanding.remove(&id));
                }
                Expansion::OpaqueStart {
                    id,
                    span,
                    argument_count,
                } => {
                    if !self.expanding.insert(id) {
                        return Err(Diagnostic::error("recursive opaque representation")
                            .with_primary(span, "this representation forms a type cycle"));
                    }
                    let arguments = take_last(&mut values, argument_count);
                    let definition = self
                        .opaque_types
                        .get(&id)
                        .expect("opaque type exists while expanding");
                    let substitutions = std::sync::Arc::new(
                        definition
                            .parameters
                            .iter()
                            .map(|parameter| parameter.id)
                            .zip(arguments.iter().cloned())
                            .collect(),
                    );
                    pending.push(Expansion::OpaqueFinish { id, arguments });
                    pending.push(Expansion::Expression(
                        definition.representation.clone(),
                        substitutions,
                    ));
                }
                Expansion::OpaqueFinish { id, arguments } => {
                    let representation = values
                        .pop()
                        .expect("opaque representation expansion produces a type");
                    let definition = self
                        .opaque_types
                        .get(&id)
                        .expect("opaque type exists when expansion finishes");
                    values.push(Type::Opaque {
                        id,
                        name: definition.binding.name.text.clone().into(),
                        arguments: arguments.into(),
                        representation: representation.into(),
                        declaration_file: definition.binding.name.span.file(),
                    });
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

    fn validate_phantom_type(
        &self,
        root: &Node<resolved::TypeExpression>,
    ) -> Result<(), Diagnostic> {
        let mut pending = vec![root];
        while let Some(expression) = pending.pop() {
            match &expression.kind {
                resolved::TypeExpression::Named(reference) => {
                    if reference.id == BUFFER_TYPE
                        || self.generic_aliases.contains_key(&reference.id)
                    {
                        return Err(Diagnostic::error("generic type requires arguments")
                            .with_primary(
                                reference.name.span,
                                "supply the declared type arguments",
                            ));
                    }
                }
                resolved::TypeExpression::Application {
                    constructor,
                    arguments,
                } => {
                    let expected = if constructor.id == BUFFER_TYPE {
                        1
                    } else if let Some(definition) = self.generic_aliases.get(&constructor.id) {
                        definition.parameters.len()
                    } else if let Some(definition) = self.opaque_types.get(&constructor.id) {
                        definition.parameters.len()
                    } else {
                        return Err(Diagnostic::error("type does not accept arguments")
                            .with_primary(constructor.name.span, "remove these type arguments"));
                    };
                    if arguments.len() != expected {
                        return Err(Diagnostic::error("generic type argument arity mismatch")
                            .with_primary(
                                constructor.name.span,
                                format!(
                                    "expected {expected} arguments but found {}",
                                    arguments.len()
                                ),
                            ));
                    }
                    pending.extend(arguments);
                }
                resolved::TypeExpression::Parenthesized(inner) => pending.push(inner),
                resolved::TypeExpression::Product(elements)
                | resolved::TypeExpression::Sum(elements) => pending.extend(elements),
                resolved::TypeExpression::Function { parameter, result } => {
                    pending.push(parameter);
                    pending.push(result);
                }
                resolved::TypeExpression::Unit => {}
            }
        }
        Ok(())
    }
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
    Buffer(Span),
    Alias(TypeId),
    GenericAlias {
        id: TypeId,
        span: Span,
        parameters: Vec<TypeId>,
    },
    FinishGenericAlias(TypeId),
    OpaqueStart {
        id: TypeId,
        span: Span,
        argument_count: usize,
    },
    OpaqueFinish {
        id: TypeId,
        arguments: Vec<Type>,
    },
    Product(usize),
    Sum(usize),
    Function,
}

fn predefined_type(id: TypeId) -> Option<Type> {
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

fn used_parameters(
    parameters: &[resolved::TypeBinding],
    value: &Node<resolved::TypeExpression>,
) -> Vec<bool> {
    let positions = parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| (parameter.id, index))
        .collect::<std::collections::HashMap<_, _>>();
    let mut used = vec![false; parameters.len()];
    let mut pending = vec![value];
    while let Some(expression) = pending.pop() {
        match &expression.kind {
            resolved::TypeExpression::Named(reference) => {
                if let Some(index) = positions.get(&reference.id) {
                    used[*index] = true;
                }
            }
            resolved::TypeExpression::Application { arguments, .. }
            | resolved::TypeExpression::Product(arguments)
            | resolved::TypeExpression::Sum(arguments) => pending.extend(arguments),
            resolved::TypeExpression::Parenthesized(inner) => pending.push(inner),
            resolved::TypeExpression::Function { parameter, result } => {
                pending.push(parameter);
                pending.push(result);
            }
            resolved::TypeExpression::Unit => {}
        }
    }
    used
}

pub(super) fn substitute_type(
    ty: &Type,
    substitutions: &std::collections::HashMap<TypeId, Type>,
) -> Type {
    match ty {
        Type::Parameter { id, .. } => substitutions.get(id).cloned().unwrap_or_else(|| ty.clone()),
        Type::Buffer(element) => Type::Buffer(substitute_type(element, substitutions).into()),
        Type::Opaque {
            id,
            name,
            arguments,
            representation,
            declaration_file,
        } => Type::Opaque {
            id: *id,
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|argument| substitute_type(argument, substitutions))
                .collect::<Vec<_>>()
                .into(),
            representation: substitute_type(representation, substitutions).into(),
            declaration_file: *declaration_file,
        },
        Type::Product(elements) => Type::Product(
            elements
                .iter()
                .map(|element| substitute_type(element, substitutions))
                .collect::<Vec<_>>()
                .into(),
        ),
        Type::Sum(members) => Type::Sum(
            members
                .iter()
                .map(|member| substitute_type(member, substitutions))
                .collect::<Vec<_>>()
                .into(),
        ),
        Type::Function { parameter, result } => Type::Function {
            parameter: substitute_type(parameter, substitutions).into(),
            result: substitute_type(result, substitutions).into(),
        },
        _ => ty.clone(),
    }
}

pub(super) fn runtime_type(ty: &Type) -> Type {
    match ty {
        Type::Opaque { representation, .. } => runtime_type(representation),
        Type::Buffer(element) => Type::Buffer(runtime_type(element).into()),
        Type::Product(elements) => {
            Type::Product(elements.iter().map(runtime_type).collect::<Vec<_>>().into())
        }
        Type::Sum(members) => {
            Type::Sum(members.iter().map(runtime_type).collect::<Vec<_>>().into())
        }
        Type::Function { parameter, result } => Type::Function {
            parameter: runtime_type(parameter).into(),
            result: runtime_type(result).into(),
        },
        _ => ty.clone(),
    }
}

pub(super) fn bool_type() -> Type {
    Type::Sum(vec![Type::Unit, Type::Unit].into())
}

pub(super) fn representation_view(ty: &Type, file: FileId) -> &Type {
    let mut current = ty;
    while let Type::Opaque {
        representation,
        declaration_file,
        ..
    } = current
    {
        if *declaration_file != file {
            break;
        }
        current = representation;
    }
    current
}

pub(super) fn equivalent_in_file(left: &Type, right: &Type, file: FileId) -> bool {
    let mut pending = vec![(left, right)];
    while let Some((left, right)) = pending.pop() {
        if left == right {
            continue;
        }
        if matches!((left, right), (Type::Opaque { .. }, Type::Opaque { .. })) {
            return false;
        }
        let left_view = representation_view(left, file);
        let right_view = representation_view(right, file);
        if !std::ptr::eq(left, left_view) || !std::ptr::eq(right, right_view) {
            pending.push((left_view, right_view));
            continue;
        }
        match (left, right) {
            (Type::Buffer(left), Type::Buffer(right)) => pending.push((left, right)),
            (Type::Product(left), Type::Product(right)) | (Type::Sum(left), Type::Sum(right))
                if left.len() == right.len() =>
            {
                pending.extend(left.iter().zip(right.iter()));
            }
            (
                Type::Function {
                    parameter: left_parameter,
                    result: left_result,
                },
                Type::Function {
                    parameter: right_parameter,
                    result: right_result,
                },
            ) => {
                pending.push((left_parameter, right_parameter));
                pending.push((left_result, right_result));
            }
            _ => return false,
        }
    }
    true
}

pub(super) fn function_placeholder() -> Type {
    Type::Function {
        parameter: Type::Unit.into(),
        result: Type::Unit.into(),
    }
}
