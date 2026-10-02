use super::*;

#[test]
fn opaque_types_use_their_representation_only_in_the_declaring_file() {
    let program = check_ok(
        "opaque Pair<A> :: (A, A);\n\
         makePair<A> :: (A, A) -> Pair<A> := (pair) -> pair;\n\
         first<A> :: Pair<A> -> A := ((first, _)) -> first;\n\
         main :: Unit -> Int32 := () -> first(makePair((40i32, 2i32)));",
    );

    let TopItem::OpaqueType { .. } = &program.items[0].kind else {
        panic!("expected opaque declaration");
    };
    let specialized = check::specialize(program).expect("erase opaque boundaries");
    assert!(
        specialized
            .program()
            .items
            .iter()
            .all(|item| { !matches!(item.kind, TopItem::OpaqueType { .. }) })
    );
}

#[test]
fn opaque_sum_uses_existing_construction_and_elimination_syntax() {
    let program = check_ok(
        "opaque Option<A> :: [Unit, A];\n\
         none<A> :: Unit -> Option<A> := () -> [none, some] => none();\n\
         isNone<A> :: Option<A> -> Bool := (value) -> value[\n\
             () -> true,\n\
             (_) -> false\n\
         ];\n\
         main :: Unit -> Int32 := () -> {\n\
             value :: Option<Int32> := none<Int32>();\n\
             if (isNone(value)) then 0 else 1;\n\
         };",
    );

    check::specialize(program).expect("erase opaque sum boundary");
}

#[test]
fn opaque_declarations_with_the_same_representation_remain_distinct() {
    let error = check_error(
        "opaque Left :: Int32;\n\
         opaque Right :: Int32;\n\
         wrong :: Left -> Right := (value) -> value;",
    );

    assert_eq!(error.message, "type mismatch");
}

#[test]
fn operators_see_the_representation_in_the_declaring_file() {
    check_ok(
        "opaque Counter :: Int32;
\
         opaque Name :: Symbol;
\
         start :: Counter := 1 + 2;
\
         next :: Counter -> Counter := (count) -> count + 1i32;
\
         back :: Counter -> Counter := (count) -> -count;
\
         same :: (Counter, Counter) -> Bool := (left, right) -> left == right;
\
         before :: (Counter, Counter) -> Bool := (left, right) -> left < right;
\
         join :: (Name, Name) -> Name := (left, right) -> left + right;
\
         split :: Name -> Name := (name) -> name / 1usize;
\
         size :: Name -> USize := (name) -> #name;",
    );
}

#[test]
fn an_opaque_type_equals_only_its_own_representation_layers() {
    check_ok(
        "opaque Inner :: Int32;
\
         opaque Outer :: Inner;
\
         wrap :: Inner -> Outer := (inner) -> inner;
\
         unwrap :: Outer -> Inner := (outer) -> outer;
\
         deep :: Int32 -> Outer := (value) -> value;",
    );
    assert_eq!(
        check_error(
            "opaque Inner :: Int32;
\
             opaque Other :: Int32;
\
             opaque Outer :: Inner;
\
             wrong :: Other -> Outer := (other) -> other;",
        )
        .message,
        "type mismatch"
    );
}

#[test]
fn generic_code_keeps_phantom_opaque_arguments_equal() {
    check_ok(
        "opaque Tagged<S, T> :: UInt64;\n\
         forward<T> :: Tagged<Unit, T> -> Unit := (value) -> ();\n\
         relay<T> :: Tagged<Unit, T> -> Unit := (value) -> forward<T>(value);\n\
         family<S, T> :: Tagged<S, T> -> Unit;\n\
         family<S, UInt64> :: Tagged<S, UInt64> -> Unit := (value) -> ();",
    );
}

#[test]
fn generic_results_view_opaque_layers_declared_in_the_file() {
    check_ok(
        "opaque Inner<T> :: (T, T);\n\
         build<T> :: T -> Inner<T> := (value) -> (value, value);\n\
         opaque Outer<T> :: Inner<T>;\n\
         explicit<T> :: T -> Outer<T> := (value) -> build<T>(value);\n\
         inferred :: Int32 -> Outer<Int32> := (value) -> build(value);",
    );
    assert_eq!(
        check_error(
            "opaque Left :: Int32;\n\
             opaque Right :: Int32;\n\
             same<T> :: (T, T) -> T := (first, _) -> first;\n\
             wrong :: (Left, Right) -> Left := (left, right) -> same(left, right);",
        )
        .message,
        "conflicting generic type inference"
    );
}

#[test]
fn phantom_type_arguments_are_inferred_unless_they_are_constructors() {
    check_ok(
        "opaque Tagged<T> :: UInt64;\n\
         tag<T> :: UInt64 -> Tagged<T> := (value) -> value;\n\
         untag<T> :: Tagged<T> -> UInt64 := (value) -> value;\n\
         fromOperand :: Tagged<Int32> -> UInt64 := (value) -> untag(value);\n\
         fromExpected :: UInt64 -> Tagged<Int32> := (value) -> tag(value);\n\
         forwarded<T> :: Tagged<T> -> UInt64 := (value) -> untag(value);\n\
         Ap<F, A> :: F<A>;\n\
         opaque Link<F, A> :: Tagged<Ap<F, A>>;\n\
         link<F, A> :: Link<F, A> -> Unit := (value) -> ();\n\
         prefixed :: Link<Buffer, Int32> -> Unit := (value) -> link<Buffer>(value);",
    );
    assert_eq!(
        check_error(
            "opaque Holder<F> :: UInt64;\n\
             hold<F> :: Holder<F> -> Unit := (value) -> ();\n\
             direct :: Holder<Buffer> -> Unit := (value) -> hold(value);",
        )
        .message,
        "generic type arguments cannot be inferred"
    );
    assert_eq!(
        check_error(
            "opaque Tagged<T> :: UInt64;\n\
             same<T> :: (Tagged<T>, Tagged<T>) -> Unit := (left, right) -> ();\n\
             mixed :: (Tagged<Int32>, Tagged<UInt8>) -> Unit := (left, right) -> same(left, right);",
        )
        .message,
        "conflicting generic type inference"
    );
}

#[test]
fn diagnostics_name_opaque_applications_with_their_arguments() {
    let error = check_error(
        "opaque Tagged<T> :: UInt64;\n\
         wrong :: Tagged<Int32> -> Tagged<UInt8> := (value) -> value;",
    );
    let label = error.primary.expect("primary label").message;
    assert!(label.contains("`Tagged<Int32>`"), "label: {label}");
}

#[test]
fn rejects_recursive_opaque_representations_even_when_unused() {
    assert_eq!(
        check_error("opaque Loop<A> :: (A, Loop<A>);").message,
        "recursive opaque representation"
    );
}

#[test]
fn opaque_buffer_uses_memory_operations_only_in_its_declaring_file() {
    let program = check_ok(
        "opaque Values<A> :: Buffer<A>;\n\
         values<A> :: USize -> Values<A> := (capacity) -> make<A>(capacity);\n\
         append<A> :: (Values<A>, A) -> USize := (items, value) -> items.new(value);\n\
         length<A> :: Values<A> -> USize := (items) -> #items;\n\
         main :: Unit -> Int32 := () -> {\n\
             items :: Values<Int32> := values<Int32>(1usize);\n\
             items.append(42i32);\n\
             if (items.length() == 1usize && items.get(0usize) == 42i32) then 0 else 1;\n\
         };",
    );

    check::specialize(program).expect("erase opaque Buffer boundary");
}

#[test]
fn source_opaque_types_do_not_cross_the_extern_boundary() {
    assert_eq!(
        check_error("opaque Counter :: Int32; extern inspect :: Counter -> Unit;").message,
        "external operation `inspect` uses a type that is not host mappable"
    );
}
