use super::{FunctionSignature, Parameter, TypeName, VariableDeclaration};

#[test]
fn renders_structured_function_signatures() {
    let signature = FunctionSignature::static_inline(
        TypeName::const_named("uint8_t").pointer(),
        "read_bytes",
        [
            Parameter::named(TypeName::named("MalContext").pointer(), "context"),
            Parameter::named("uint64_t", "length"),
        ],
    );

    assert_eq!(
        signature.render(),
        "static inline const uint8_t *read_bytes(MalContext *context, uint64_t length)"
    );
}

#[test]
fn renders_a_non_inline_function_contract() {
    let signature = FunctionSignature::static_noinline("uint8_t", "slow", []);

    assert_eq!(
        signature.render(),
        "static __attribute__((noinline)) uint8_t slow(void)"
    );
}

#[test]
fn renders_function_pointer_declarators() {
    let declaration = VariableDeclaration::function_pointer(
        "int32_t",
        "call",
        [Parameter::unnamed(TypeName::const_named("void").pointer())],
    );

    assert_eq!(declaration.render(), "int32_t (*call)(const void *)");
}
