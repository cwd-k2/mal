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
