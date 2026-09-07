use crate::c_emit::syntax::{
    AggregateDefinition, AggregateField, AggregateKind, Declaration, Parameter, TranslationUnit,
    TypeName,
};
use crate::check::ast::Type;

mod collect;
mod host;
mod lifetime;

#[derive(Default)]
pub(super) struct TypeRegistry {
    aggregates: Vec<Type>,
    uses_float32: bool,
    uses_float64: bool,
}

#[derive(Default)]
pub(super) struct HostTypes {
    types: Vec<Type>,
    opaque_names: Vec<String>,
}

impl TypeRegistry {
    pub(super) fn c_type(&self, ty: &Type) -> TypeName {
        if is_bool(ty) {
            return TypeName::named("MalType_Bool");
        }
        match ty {
            Type::Unit => TypeName::named("MalType_Unit"),
            Type::Int8 => TypeName::named("MalType_Int8"),
            Type::Int16 => TypeName::named("MalType_Int16"),
            Type::Int32 => TypeName::named("MalType_Int32"),
            Type::Int64 => TypeName::named("MalType_Int64"),
            Type::UInt8 => TypeName::named("MalType_UInt8"),
            Type::UInt16 => TypeName::named("MalType_UInt16"),
            Type::UInt32 => TypeName::named("MalType_UInt32"),
            Type::UInt64 => TypeName::named("MalType_UInt64"),
            Type::Float32 => TypeName::named("MalType_Float32"),
            Type::Float64 => TypeName::named("MalType_Float64"),
            Type::Symbol => TypeName::named("MalType_Symbol"),
            Type::Ptr => TypeName::named("MalType_Ptr"),
            Type::External { name, .. } => TypeName::named(format!("MalType_{name}")),
            Type::Product(_) => TypeName::named(format!("MalRepr_Product_{}", self.index(ty))),
            Type::Sum(_) => TypeName::named(format!("MalRepr_Sum_{}", self.index(ty))),
            Type::Function { .. } => TypeName::named(format!("MalRepr_Closure_{}", self.index(ty))),
        }
    }

    pub(super) fn uses_float(&self) -> bool {
        self.uses_float32 || self.uses_float64
    }

    pub(super) fn uses_float32(&self) -> bool {
        self.uses_float32
    }

    pub(super) fn uses_float64(&self) -> bool {
        self.uses_float64
    }

    pub(super) fn source_declarations(&self, host: &HostTypes) -> TranslationUnit {
        self.declarations(host, false)
    }

    fn declarations(&self, host: &HostTypes, public: bool) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for (index, ty) in self.aggregates.iter().enumerate() {
            if host.contains(ty) != public {
                continue;
            }
            let kind = match ty {
                Type::Product(_) => "MalRepr_Product",
                Type::Sum(_) => "MalRepr_Sum",
                Type::Function { .. } => "MalRepr_Closure",
                Type::External { .. }
                | Type::Unit
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
                | Type::Symbol
                | Type::Ptr => unreachable!(),
            };
            output.push(Declaration::type_alias(
                TypeName::structure(format!("{kind}_{index}")),
                format!("{kind}_{index}"),
            ));
        }
        if !output.is_empty() {
            output.blank_line();
        }
        for (index, ty) in self.aggregates.iter().enumerate() {
            if host.contains(ty) != public {
                continue;
            }
            match ty {
                Type::Product(elements) => {
                    let fields = elements.iter().enumerate().map(|(element_index, element)| {
                        AggregateField::variable(
                            self.c_type(element),
                            format!("field_{element_index}"),
                        )
                    });
                    output.push(AggregateDefinition::structure(
                        format!("MalRepr_Product_{index}"),
                        fields,
                    ));
                    output.blank_line();
                }
                Type::Sum(members) => {
                    let variants = members.iter().enumerate().map(|(member_index, member)| {
                        AggregateField::variable(
                            self.c_type(member),
                            format!("variant_{member_index}"),
                        )
                    });
                    output.push(AggregateDefinition::structure(
                        format!("MalRepr_Sum_{index}"),
                        [
                            AggregateField::variable("uint32_t", "tag"),
                            AggregateField::aggregate(AggregateKind::Union, variants, "payload"),
                        ],
                    ));
                    output.blank_line();
                }
                Type::Function { parameter, result } => {
                    output.push(AggregateDefinition::structure(
                        format!("MalRepr_Closure_{index}"),
                        [
                            AggregateField::function_pointer(
                                self.c_type(result),
                                "call",
                                [
                                    Parameter::unnamed(TypeName::named("MalContext").pointer()),
                                    Parameter::unnamed(TypeName::const_named("void").pointer()),
                                    Parameter::unnamed(self.c_type(parameter)),
                                ],
                            ),
                            AggregateField::variable(
                                TypeName::const_named("void").pointer(),
                                "environment",
                            ),
                            AggregateField::function_pointer(
                                "void",
                                "destroy_environment",
                                [
                                    Parameter::unnamed(TypeName::named("MalContext").pointer()),
                                    Parameter::unnamed(TypeName::const_named("void").pointer()),
                                ],
                            ),
                        ],
                    ));
                    output.blank_line();
                }
                Type::External { .. }
                | Type::Unit
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
                | Type::Symbol
                | Type::Ptr => unreachable!(),
            }
        }
        output
    }
}

pub(super) fn is_bool(ty: &Type) -> bool {
    matches!(ty, Type::Sum(members) if members == &[Type::Unit, Type::Unit])
}
