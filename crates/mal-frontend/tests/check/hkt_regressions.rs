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

    check_ok(&chain(100));
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

#[test]
fn forwards_a_binary_constructor_parameter_with_kind_polymorphic_arguments() {
    // `M` and `V` only reach `F`, so their kinds are variables. Normalizing the explicit arguments of
    // `swap<F, M, V>` must keep the caller's kind variables, or the caller's `F<M>` would differ from
    // itself. Supplying only `F` must still infer `M` and `V` from the operand.
    let program = check_ok(
        "opaque Ix<M, V> :: (Buffer<M>, Buffer<V>);\n\
         swap<F, M, V> :: (F<M, V>, M) -> (F<M, V>, M);\n\
         swap<Ix, M, V> :: (Ix<M, V>, M) -> (Ix<M, V>, M) := (pool, next) -> (pool, next);\n\
         explicit<F, M, V> :: (F<M, V>, M) -> F<M, V> := (pool, value) -> {\n\
             (next, _) := swap<F, M, V>(pool, value);\n\
             next;\n\
         };\n\
         inferred<F, M, V> :: F<M, V> -> F<M, V> := (pool) -> pool;\n\
         forwarded<F, M, V> :: F<M, V> -> F<M, V> := (pool) -> inferred<F>(pool);\n\
         main :: Unit -> Int32 := () -> {\n\
             meta :: Buffer<USize> := make(1usize);\n\
             values :: Buffer<UInt64> := make(0usize);\n\
             pool :: Ix<USize, UInt64> := (meta, values);\n\
             _ := forwarded<Ix>(explicit<Ix>(pool, 1usize));\n\
             0;\n\
         };",
    );

    check::specialize(program).expect("forward a binary constructor parameter");
}

#[test]
fn checks_a_body_kind_requirement_when_the_instance_is_concrete() {
    // `V` only reaches `F`, so its kind is open in the signature, while the body passes it where kind
    // `Type` is needed. The body checks, and each instance is checked when it is specialized.
    let body = "peek<F, M, V> :: (F<M, V>, USize) -> [Unit, V];\n\
                live<F, M, V> :: (F<M, V>, USize) -> Bool := (pool, index) ->\n\
                    peek<F, M, V>(pool, index)[() -> false, (_) -> true];\n";
    let accepted = check_ok(&format!(
        "opaque Ix<M, V> :: (Buffer<M>, Buffer<V>);\n\
         {body}\
         peek<Ix, M, V> :: (Ix<M, V>, USize) -> [Unit, V] :=\n\
             ((_, values), index) -> [empty, full] => full(values.get(index));\n\
         main :: Unit -> Int32 := () -> {{\n\
             meta :: Buffer<USize> := make(1usize);\n\
             values :: Buffer<UInt64> := make(0usize);\n\
             pool :: Ix<USize, UInt64> := (meta, values);\n\
             if (live<Ix>(pool, 0usize)) then 1 else 0;\n\
         }};"
    ));
    check::specialize(accepted).expect("an instance that gives `V` kind Type");

    let rejected = check_ok(&format!(
        "opaque Holder<M, G> :: (Buffer<M>, G<UInt64>);\n\
         {body}\
         main :: Unit -> Int32 := () -> {{\n\
             meta :: Buffer<USize> := make(1usize);\n\
             values :: Buffer<UInt64> := make(0usize);\n\
             holder :: Holder<USize, Buffer> := (meta, values);\n\
             if (live<Holder, USize, Buffer>(holder, 0usize)) then 1 else 0;\n\
         }};"
    ));
    let error = check::specialize(rejected).expect_err("an instance that gives `V` a constructor");
    assert_eq!(
        error.message,
        "generic instance violates a kind requirement"
    );
    let primary = error.primary.expect("kind requirement diagnostic");
    assert!(primary.message.contains("needs kind `Type`"));
}

#[test]
fn infers_value_arguments_beside_a_constructor_at_a_narrower_kind() {
    // `isLive` leaves the kind of `V` open while `peek` needs kind `Type`; naming only `F` must still infer
    // `M` and `V` from the operand.
    check_ok(
        "peek<F, M, V> :: (F<M, V>, USize) -> [Unit, V];\n\
         isLive<F, M, V> :: (F<M, V>, USize) -> Bool := (pool, index) ->\n\
             peek<F>(pool, index)[() -> false, (_) -> true];\n\
         main :: Unit -> Int32 := () -> 0;",
    );
}
