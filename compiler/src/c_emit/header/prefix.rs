use crate::c_emit::syntax::{
    AggregateDefinition, AggregateField, Attribute, Comment, Declaration, Directive, Expr,
    FunctionSignature, Initializer, Parameter, PastePart, PreprocessorExpr, TranslationUnit,
    TypeName,
};

use super::append_function;

pub(super) fn emit_prefix() -> TranslationUnit {
    let mut output = TranslationUnit::default();
    for directive in [
        Directive::Ifndef("MAL_PROGRAM_MAL_H".into()),
        Directive::define_empty("MAL_PROGRAM_MAL_H"),
    ] {
        output.push(directive);
    }
    output.blank_line();
    output.push(Directive::include_system("stdint.h"));
    output.blank_line();
    for directive in [
        Directive::define_expr("MAL_C_ABI_VERSION", Expr::number("0x000500u")),
        Directive::function_alias(
            "MAL_TYPE",
            ["name"],
            [PastePart::text("MalType_"), PastePart::parameter("name")],
        ),
        Directive::function_alias(
            "MAL_OPERATION",
            ["type", "operation"],
            [
                PastePart::text("mal_"),
                PastePart::parameter("type"),
                PastePart::text("_"),
                PastePart::parameter("operation"),
            ],
        ),
        Directive::function_alias(
            "MAL_TAG",
            ["type", "variant"],
            [
                PastePart::text("MAL_"),
                PastePart::parameter("type"),
                PastePart::text("_TAG_"),
                PastePart::parameter("variant"),
            ],
        ),
        Directive::function_alias(
            "MAL_EXTERN",
            ["name"],
            [PastePart::text("mal_ext_"), PastePart::parameter("name")],
        ),
    ] {
        output.push(directive);
    }
    output.blank_line();
    output.push(Directive::If(PreprocessorExpr::logical_or(
        PreprocessorExpr::defined("__clang__"),
        PreprocessorExpr::defined("__GNUC__"),
    )));
    output.push(Directive::define_attribute(
        "MAL_DETAIL_MAYBE_UNUSED",
        Attribute::Unused,
    ));
    output.push(Directive::Else);
    output.push(Directive::define_empty("MAL_DETAIL_MAYBE_UNUSED"));
    output.push(Directive::Endif);
    output.blank_line();
    output.push(Comment::new("Runtime API"));
    output.blank_line();
    output.push(Declaration::type_alias(
        TypeName::structure("MalContext"),
        "MalContext",
    ));
    output.push(AggregateDefinition::typedef_structure(
        None,
        [AggregateField::variable("uint8_t", "unused")],
        "MalType_Unit",
    ));
    for (source, alias) in [
        ("uint8_t", "MalType_Bool"),
        ("int8_t", "MalType_Int8"),
        ("int16_t", "MalType_Int16"),
        ("int32_t", "MalType_Int32"),
        ("int64_t", "MalType_Int64"),
        ("uint8_t", "MalType_UInt8"),
        ("uint16_t", "MalType_UInt16"),
        ("uint32_t", "MalType_UInt32"),
        ("uint64_t", "MalType_UInt64"),
        ("float", "MalType_Float32"),
        ("double", "MalType_Float64"),
    ] {
        output.push(Declaration::type_alias(source, alias));
    }
    output.push(AggregateDefinition::typedef_structure(
        None,
        [
            AggregateField::variable(TypeName::const_named("uint8_t").pointer(), "data"),
            AggregateField::variable("uint64_t", "length"),
        ],
        "MalType_Symbol",
    ));
    output.push(AggregateDefinition::typedef_structure(
        None,
        [AggregateField::variable(
            TypeName::named("uint8_t").pointer(),
            "address",
        )],
        "MalType_Ptr",
    ));
    output.blank_line();
    output.push(Directive::define_expr(
        "MAL_FALSE",
        Expr::cast(
            "MalType_Bool",
            Expr::named_call("UINT8_C", [Expr::number("0")]),
        ),
    ));
    output.push(Directive::define_expr(
        "MAL_TRUE",
        Expr::cast(
            "MalType_Bool",
            Expr::named_call("UINT8_C", [Expr::number("1")]),
        ),
    ));
    output.blank_line();
    output.push(Declaration::function(FunctionSignature::no_return(
        "void",
        "mal_trap",
        [
            Parameter::named(TypeName::named("MalContext").pointer(), "context"),
            Parameter::named(TypeName::const_named("char").pointer(), "message"),
        ],
    )));
    output.push(Declaration::function(FunctionSignature::new(
        "MalType_Symbol",
        "mal_Symbol_copy_from_bytes",
        [
            Parameter::named(TypeName::named("MalContext").pointer(), "context"),
            Parameter::named(TypeName::const_named("uint8_t").pointer(), "data"),
            Parameter::named("uint64_t", "length"),
        ],
    )));
    output.blank_line();
    append_function(
        &mut output,
        FunctionSignature::static_inline(
            TypeName::const_named("uint8_t").pointer(),
            "mal_Symbol_data",
            [Parameter::named("MalType_Symbol", "value")],
        ),
        Expr::identifier("value").field("data"),
    );
    append_function(
        &mut output,
        FunctionSignature::static_inline(
            "uint64_t",
            "mal_Symbol_length",
            [Parameter::named("MalType_Symbol", "value")],
        ),
        Expr::identifier("value").field("length"),
    );
    append_function(
        &mut output,
        FunctionSignature::static_inline(
            "MalType_Ptr",
            "mal_Ptr_from_address",
            [Parameter::named(
                TypeName::named("uint8_t").pointer(),
                "address",
            )],
        ),
        Expr::compound_literal(
            "MalType_Ptr",
            [Initializer::designated(
                "address",
                Expr::identifier("address"),
            )],
        ),
    );
    append_function(
        &mut output,
        FunctionSignature::static_inline(
            TypeName::named("uint8_t").pointer(),
            "mal_Ptr_address",
            [Parameter::named("MalType_Ptr", "value")],
        ),
        Expr::identifier("value").field("address"),
    );
    output
}
