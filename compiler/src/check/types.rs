use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::{
    self as resolved, ADDRESS_TYPE, BOOL_TYPE, BYTE_SIZE_TYPE, CURSOR_TYPE, FLOAT32_TYPE,
    FLOAT64_TYPE, INT8_TYPE, INT16_TYPE, INT32_TYPE, INT64_TYPE, PACKED_TYPE, REGION_TYPE,
    SYMBOL_TYPE, TypeId, U_SIZE_TYPE, UINT8_TYPE, UINT16_TYPE, UINT32_TYPE, UINT64_TYPE, UNIT_TYPE,
};
use crate::source::Span;

use super::{Checker, ast::Type};

const MAX_REPRESENTATION_UNITS: usize = 65_536;
const MAX_REPRESENTATION_DEPTH: usize = 64;

#[derive(Clone)]
pub(super) struct AliasDefinition {
    pub(super) binding: resolved::TypeBinding,
    pub(super) value: Node<resolved::TypeExpression>,
}

#[derive(Clone)]
pub(super) struct GenericAliasDefinition {
    pub(super) parameters: Vec<resolved::TypeBinding>,
    pub(super) value: Node<resolved::TypeExpression>,
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

    pub(super) fn collect_aliases(&mut self, program: &resolved::Program) {
        for item in &program.items {
            match &item.kind {
                resolved::TopItem::TypeAlias { binding, value } => {
                    self.aliases.insert(
                        binding.id,
                        AliasDefinition {
                            binding: binding.clone(),
                            value: value.clone(),
                        },
                    );
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
                            value: value.clone(),
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
                            pending.push(Expansion::Application(constructor, arguments.len()));
                            pending.extend(arguments.into_iter().rev().map(|argument| {
                                Expansion::Expression(argument, substitutions.clone())
                            }));
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
                    } else if let Some(binding) = self.external_types.get(&id) {
                        values.push(Type::External {
                            id,
                            name: binding.name.text.clone(),
                        });
                    } else if let Some(expanded) = self.expanded_aliases.get(&id) {
                        values.push(expanded.clone());
                    } else if matches!(id, CURSOR_TYPE | REGION_TYPE | PACKED_TYPE)
                        || self.generic_aliases.contains_key(&id)
                    {
                        return Err(Diagnostic::error("generic type requires arguments")
                            .with_primary(use_span, "supply the declared type arguments"));
                    } else {
                        if !self.expanding.insert(id) {
                            return Err(Diagnostic::error("recursive type alias")
                                .with_primary(use_span, "this reference forms an alias cycle"));
                        }
                        let definition = self
                            .aliases
                            .get(&id)
                            .expect("resolved type IDs must have a definition");
                        pending.push(Expansion::Alias(id));
                        pending.push(Expansion::Expression(
                            definition.value.clone(),
                            substitutions,
                        ));
                    }
                }
                Expansion::Application(constructor, arity) => {
                    let arguments = take_last(&mut values, arity);
                    let expected =
                        if matches!(constructor.id, CURSOR_TYPE | REGION_TYPE | PACKED_TYPE) {
                            1
                        } else if let Some(definition) = self.generic_aliases.get(&constructor.id) {
                            definition.parameters.len()
                        } else {
                            return Err(Diagnostic::error("type does not accept arguments")
                                .with_primary(
                                    constructor.name.span,
                                    "remove these type arguments",
                                ));
                        };
                    if arity != expected {
                        return Err(Diagnostic::error("generic type argument arity mismatch")
                            .with_primary(
                                constructor.name.span,
                                format!("expected {expected} arguments but found {arity}"),
                            ));
                    }
                    if constructor.id == CURSOR_TYPE {
                        let element = arguments.into_iter().next().unwrap();
                        ensure_memory_representable(&element, constructor.name.span)?;
                        values.push(Type::Cursor(element.into()));
                    } else if constructor.id == REGION_TYPE {
                        let element = arguments.into_iter().next().unwrap();
                        ensure_memory_representable(&element, constructor.name.span)?;
                        values.push(Type::Region(element.into()));
                    } else if constructor.id == PACKED_TYPE {
                        let element = arguments.into_iter().next().unwrap();
                        ensure_memory_representable(&element, constructor.name.span)?;
                        values.push(Type::Packed(element.into()));
                    } else {
                        if !self.expanding.insert(constructor.id) {
                            return Err(Diagnostic::error("recursive generic type alias")
                                .with_primary(
                                    constructor.name.span,
                                    "this application forms an alias cycle",
                                ));
                        }
                        let definition = self
                            .generic_aliases
                            .get(&constructor.id)
                            .expect("generic alias was found above");
                        let substitutions = std::sync::Arc::new(
                            definition
                                .parameters
                                .iter()
                                .map(|parameter| parameter.id)
                                .zip(arguments)
                                .collect(),
                        );
                        pending.push(Expansion::GenericAlias(constructor.id));
                        pending.push(Expansion::Expression(
                            definition.value.clone(),
                            substitutions,
                        ));
                    }
                }
                Expansion::GenericAlias(id) => {
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
    Application(resolved::TypeReference, usize),
    Alias(TypeId),
    GenericAlias(TypeId),
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

pub(super) fn ensure_memory_representable(ty: &Type, span: Span) -> Result<(), Diagnostic> {
    let mut pending = vec![ty];
    while let Some(ty) = pending.pop() {
        match ty {
            Type::Unit
            | Type::Int8
            | Type::Int16
            | Type::Int32
            | Type::Int64
            | Type::UInt8
            | Type::UInt16
            | Type::UInt32
            | Type::UInt64
            | Type::Float32
            | Type::Float64
            | Type::Address
            | Type::ByteSize
            | Type::USize
            | Type::Parameter { .. } => {}
            Type::Product(elements) => pending.extend(elements.iter()),
            Type::Sum(members) if !members.is_empty() => pending.extend(members.iter()),
            Type::Symbol
            | Type::External { .. }
            | Type::Function { .. }
            | Type::Cursor(_)
            | Type::Region(_)
            | Type::Packed(_)
            | Type::Sum(_) => {
                return Err(
                    Diagnostic::error("memory element type is not representable").with_primary(
                        span,
                        format!("`{}` has no canonical memory representation", type_name(ty)),
                    ),
                );
            }
        }
    }
    Ok(())
}

pub(super) fn representable_requirements(ty: &Type) -> std::collections::HashSet<TypeId> {
    let mut requirements = std::collections::HashSet::new();
    let mut pending = vec![(ty, false)];
    while let Some((ty, required)) = pending.pop() {
        match ty {
            Type::Parameter { id, .. } if required => {
                requirements.insert(*id);
            }
            Type::Cursor(element) | Type::Region(element) | Type::Packed(element) => {
                pending.push((element, true));
            }
            Type::Product(elements) | Type::Sum(elements) => {
                pending.extend(elements.iter().map(|element| (element, required)));
            }
            Type::Function { parameter, result } => {
                pending.push((parameter, required));
                pending.push((result, required));
            }
            _ => {}
        }
    }
    requirements
}

pub(super) fn satisfies_representable_requirement(
    ty: &Type,
    available: &std::collections::HashSet<TypeId>,
) -> bool {
    let mut pending = vec![ty];
    while let Some(ty) = pending.pop() {
        match ty {
            Type::Parameter { id, .. } => {
                if !available.contains(id) {
                    return false;
                }
            }
            Type::Product(elements) => pending.extend(elements.iter()),
            Type::Sum(members) if !members.is_empty() => pending.extend(members.iter()),
            Type::Unit
            | Type::Int8
            | Type::Int16
            | Type::Int32
            | Type::Int64
            | Type::UInt8
            | Type::UInt16
            | Type::UInt32
            | Type::UInt64
            | Type::Float32
            | Type::Float64
            | Type::Address
            | Type::ByteSize
            | Type::USize => {}
            _ => return false,
        }
    }
    true
}

pub(super) fn substitute_type(
    ty: &Type,
    substitutions: &std::collections::HashMap<TypeId, Type>,
) -> Type {
    match ty {
        Type::Parameter { id, .. } => substitutions.get(id).cloned().unwrap_or_else(|| ty.clone()),
        Type::Cursor(element) => Type::Cursor(substitute_type(element, substitutions).into()),
        Type::Region(element) => Type::Region(substitute_type(element, substitutions).into()),
        Type::Packed(element) => Type::Packed(substitute_type(element, substitutions).into()),
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

pub(super) fn ensure_representable(ty: &Type, span: Span) -> Result<(), Diagnostic> {
    if representation_units(ty).is_none() {
        return Err(
            Diagnostic::error("type representation is too large").with_primary(
                span,
                format!(
                    "the reference compiler supports at most {MAX_REPRESENTATION_DEPTH} nested levels and {MAX_REPRESENTATION_UNITS} storage components"
                ),
            ),
        );
    }
    Ok(())
}

fn representation_units(ty: &Type) -> Option<usize> {
    let mut pending = vec![Representation::Type(ty)];
    let mut values = Vec::new();
    let mut cache = std::collections::HashMap::new();
    while let Some(item) = pending.pop() {
        match item {
            Representation::Type(ty) => {
                if let Some(measure) = ty.shared_id().and_then(|id| cache.get(&id).copied()) {
                    values.push(measure);
                    continue;
                }
                match ty {
                    Type::Product(elements) => {
                        pending.push(Representation::Product(ty.shared_id(), elements.len()));
                        pending.extend(elements.iter().rev().map(Representation::Type));
                    }
                    Type::Sum(elements) => {
                        pending.push(Representation::Sum(ty.shared_id(), elements.len()));
                        pending.extend(elements.iter().rev().map(Representation::Type));
                    }
                    Type::Function { parameter, result } => {
                        pending.push(Representation::Function(ty.shared_id()));
                        pending.push(Representation::Type(result));
                        pending.push(Representation::Type(parameter));
                    }
                    _ => values.push(RepresentationMeasure { units: 1, depth: 0 }),
                }
            }
            Representation::Product(id, length) => {
                let children = take_measures(&mut values, length);
                let units = children
                    .iter()
                    .try_fold(0usize, |total, measure| total.checked_add(measure.units))?;
                let measure = composite_measure(units, &children)?;
                store_measure(id, measure, &mut cache);
                values.push(measure);
            }
            Representation::Sum(id, length) => {
                let children = take_measures(&mut values, length);
                let units = children
                    .iter()
                    .map(|measure| measure.units)
                    .max()
                    .unwrap_or(0)
                    .checked_add(1)?;
                let measure = composite_measure(units, &children)?;
                store_measure(id, measure, &mut cache);
                values.push(measure);
            }
            Representation::Function(id) => {
                let children = take_measures(&mut values, 2);
                let units = children
                    .iter()
                    .map(|measure| measure.units)
                    .max()
                    .unwrap_or(2)
                    .max(2);
                let depth = children
                    .iter()
                    .map(|measure| measure.depth)
                    .max()
                    .unwrap_or(0);
                let measure = (units <= MAX_REPRESENTATION_UNITS
                    && depth <= MAX_REPRESENTATION_DEPTH)
                    .then_some(RepresentationMeasure { units, depth })?;
                store_measure(id, measure, &mut cache);
                values.push(measure);
            }
        }
    }
    let [measure] = values.try_into().ok()?;
    Some(measure.units)
}

enum Representation<'a> {
    Type(&'a Type),
    Product(Option<super::ast::SharedTypeId>, usize),
    Sum(Option<super::ast::SharedTypeId>, usize),
    Function(Option<super::ast::SharedTypeId>),
}

#[derive(Clone, Copy)]
struct RepresentationMeasure {
    units: usize,
    depth: usize,
}

fn take_measures(
    values: &mut Vec<RepresentationMeasure>,
    length: usize,
) -> Vec<RepresentationMeasure> {
    values.split_off(
        values
            .len()
            .checked_sub(length)
            .expect("all child units exist"),
    )
}

fn composite_measure(
    units: usize,
    children: &[RepresentationMeasure],
) -> Option<RepresentationMeasure> {
    let depth = children
        .iter()
        .map(|measure| measure.depth)
        .max()
        .unwrap_or(0)
        .checked_add(1)?;
    (units <= MAX_REPRESENTATION_UNITS && depth <= MAX_REPRESENTATION_DEPTH)
        .then_some(RepresentationMeasure { units, depth })
}

fn store_measure(
    id: Option<super::ast::SharedTypeId>,
    measure: RepresentationMeasure,
    cache: &mut std::collections::HashMap<super::ast::SharedTypeId, RepresentationMeasure>,
) {
    if let Some(id) = id {
        cache.insert(id, measure);
    }
}

pub(super) fn bool_type() -> Type {
    Type::Sum(vec![Type::Unit, Type::Unit].into())
}

pub(super) fn function_placeholder() -> Type {
    Type::Function {
        parameter: Type::Unit.into(),
        result: Type::Unit.into(),
    }
}

pub(super) fn type_name(ty: &Type) -> String {
    const LIMIT: usize = 4096;

    let mut output = String::new();
    let mut pending = vec![TypeNamePart::Type(ty)];
    while let Some(part) = pending.pop() {
        let text = match part {
            TypeNamePart::Text(text) => text,
            TypeNamePart::Type(ty) => match ty {
                Type::Unit => "Unit",
                Type::Int8 => "Int8",
                Type::Int16 => "Int16",
                Type::Int32 => "Int32",
                Type::Int64 => "Int64",
                Type::UInt8 => "UInt8",
                Type::UInt16 => "UInt16",
                Type::UInt32 => "UInt32",
                Type::UInt64 => "UInt64",
                Type::Float32 => "Float32",
                Type::Float64 => "Float64",
                Type::Symbol => "Symbol",
                Type::Address => "Address",
                Type::ByteSize => "ByteSize",
                Type::USize => "USize",
                Type::Parameter { name, .. } => name,
                Type::Cursor(element) => {
                    pending.push(TypeNamePart::Text(">"));
                    pending.push(TypeNamePart::Type(element));
                    "Cursor<"
                }
                Type::Region(element) => {
                    pending.push(TypeNamePart::Text(">"));
                    pending.push(TypeNamePart::Type(element));
                    "Region<"
                }
                Type::Packed(element) => {
                    pending.push(TypeNamePart::Text(">"));
                    pending.push(TypeNamePart::Type(element));
                    "Packed<"
                }
                Type::External { name, .. } => name,
                Type::Product(elements) => {
                    push_aggregate_name(&mut pending, elements, ")");
                    "("
                }
                Type::Sum(members) if members.as_ref() == [Type::Unit, Type::Unit] => "Bool",
                Type::Sum(members) => {
                    push_aggregate_name(&mut pending, members, "]");
                    "["
                }
                Type::Function { parameter, result } => {
                    pending.push(TypeNamePart::Type(result));
                    pending.push(TypeNamePart::Text(" -> "));
                    pending.push(TypeNamePart::Type(parameter));
                    continue;
                }
            },
        };
        if output.len().saturating_add(text.len()) > LIMIT {
            output.push('…');
            break;
        }
        output.push_str(text);
    }
    output
}

enum TypeNamePart<'a> {
    Type(&'a Type),
    Text(&'a str),
}

fn push_aggregate_name<'a>(
    pending: &mut Vec<TypeNamePart<'a>>,
    elements: &'a [Type],
    close: &'static str,
) {
    pending.push(TypeNamePart::Text(close));
    for (index, element) in elements.iter().enumerate().rev() {
        pending.push(TypeNamePart::Type(element));
        if index != 0 {
            pending.push(TypeNamePart::Text(", "));
        }
    }
}
