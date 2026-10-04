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
fn renders_function_pointer_declarators() {
    let declaration = VariableDeclaration::function_pointer(
        "int32_t",
        "call",
        [crate::backend::c::syntax::c_parameter!(_ : *const void)],
    );

    assert_eq!(declaration.render(), "int32_t (*call)(const void *)");
}
