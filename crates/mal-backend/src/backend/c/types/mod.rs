//! Structural registry and target-aware C mapping of host-visible types.

use crate::backend::c::syntax::{
    Directive, MacroInvocation, TranslationUnit, TypeName, c_aggregate, c_aggregate_fields,
    c_declaration, c_directive, c_expr, c_macro_invocation, c_type,
};
use mal_frontend::check::ast::{SharedTypeId, Type};

mod collect;
mod host;

#[derive(Default)]
pub(super) struct TypeRegistry {
    aggregates: Vec<Type>,
    collected: std::collections::HashSet<SharedTypeId>,
    indices: std::collections::HashMap<SharedTypeId, RepresentationId>,
    structural_indices: std::collections::HashMap<AggregateKey, RepresentationId>,
    fingerprint_keys: std::collections::HashMap<RepresentationId, AggregateKey>,
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum AggregateKey {
    Product(Vec<ElementKey>),
    Sum(Vec<ElementKey>),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
    External(String),
    Aggregate(RepresentationId),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct RepresentationId(u64);

impl std::fmt::Display for RepresentationId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:016x}", self.0)
    }
}

impl TypeRegistry {
    pub(super) fn c_type(&self, ty: &Type) -> TypeName {
        if is_bool(ty) {
            return c_type!(named("MalType_Bool"));
        }
        match ty {
            Type::Unit => c_type!(named("MalType_Unit")),
            Type::Int8 => c_type!(named("MalType_Int8")),
            Type::Int16 => c_type!(named("MalType_Int16")),
            Type::Int32 => c_type!(named("MalType_Int32")),
            Type::Int64 => c_type!(named("MalType_Int64")),
            Type::UInt8 => c_type!(named("MalType_UInt8")),
            Type::UInt16 => c_type!(named("MalType_UInt16")),
            Type::UInt32 => c_type!(named("MalType_UInt32")),
            Type::UInt64 => c_type!(named("MalType_UInt64")),
            Type::Float32 => c_type!(named("MalType_Float32")),
            Type::Float64 => c_type!(named("MalType_Float64")),
            Type::Address => c_type!(named("MalType_Address")),
            Type::ByteSize => c_type!(named("MalType_ByteSize")),
            Type::USize => c_type!(named("MalType_USize")),
            Type::External { name, .. } => c_type!(named(#{ format!("MalType_{name}") })),
            Type::Product(_) => {
                c_type!(named(#{ format!("MalRepr_Product_{}", self.index(ty)) }))
            }
            Type::Sum(_) => c_type!(named(#{ format!("MalRepr_Sum_{}", self.index(ty)) })),
            Type::Function { .. } => {
                c_type!(named(#{ format!("MalRepr_Closure_{}", self.index(ty)) }))
            }
            Type::Symbol | Type::Parameter { .. } | Type::Buffer(_) | Type::Opaque { .. } => {
                unreachable!("these types never enter the C host registry")
            }
        }
    }

    pub(super) fn host_value_c_type(&self, ty: &Type, alias: Option<&str>) -> TypeName {
        if let Some(alias) = alias {
            return c_type!(named(#{ format!("mal_{alias}_t") }));
        }
        if is_bool(ty) {
            return c_type!(named("mal_Bool_t"));
        }
        match ty {
            Type::Unit => c_type!(named("mal_Unit_t")),
            Type::Int8 => c_type!(named("mal_Int8_t")),
            Type::Int16 => c_type!(named("mal_Int16_t")),
            Type::Int32 => c_type!(named("mal_Int32_t")),
            Type::Int64 => c_type!(named("mal_Int64_t")),
            Type::UInt8 => c_type!(named("mal_UInt8_t")),
            Type::UInt16 => c_type!(named("mal_UInt16_t")),
            Type::UInt32 => c_type!(named("mal_UInt32_t")),
            Type::UInt64 => c_type!(named("mal_UInt64_t")),
            Type::Float32 => c_type!(named("mal_Float32_t")),
            Type::Float64 => c_type!(named("mal_Float64_t")),
            Type::Address => c_type!(named("mal_Address_t")),
            Type::ByteSize => c_type!(named("mal_ByteSize_t")),
            Type::USize => c_type!(named("mal_USize_t")),
            Type::External { name, .. } => c_type!(named(#{ format!("mal_{name}_t") })),
            Type::Product(_) => {
                c_type! {
                    named(#{ format!("mal_repr_product_{}_t", self.index(ty)) })
                }
            }
            Type::Sum(_) => c_type!(named(#{ format!("mal_repr_sum_{}_t", self.index(ty)) })),
            Type::Function { .. } => {
                unreachable!("type checking excludes functions from extern signatures")
            }
            Type::Symbol | Type::Parameter { .. } | Type::Buffer(_) | Type::Opaque { .. } => {
                unreachable!("these types are not host mappable")
            }
        }
    }

    fn declarations(&self, host: &HostTypes, public: bool) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for ty in &self.aggregates {
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
                Type::Symbol | Type::Parameter { .. } | Type::Buffer(_) | Type::Opaque { .. } => {
                    unreachable!("these types never enter the C host registry")
                }
            };
            let id = self.index(ty);
            let guard = format!("MAL_DETAIL_RAW_REPR_{id}_DECLARED");
            let name = format!("{kind}_{id}");
            output.push(c_directive!(ifndef #{ guard.clone() }));
            output.push(c_directive!(define #{ guard };));
            output.push(c_declaration!(type #{ name } = struct(#{ format!("{kind}_{id}") })));
            output.push(c_directive!(endif));
        }
        if !output.is_empty() {
            output.blank_line();
        }
        for ty in &self.aggregates {
            if host.external_contains(ty) != public {
                continue;
            }
            if matches!(ty, Type::Product(_) | Type::Sum(_)) {
                self.append_repr_descriptor(&mut output, ty);
            }
            let guard = format!("MAL_DETAIL_RAW_REPR_{}_DEFINED", self.index(ty));
            output.push(c_directive!(ifndef #{ guard.clone() }));
            output.push(c_directive!(define #{ guard };));
            match ty {
                Type::Product(_) => {
                    let tag = format!("MalRepr_Product_{}", self.index(ty));
                    output.push(c_macro_invocation! {
                        "MAL_DETAIL_DEFINE_PRODUCT_REPR"([
                            id(#{ tag }),
                            id(#{ format!("MAL_DETAIL_REPR_FIELDS_{}", self.index(ty)) }),
                            id("MAL_DETAIL_RAW_REPR_FIELD"),
                        ])
                    });
                }
                Type::Sum(members) => {
                    let tag = format!("MalRepr_Sum_{}", self.index(ty));
                    let template = if members.is_empty() {
                        "MAL_DETAIL_DEFINE_EMPTY_SUM_REPR"
                    } else {
                        "MAL_DETAIL_DEFINE_SUM_REPR"
                    };
                    let mut arguments = vec![c_expr!(id(#{ tag }))];
                    if !members.is_empty() {
                        arguments.push(c_expr! {
                            id(#{ format!("MAL_DETAIL_REPR_FIELDS_{}", self.index(ty)) })
                        });
                        arguments.push(c_expr!(id("MAL_DETAIL_RAW_REPR_FIELD")));
                    }
                    output.push(MacroInvocation::new(template, arguments));
                }
                Type::Function { parameter, result } => {
                    let tag = format!("MalRepr_Closure_{}", self.index(ty));
                    let fields = c_aggregate_fields! {
                        fn "call"(
                            _: ptr(named("MalContext")),
                            _: ptr(const(named("void"))),
                            _: #{ self.c_type(parameter) },
                        ) -> #{ self.c_type(result) },
                        "environment": ptr(const(named("void"))),
                        fn "destroy_environment"(
                            _: ptr(named("MalContext")),
                            _: ptr(const(named("void"))),
                        ) -> named("void"),
                    };
                    output.push(c_aggregate!(struct #{ tag } { ...#{ fields } }));
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
                Type::Symbol | Type::Parameter { .. } | Type::Buffer(_) | Type::Opaque { .. } => {
                    unreachable!("these types never enter the C host registry")
                }
            }
            output.push(c_directive!(endif));
            output.blank_line();
        }
        output
    }

    fn append_repr_descriptor(&self, output: &mut TranslationUnit, ty: &Type) {
        const FIELDS_PER_CHUNK: usize = 32;

        let (member_prefix, elements) = match ty {
            Type::Product(elements) => ("field", elements.as_ref()),
            Type::Sum(members) => ("variant", members.as_ref()),
            _ => unreachable!("only products and sums have field descriptors"),
        };
        let id = self.index(ty);
        let descriptor = format!("MAL_DETAIL_REPR_FIELDS_{id}");
        let guard = format!("{descriptor}_DEFINED");
        output.push(c_directive!(ifndef #{ guard.clone() }));
        output.push(c_directive!(define #{ guard };));

        let fields = elements
            .iter()
            .enumerate()
            .map(|(index, element)| {
                let (to_host, to_raw) = self.repr_field_conversions(ty, element);
                MacroInvocation::new(
                    "field",
                    [
                        c_expr!(id("context")),
                        c_expr!(number(#{ index })),
                        c_expr!(id(#{ format!("{member_prefix}_{index}") })),
                        c_expr!(id(#{ self.c_type(element).to_string() })),
                        c_expr!(id(#{ self.host_value_c_type(element, None).to_string() })),
                        c_expr!(id(#{ to_host })),
                        c_expr!(id(#{ to_raw })),
                    ],
                )
            })
            .collect::<Vec<_>>();

        if fields.len() <= FIELDS_PER_CHUNK {
            output.push(Directive::invocations_define(
                descriptor,
                ["field", "context"],
                fields,
            ));
        } else {
            let mut chunks = Vec::new();
            for (chunk_index, fields) in fields.chunks(FIELDS_PER_CHUNK).enumerate() {
                let chunk = format!("{descriptor}_{chunk_index}");
                output.push(Directive::invocations_define(
                    chunk.clone(),
                    ["field", "context"],
                    fields.iter().cloned(),
                ));
                chunks.push(MacroInvocation::new(
                    chunk,
                    [c_expr!(id("field")), c_expr!(id("context"))],
                ));
            }
            output.push(Directive::invocations_define(
                descriptor,
                ["field", "context"],
                chunks,
            ));
        }
        output.push(c_directive!(endif));
    }

    fn repr_field_conversions(&self, aggregate: &Type, element: &Type) -> (String, String) {
        let identity = || "MAL_DETAIL_REPR_IDENTITY".to_string();
        let aggregate_to_host = || match element {
            Type::Product(_) | Type::Sum(_) if !is_bool(element) => {
                format!("mal_detail_to_host_{}", self.index(element))
            }
            Type::External { name, .. } => format!("mal_detail_to_host_{name}"),
            _ => identity(),
        };
        let aggregate_to_raw = || match element {
            Type::Unit => "mal_detail_convert_Unit".into(),
            Type::Product(_) => format!("mal_repr_product_{}_return", self.index(element)),
            Type::Sum(_) if !is_bool(element) => {
                format!("mal_detail_to_raw_{}", self.index(element))
            }
            Type::External { name, .. } => format!("mal_{name}_return"),
            Type::Address => "mal_Address_return".into(),
            _ => identity(),
        };
        if matches!(aggregate, Type::Product(_)) {
            return (aggregate_to_host(), aggregate_to_raw());
        }

        let scalar = |ty: &Type| {
            if is_bool(ty) {
                return "Bool";
            }
            match ty {
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
                Type::Address => "Address",
                Type::ByteSize => "ByteSize",
                Type::USize => "USize",
                _ => unreachable!("only host scalar types have builtin conversion helpers"),
            }
        };
        let sum_conversion = |direction: &str| match element {
            Type::Unit => "mal_detail_convert_Unit".into(),
            Type::Product(_) => {
                if direction == "host" {
                    format!("mal_detail_to_host_{}", self.index(element))
                } else {
                    format!("mal_repr_product_{}_return", self.index(element))
                }
            }
            Type::Sum(_) if !is_bool(element) => {
                format!("mal_detail_to_{direction}_{}", self.index(element))
            }
            Type::External { name, .. } => {
                if direction == "host" {
                    format!("mal_detail_to_host_{name}")
                } else {
                    format!("mal_{name}_return")
                }
            }
            _ => format!("mal_{}_return", scalar(element)),
        };
        (sum_conversion("host"), sum_conversion("raw"))
    }
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
                c_type!(named(#{ expected }))
            );
        }
        assert_eq!(
            registry.host_value_c_type(&product, None),
            c_type!(named("mal_repr_product_1e5f7ae9f35ae3d3_t"))
        );
        assert_eq!(
            registry.host_value_c_type(&sum, None),
            c_type!(named("mal_repr_sum_a47b44facfce4b92_t"))
        );
        assert_eq!(
            registry.host_value_c_type(&Type::UInt64, Some("Count")),
            c_type!(named("mal_Count_t"))
        );
        assert_eq!(
            registry.host_value_c_type(&product, Some("Packet")),
            c_type!(named("mal_Packet_t"))
        );
    }

    #[test]
    fn maps_bool_before_its_structural_sum_representation() {
        let bool_type = Type::Sum(vec![Type::Unit, Type::Unit].into());
        assert_eq!(
            TypeRegistry::default().host_value_c_type(&bool_type, None),
            c_type!(named("mal_Bool_t"))
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
            declarations.contains(
                "typedef struct mal_detail_repr_product_1e5f7ae9f35ae3d3 mal_repr_product_1e5f7ae9f35ae3d3_t;"
            ),
            "{declarations}"
        );
        assert!(declarations.contains(
            "typedef struct mal_detail_repr_sum_a47b44facfce4b92 mal_repr_sum_a47b44facfce4b92_t;"
        ));
        assert!(declarations.contains("typedef mal_repr_product_1e5f7ae9f35ae3d3_t mal_Packet_t;"));
        assert!(declarations.contains("typedef mal_repr_sum_a47b44facfce4b92_t mal_Result_t;"));
        assert!(declarations.contains(concat!(
            "field(context, 1, field_1, MalType_Address, mal_Address_t, ",
            "MAL_DETAIL_REPR_IDENTITY, mal_Address_return)"
        )));
        assert!(declarations.contains(concat!(
            "field(context, 1, variant_1, MalRepr_Product_1e5f7ae9f35ae3d3, ",
            "mal_repr_product_1e5f7ae9f35ae3d3_t, ",
            "mal_detail_to_host_1e5f7ae9f35ae3d3, ",
            "mal_repr_product_1e5f7ae9f35ae3d3_return)"
        )));
    }

    #[test]
    fn chunks_large_representation_descriptors() {
        let product = Type::Product(vec![Type::UInt8; 33].into());
        let mut registry = TypeRegistry::default();
        registry.collect(&product);
        let host = HostTypes {
            types: vec![product],
            ..HostTypes::default()
        };

        let declarations = registry.host_value_declarations(&host, &[]).render();

        assert!(declarations.contains("#define MAL_DETAIL_REPR_FIELDS_"));
        assert!(declarations.contains("_0(field, context)"));
        assert!(declarations.contains("_1(field, context)"));
        assert!(declarations.contains("_0(field, context) \\\n"));
        assert!(declarations.contains("_1(field, context)\n"));
    }
}
