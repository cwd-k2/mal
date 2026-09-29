use super::*;
use crate::closure::ast::AtomId;
use mal_frontend::check::ast::Type;
use mal_syntax::source::{FileId, Span};

fn reference(id: ValueId, index: usize) -> Atom {
    Atom {
        id: AtomId(index),
        kind: AtomKind::Reference(Reference::Binding(id)),
        ty: Type::Int32,
        span: Span::new(FileId::new(0), 0, 0),
    }
}

#[test]
fn follows_long_flat_alias_chains_without_host_recursion() {
    const ALIAS_COUNT: usize = 50_000;
    let root = ValueId::Temporary(0);
    let mut operations = Vec::with_capacity(ALIAS_COUNT + 1);
    operations.push(Operation::Product(Vec::new()));
    operations
        .extend((1..=ALIAS_COUNT).map(|index| {
            Operation::Atom(reference(ValueId::Temporary((index - 1) as u32), index))
        }));
    let bindings = operations
        .iter()
        .enumerate()
        .map(|(index, operation)| (ValueId::Temporary(index as u32), operation))
        .collect();
    let pass_through = ParameterPassThrough { bindings };
    let argument = reference(ValueId::Temporary(ALIAS_COUNT as u32), ALIAS_COUNT + 1);

    assert!(pass_through.resolves_to_binding(&argument, root));
    assert_eq!(
        pass_through.product_elements(&argument),
        Some([].as_slice())
    );
}

#[test]
fn stops_at_an_alias_cycle_in_malformed_control() {
    let operations = [
        Operation::Atom(reference(ValueId::Temporary(1), 0)),
        Operation::Atom(reference(ValueId::Temporary(0), 1)),
    ];
    let bindings = operations
        .iter()
        .enumerate()
        .map(|(index, operation)| (ValueId::Temporary(index as u32), operation))
        .collect();
    let pass_through = ParameterPassThrough { bindings };
    let argument = reference(ValueId::Temporary(0), 2);

    assert!(!pass_through.resolves_to_binding(&argument, ValueId::Temporary(2)));
    assert_eq!(pass_through.product_elements(&argument), None);
}
