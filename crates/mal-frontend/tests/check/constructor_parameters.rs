//! Constructor type parameters forwarded between generic bindings: kind variables kept by the caller,
//! kind requirements of a body checked at specialization, and keys over kind-polymorphic constructors.

use super::*;

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

#[test]
fn selects_a_key_over_a_kind_polymorphic_opaque_constructor() {
    // The phantom parameters of `Tag` make it kind-polymorphic, so the key and the explicit argument
    // spell its kind variables differently; selection must still match the constructor.
    let program = check_ok(
        "opaque Tag<M, V> :: UInt64;\n\
         width<F, M, V> :: F<M, V> -> USize;\n\
         width<Tag, M, V> :: Tag<M, V> -> USize := (_) -> 1usize;\n\
         tagged :: Tag<USize, UInt64> := 7u64;\n\
         main :: Unit -> Int32 := () -> if (width<Tag>(tagged) == 1usize) then 0 else 1;",
    );

    check::specialize(program).expect("select the implementation keyed by the opaque constructor");
}
