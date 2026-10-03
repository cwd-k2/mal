//! Implementation keys as type patterns: names that are not visible types become variables, and a
//! constructor position may hold an opaque type or Buffer partially applied to them.

use super::*;

const EITHER: &str = "opaque Either<E, A> :: [E, A];\n\
    pure<F, A> :: A -> F<A>;\n\
    bind<F, A, B> :: (F<A>, A -> F<B>) -> F<B>;\n";

#[test]
fn selects_a_partially_applied_key_for_every_argument() {
    let program = check_ok(&format!(
        "{EITHER}\
         fmap<F, A, B> :: (F<A>, A -> B) -> F<B> := (value, function) ->\n\
             bind<F>(value, (element) -> pure<F, B>(function(element)));\n\
         pure<Either<E>, A> :: A -> Either<E, A> := (value) -> [failure, success] => success(value);\n\
         bind<Either<E>, A, B> :: (Either<E, A>, A -> Either<E, B>) -> Either<E, B> :=\n\
             (value, next) -> [failure, success] =>\n\
                 value[failure, (element) -> next(element)[failure, success]];\n\
         describe<E> :: E -> USize;\n\
         describe<Symbol> :: Symbol -> USize := (text) -> #text;\n\
         failureSize<F, A> :: F<A> -> USize;\n\
         failureSize<Either<E>, A> :: Either<E, A> -> USize :=\n\
             (value) -> value[(error) -> describe(error), (_) -> 0usize];\n\
         main :: Unit -> Int32 := () -> {{\n\
             text := fmap<Either<Symbol>>(pure<Either<Symbol>>(20i32), (value) -> value + 22i32);\n\
             code := fmap<Either<UInt8>>(pure<Either<UInt8>>(1i32), (value) -> value * 3i32);\n\
             failed :: Either<Symbol, Int32> := [failure, success] => failure(\"boom\");\n\
             failureSize<Either<Symbol>>(failed).i32;\n\
         }};"
    ));

    check::specialize(program).expect("one implementation serves every error type");
}

#[test]
fn rejects_a_partial_key_that_overlaps_a_closed_one() {
    let error = check_error(&format!(
        "{EITHER}\
         pure<Either<E>, A> :: A -> Either<E, A> := (value) -> [failure, success] => success(value);\n\
         pure<Either<Symbol>, A> :: A -> Either<Symbol, A> :=\n\
             (value) -> [failure, success] => success(value);"
    ));

    assert_eq!(error.message, "duplicate operation implementation");
}

#[test]
fn accepts_partial_keys_with_different_heads() {
    check_ok(&format!(
        "{EITHER}\
         opaque Result<E, A> :: [E, A];\n\
         pure<Either<E>, A> :: A -> Either<E, A> := (value) -> [failure, success] => success(value);\n\
         pure<Result<E>, A> :: A -> Result<E, A> := (value) -> [failure, success] => success(value);"
    ));
}

#[test]
fn rejects_a_variable_key_constructor_without_a_nominal_head() {
    for key in ["Pair<E>", "G"] {
        let error = check_error(&format!(
            "{EITHER}\
             Pair<E, A> :: (E, A);\n\
             pure<{key}, A> :: A -> Pair<UInt8, A> := (value) -> (0u8, value);"
        ));

        assert_eq!(
            error.message, "operation constructor key needs a nominal head",
            "key `{key}`"
        );
    }
}

#[test]
fn checks_storable_requirements_of_a_selected_implementation() {
    let header = "wrap<F, A> :: A -> F<Buffer<A>>;\n\
        opaque Id<A> :: A;\n\
        wrap<Id, A> :: A -> Id<Buffer<A>> := (value) -> {\n\
            values := make<A>(1usize);\n\
            values.new(value);\n\
            values;\n\
        };\n";

    let accepted = check_ok(&format!(
        "{header}main :: Unit -> Int32 := () -> {{ wrap<Id>(1i32); 0i32; }};"
    ));
    check::specialize(accepted).expect("a storable element");

    let rejected = check_ok(&format!(
        "{header}main :: Unit -> Int32 := () -> {{\n\
             callback :: Int32 -> Int32 := (value) -> value;\n\
             wrap<Id>(callback);\n\
             0i32;\n\
         }};"
    ));
    let error = check::specialize(rejected).expect_err("a function element");
    assert_eq!(
        error.message,
        "operation instance violates a Storable requirement"
    );
}

#[test]
fn explains_a_catch_all_key_made_of_unknown_names() {
    let error = check_error(
        "equal<A> :: (A, A) -> Bool;\n\
         equal<Int23> :: (Int23, Int23) -> Bool := (left, right) -> true;",
    );

    assert_eq!(
        error.message,
        "generic operation implementation requires structure"
    );
    assert!(
        error
            .notes
            .iter()
            .any(|note| note.contains("`Int23` names no visible type")),
        "{:?}",
        error.notes
    );
}

#[test]
fn suggests_the_type_a_misspelled_key_binder_may_name() {
    let suggestion = "did you mean the type `Int32`?";
    let catch_all = check_error(
        "equal<A> :: (A, A) -> Bool;\n\
         equal<Int23> :: (Int23, Int23) -> Bool := (left, right) -> true;",
    );
    assert!(
        catch_all.notes.iter().any(|note| note.contains(suggestion)),
        "{:?}",
        catch_all.notes
    );

    // The body treats the binder as the integer type it was meant to be.
    let body = check_error(
        "first<F, A> :: F<A> -> A;\n\
         first<Buffer, Int23> :: Buffer<Int23> -> Int23 := (values) -> values.get(0usize) + 1i32;",
    );
    assert!(
        body.notes.iter().any(|note| note.contains(suggestion)),
        "{:?}",
        body.notes
    );

    // A single-letter binder is an ordinary type variable and is never suspected.
    let single = check_error(
        "first<F, A> :: F<A> -> A;\n\
         first<Buffer, I> :: Buffer<I> -> I := (values) -> values.get(0usize) + 1i32;",
    );
    assert!(
        !single
            .notes
            .iter()
            .any(|note| note.contains("did you mean")),
        "{:?}",
        single.notes
    );
}
