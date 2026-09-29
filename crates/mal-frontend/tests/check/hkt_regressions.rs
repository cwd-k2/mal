use super::*;

#[test]
fn beta_reduces_composed_constructors_after_substitution() {
    let program = check_ok(
        "Compose<F, G, A> :: F<G<A>>;\n\
         Pair<A> :: (A, A);\n\
         value :: Compose<Pair, Pair, Int32> := ((1i32, 2i32), (3i32, 4i32));",
    );
    let pair = Type::Product(vec![Type::Int32, Type::Int32].into());

    assert_eq!(
        top_binding(&program, 0).value.ty,
        Type::Product(vec![pair.clone(), pair].into())
    );
}

#[test]
fn rejects_amplifying_constructor_composition_before_allocating_its_normal_form() {
    let source = "Pair<A> :: (A, A);\n\
                  Twice<F, A> :: F<F<A>>;\n\
                  T0 :: Pair;\n\
                  T1 :: Twice<T0>;\n\
                  T2 :: Twice<T1>;\n\
                  T3 :: Twice<T2>;\n\
                  T4 :: Twice<T3>;\n\
                  T5 :: Twice<T4>;";

    let error = check_error(source);

    assert_eq!(error.message, "type normalization is too large");
    let primary = error.primary.expect("normalization diagnostic");
    assert_eq!(
        primary.span.start(),
        source.find("T5 :: Twice<T4>").unwrap()
    );
    assert!(primary.message.contains("65536 normalization steps"));
}

#[test]
fn reports_delayed_generic_normalization_instead_of_panicking() {
    let fields = vec!["A"; 65_536].join(", ");
    let source = format!(
        "Huge<A> :: ({fields});\n\
         take<F, A> :: F<A> -> Unit := (_) -> ();\n\
         value := take<Huge, Unit>;"
    );

    let error = check_error(&source);

    assert_eq!(error.message, "type normalization is too large");
    assert_eq!(
        error
            .primary
            .expect("normalization diagnostic")
            .span
            .start(),
        source.find("take<Huge, Unit>").unwrap()
    );
}

#[test]
fn rejects_flat_parameter_lists_before_constructing_a_deep_kind() {
    let parameters = (0..=256)
        .map(|index| format!("T{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let error = check_error(&format!("Wide<{parameters}> :: Unit;"));

    assert_eq!(error.message, "type kind is too large");
    assert!(
        error
            .primary
            .expect("kind admission diagnostic")
            .message
            .contains("at most 256 type parameters")
    );
}

#[test]
fn rejects_kind_growth_across_shallow_declarations() {
    fn chain(last: usize) -> String {
        let mut source = "K0<A> :: A;\n".to_owned();
        for level in 1..=last {
            source.push_str(&format!("K{level}<F> :: F<K{}>;\n", level - 1));
        }
        source
    }

    check_ok(&chain(40));
    let source = chain(300);

    let error = check_error(&source);

    assert_eq!(error.message, "type kind is too large");
    let primary = error.primary.expect("kind admission diagnostic");
    assert!(primary.message.contains("256 nested function levels"));
    assert!(primary.span.start() > source.find("K100<F>").unwrap());
}

#[test]
fn binds_type_parameters_beside_a_closed_constructor_key() {
    let program = check_ok(
        "Pair<A> :: (A, A);\n\
         first<F, A> :: F<A> -> A;\n\
         first<Pair, A> :: Pair<A> -> A := (left, _) -> left;\n\
         main :: Unit -> Int32 := () -> first<Pair>((42i32, 0i32));",
    );

    check::specialize(program).expect("select a constructor implementation generic in its element");
}

#[test]
fn forwards_rigid_arguments_to_an_operation_requirement() {
    let program = check_ok(
        "Pair<A> :: (A, A);\n\
         extent<F, A> :: F<A> -> USize;\n\
         extent<Pair, A> :: Pair<A> -> USize := (_) -> 2usize;\n\
         extentOf<F, A> :: F<A> -> USize := (value) -> extent<F, A>(value);\n\
         main :: Unit -> Int32 := () ->\n\
             if (extentOf<Pair>((20i32, 22i32)) == 2usize) then 0 else 1;",
    );

    check::specialize(program).expect("forward a constructor operation requirement");
}
