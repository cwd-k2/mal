use mal_frontend::resolve::ast::TypeId;

use super::*;

#[test]
fn maps_every_boundary_value_to_one_public_carrier() {
    let product = Type::Product(vec![Type::UInt64, Type::Symbol].into());
    let sum = Type::Sum(vec![Type::Unit, product.clone()].into());
    let mut registry = TypeRegistry::default();
    registry.collect(&sum);

    for (ty, expected) in [
        (Type::Unit, "mal_Unit_t"),
        (Type::Int32, "mal_Int32_t"),
        (Type::UInt64, "mal_UInt64_t"),
        (Type::Float64, "mal_Float64_t"),
        (Type::Symbol, "mal_Symbol_t"),
        (Type::Buffer(Type::UInt8.into()), "mal_Buffer_t"),
        (
            Type::External {
                id: TypeId(3),
                name: "File".into(),
            },
            "mal_File_t",
        ),
    ] {
        assert_eq!(registry.host_value_c_type(&ty, None), c_type!({ expected }));
    }
    assert_eq!(
        registry.host_value_c_type(&product, None),
        c_type!(mal_repr_product_1e5f7ae9f35ae3d3_t)
    );
    assert_eq!(
        registry.host_value_c_type(&sum, None),
        c_type!(mal_repr_sum_11ce2ff4640b9e1d_t)
    );
    assert_eq!(
        registry.host_value_c_type(&Type::UInt64, Some("Count")),
        c_type!(mal_Count_t)
    );
    assert_eq!(
        registry.host_value_c_type(&product, Some("Packet")),
        c_type!(mal_Packet_t)
    );
}

#[test]
fn maps_bool_before_its_structural_sum_representation() {
    let bool_type = Type::Sum(vec![Type::Unit, Type::Unit].into());
    assert_eq!(
        TypeRegistry::default().host_value_c_type(&bool_type, None),
        c_type!(mal_Bool_t)
    );
}

#[test]
fn declares_host_aggregates_in_structural_dependency_order() {
    let product = Type::Product(vec![Type::UInt64, Type::Symbol].into());
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
            span: mal_syntax::source::Span::new(mal_syntax::source::FileId::new(0), 0, 0),
        },
        crate::core::ast::TypeAlias {
            name: "Result".into(),
            ty: sum,
            element_aliases: vec![None, Some("Packet".into())],
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
        "typedef struct mal_detail_repr_sum_11ce2ff4640b9e1d mal_repr_sum_11ce2ff4640b9e1d_t;"
    ));
    assert!(declarations.contains("typedef mal_repr_product_1e5f7ae9f35ae3d3_t mal_Packet_t;"));
    assert!(declarations.contains("typedef mal_repr_sum_11ce2ff4640b9e1d_t mal_Result_t;"));
    assert!(declarations.contains("field(context, 1, field_1, mal_Symbol_t)"));
    assert!(declarations.contains(concat!(
        "field(context, 1, variant_1, ",
        "mal_repr_product_1e5f7ae9f35ae3d3_t)"
    )));
    assert!(!declarations.contains("MalRepr_"));
    assert!(!declarations.contains("to_raw"));
    assert!(!declarations.contains("to_host"));
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
