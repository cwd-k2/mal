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
