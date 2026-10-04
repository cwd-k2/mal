use super::*;

#[test]
fn collects_shared_aggregate_dags_in_dependency_order() {
    let mut ty = Type::Unit;
    for _ in 0..64 {
        ty = Type::Product(vec![ty.clone(), ty].into());
    }
    let mut registry = TypeRegistry::default();

    registry.collect(&ty);

    assert_eq!(registry.aggregates.len(), 64);
    assert_eq!(registry.aggregates.last(), Some(&ty));
}

#[test]
fn interns_independent_structurally_equal_dags() {
    let mut left = Type::UInt8;
    let mut right = Type::UInt8;
    for _ in 0..64 {
        left = Type::Sum(vec![left.clone(), left].into());
        right = Type::Sum(vec![right.clone(), right].into());
    }
    let mut registry = TypeRegistry::default();

    registry.collect(&left);
    registry.collect(&right);

    assert_eq!(registry.aggregates.len(), 64);
    assert_eq!(registry.index(&left), registry.index(&right));
}

#[test]
fn registers_host_visible_alias_dags_before_rendering() {
    let mut external = Type::UInt8;
    let mut alias = Type::UInt8;
    for _ in 0..64 {
        external = Type::Sum(vec![external.clone(), external].into());
        alias = Type::Sum(vec![alias.clone(), alias].into());
    }
    let interface = ProgramInterface {
        type_aliases: vec![crate::core::ast::TypeAlias {
            name: "Alias".into(),
            ty: alias.clone(),
            element_aliases: vec![None, None],
            span: mal_syntax::source::Span::new(mal_syntax::source::FileId::new(0), 0, 0),
        }],
        external_types: Vec::new(),
        externals: vec![crate::core::ast::ExternalOperation {
            id: mal_frontend::resolve::ast::ExternalOperationId(0),
            name: "inspect".into(),
            parameter: external.clone(),
            parameter_alias: Some("Alias".into()),
            parameter_aliases: vec![None, None],
            result: Type::Unit,
            result_alias: None,
            span: mal_syntax::source::Span::new(mal_syntax::source::FileId::new(0), 0, 0),
        }],
    };
    let mut registry = TypeRegistry::default();

    HostTypes::collect(&interface, &mut registry);

    assert_eq!(registry.index(&external), registry.index(&alias));
}
