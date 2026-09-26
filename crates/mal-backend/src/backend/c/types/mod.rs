use crate::backend::c::syntax::{
    AggregateField, TranslationUnit, TypeName, c_aggregate, c_aggregate_field, c_aggregate_fields,
    c_declaration,
};
use mal_frontend::check::ast::{SharedTypeId, Type};
use mal_frontend::resolve::ast::TypeId;

mod collect;
mod host;

#[derive(Default)]
pub(super) struct TypeRegistry {
    aggregates: Vec<Type>,
    collected: std::collections::HashSet<SharedTypeId>,
    indices: std::collections::HashMap<SharedTypeId, usize>,
    structural_indices: std::collections::HashMap<AggregateKey, usize>,
}

#[derive(Default)]
pub(super) struct HostTypes {
    types: Vec<Type>,
    collected: std::collections::HashSet<SharedTypeId>,
    external_types: Vec<Type>,
    external_collected: std::collections::HashSet<SharedTypeId>,
    external_aliases: std::collections::HashSet<String>,
    memory_types: Vec<Type>,
    memory_collected: std::collections::HashSet<SharedTypeId>,
    opaque_names: Vec<String>,
}

#[derive(Eq, Hash, PartialEq)]
enum AggregateKey {
    Product(Vec<ElementKey>),
    Sum(Vec<ElementKey>),
}

#[derive(Eq, Hash, PartialEq)]
enum ElementKey {
    Unit,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Address,
    ByteSize,
    USize,
    External(TypeId),
    Aggregate(usize),
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
            Type::Address => TypeName::named("MalType_Address"),
            Type::ByteSize => TypeName::named("MalType_ByteSize"),
            Type::USize => TypeName::named("MalType_USize"),
            Type::External { name, .. } => TypeName::named(format!("MalType_{name}")),
            Type::Product(_) => TypeName::named(format!("MalRepr_Product_{}", self.index(ty))),
            Type::Sum(_) => TypeName::named(format!("MalRepr_Sum_{}", self.index(ty))),
            Type::Function { .. } => TypeName::named(format!("MalRepr_Closure_{}", self.index(ty))),
            Type::Symbol | Type::Parameter { .. } | Type::Buffer(_) => {
                unreachable!("these types never enter the C host registry")
            }
        }
    }

    pub(super) fn host_value_c_type(&self, ty: &Type, alias: Option<&str>) -> TypeName {
        if let Some(alias) = alias {
            return TypeName::named(format!("mal_{alias}_t"));
        }
        if is_bool(ty) {
            return TypeName::named("mal_Bool_t");
        }
        match ty {
            Type::Unit => TypeName::named("mal_Unit_t"),
            Type::Int8 => TypeName::named("mal_Int8_t"),
            Type::Int16 => TypeName::named("mal_Int16_t"),
            Type::Int32 => TypeName::named("mal_Int32_t"),
            Type::Int64 => TypeName::named("mal_Int64_t"),
            Type::UInt8 => TypeName::named("mal_UInt8_t"),
            Type::UInt16 => TypeName::named("mal_UInt16_t"),
            Type::UInt32 => TypeName::named("mal_UInt32_t"),
            Type::UInt64 => TypeName::named("mal_UInt64_t"),
            Type::Float32 => TypeName::named("mal_Float32_t"),
            Type::Float64 => TypeName::named("mal_Float64_t"),
            Type::Address => TypeName::named("mal_Address_t"),
            Type::ByteSize => TypeName::named("mal_ByteSize_t"),
            Type::USize => TypeName::named("mal_USize_t"),
            Type::External { name, .. } => TypeName::named(format!("mal_{name}_t")),
            Type::Product(_) => TypeName::named(format!("mal_repr_product_{}_t", self.index(ty))),
            Type::Sum(_) => TypeName::named(format!("mal_repr_sum_{}_t", self.index(ty))),
            Type::Function { .. } => {
                unreachable!("type checking excludes functions from extern signatures")
            }
            Type::Symbol | Type::Parameter { .. } | Type::Buffer(_) => {
                unreachable!("these types are not host mappable")
            }
        }
    }

    fn declarations(&self, host: &HostTypes, public: bool) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for (index, ty) in self.aggregates.iter().enumerate() {
            if host.external_contains(ty) != public {
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
                | Type::Address
                | Type::ByteSize
                | Type::USize => unreachable!(),
                Type::Symbol | Type::Parameter { .. } | Type::Buffer(_) => {
                    unreachable!("these types never enter the C host registry")
                }
            };
            let name = format!("{kind}_{index}");
            output.push(c_declaration!(type name = struct(format!("{kind}_{index}"))));
        }
        if !output.is_empty() {
            output.blank_line();
        }
        for (index, ty) in self.aggregates.iter().enumerate() {
            if host.external_contains(ty) != public {
                continue;
            }
            match ty {
                Type::Product(elements) => {
                    let fields = elements.iter().enumerate().map(|(element_index, element)| {
                        let name = format!("field_{element_index}");
                        c_aggregate_field!(name : { self.c_type(element) })
                    });
                    let tag = format!("MalRepr_Product_{index}");
                    output.push(c_aggregate!(struct tag => [{{ fields }}]));
                    output.blank_line();
                }
                Type::Sum(members) => {
                    let tag = format!("MalRepr_Sum_{index}");
                    let fields = sum_representation_fields(members, |member| self.c_type(member));
                    output.push(c_aggregate!(struct tag => [{{ fields }}]));
                    output.blank_line();
                }
                Type::Function { parameter, result } => {
                    let tag = format!("MalRepr_Closure_{index}");
                    let fields = c_aggregate_fields!(
                        (fn "call"(
                            _: ptr(named("MalContext")),
                            _: ptr(const(named("void"))),
                            _: { self.c_type(parameter) },
                        ) -> { self.c_type(result) }),
                        ("environment": ptr(const(named("void")))),
                        (fn "destroy_environment"(
                            _: ptr(named("MalContext")),
                            _: ptr(const(named("void"))),
                        ) -> named("void")),
                    );
                    output.push(c_aggregate!(struct tag => [{{ fields }}]));
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
                | Type::Address
                | Type::ByteSize
                | Type::USize => unreachable!(),
                Type::Symbol | Type::Parameter { .. } | Type::Buffer(_) => {
                    unreachable!("these types never enter the C host registry")
                }
            }
        }
        output
    }
}

fn sum_representation_fields(
    members: &[Type],
    c_type: impl Fn(&Type) -> TypeName,
) -> Vec<AggregateField> {
    let mut fields = c_aggregate_fields!(("tag": named("uint32_t")));
    if !members.is_empty() {
        let members = members.iter().enumerate().map(|(index, member)| {
            let name = format!("variant_{index}");
            c_aggregate_field!(name : { c_type(member) })
        });
        fields.push(c_aggregate_field!(union "payload"; [{{ members }}]));
    }
    fields
}

pub(super) fn is_bool(ty: &Type) -> bool {
    matches!(ty, Type::Sum(members) if members.as_ref() == [Type::Unit, Type::Unit])
}

#[cfg(test)]
mod tests {
    use mal_frontend::resolve::ast::TypeId;

    use super::*;

    #[test]
    fn maps_host_values_without_reusing_raw_type_names() {
        let product = Type::Product(vec![Type::UInt64, Type::Address].into());
        let sum = Type::Sum(vec![Type::Unit, product.clone()].into());
        let mut registry = TypeRegistry::default();
        registry.collect(&sum);

        for (ty, expected) in [
            (Type::Unit, "mal_Unit_t"),
            (Type::Int32, "mal_Int32_t"),
            (Type::UInt64, "mal_UInt64_t"),
            (Type::Float64, "mal_Float64_t"),
            (Type::Address, "mal_Address_t"),
            (
                Type::External {
                    id: TypeId(3),
                    name: "File".into(),
                },
                "mal_File_t",
            ),
        ] {
            assert_eq!(
                registry.host_value_c_type(&ty, None),
                TypeName::named(expected)
            );
        }
        assert_eq!(
            registry.host_value_c_type(&product, None),
            TypeName::named("mal_repr_product_0_t")
        );
        assert_eq!(
            registry.host_value_c_type(&sum, None),
            TypeName::named("mal_repr_sum_1_t")
        );
        assert_eq!(
            registry.host_value_c_type(&Type::UInt64, Some("Count")),
            TypeName::named("mal_Count_t")
        );
        assert_eq!(
            registry.host_value_c_type(&product, Some("Packet")),
            TypeName::named("mal_Packet_t")
        );
    }

    #[test]
    fn maps_bool_before_its_structural_sum_representation() {
        let bool_type = Type::Sum(vec![Type::Unit, Type::Unit].into());
        assert_eq!(
            TypeRegistry::default().host_value_c_type(&bool_type, None),
            TypeName::named("mal_Bool_t")
        );
    }

    #[test]
    fn declares_host_aggregates_in_structural_dependency_order() {
        let product = Type::Product(vec![Type::UInt64, Type::Address].into());
        let sum = Type::Sum(vec![Type::Unit, product.clone()].into());
        let mut registry = TypeRegistry::default();
        registry.collect(&sum);
        let host = HostTypes {
            types: vec![product.clone(), sum.clone()],
            external_aliases: ["Packet".into(), "Result".into()].into_iter().collect(),
            ..HostTypes::default()
        };
        let aliases = [
            crate::core::ast::TypeAlias {
                name: "Packet".into(),
                ty: product,
                element_aliases: vec![None, None],
                host_memory_access: false,
                span: mal_syntax::source::Span::new(mal_syntax::source::FileId::new(0), 0, 0),
            },
            crate::core::ast::TypeAlias {
                name: "Result".into(),
                ty: sum,
                element_aliases: vec![None, Some("Packet".into())],
                host_memory_access: false,
                span: mal_syntax::source::Span::new(mal_syntax::source::FileId::new(0), 0, 0),
            },
        ];

        let declarations = registry.host_value_declarations(&host, &aliases).render();

        assert!(
            declarations.contains("typedef struct mal_detail_repr_product_0 mal_repr_product_0_t;")
        );
        assert!(declarations.contains("typedef struct mal_detail_repr_sum_1 mal_repr_sum_1_t;"));
        assert!(declarations.contains("typedef mal_repr_product_0_t mal_Packet_t;"));
        assert!(declarations.contains("typedef mal_repr_sum_1_t mal_Result_t;"));
        assert!(declarations.contains("mal_Address_t field_1;"));
        assert!(declarations.contains("mal_repr_product_0_t variant_1;"));
    }
}
