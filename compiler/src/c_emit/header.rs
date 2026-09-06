use crate::core::ast::ProgramInterface;

use super::{HostTypes, TypeRegistry, host_signature::HostSignature};
use crate::c_emit::syntax::{
    AggregateDefinition, AggregateField, Block, Comment, Declaration, Directive, Expr,
    FunctionDefinition, FunctionSignature, Initializer, MacroInvocation, Parameter, PastePart,
    PreprocessorExpr, Statement, TranslationUnit, TypeName,
};

pub(super) fn emit(interface: &ProgramInterface, types: &TypeRegistry, host: &HostTypes) -> String {
    let signatures: Vec<_> = interface
        .externals
        .iter()
        .map(|external| HostSignature::new(external, types))
        .collect();
    let mut output = emit_prefix();
    let mut declarations = types.header_declarations(host);
    declarations.extend(types.header_alias_declarations(host, &interface.type_aliases));
    if !declarations.is_empty() {
        begin_section(&mut output, "Host-visible types");
        output.extend(declarations);
    }

    let mut helpers = types.header_opaque_helpers(host);
    helpers.extend(types.header_alias_helpers(host, &interface.type_aliases));
    if !helpers.is_empty() {
        begin_section(&mut output, "Type helpers");
        output.extend(helpers);
    }

    if !interface.externals.is_empty() {
        begin_section(&mut output, "External operations");
        for signature in &signatures {
            emit_external_declaration(&mut output, signature);
        }
        begin_section(&mut output, "External definition helpers");
        for signature in &signatures {
            emit_definition_macro(&mut output, signature);
        }
    }
    output.push(Directive::Endif);
    output.render()
}

pub(super) fn emit_host(
    interface: &ProgramInterface,
    types: &TypeRegistry,
    header_name: &str,
) -> String {
    let mut output = TranslationUnit::new([Directive::IncludeQuoted(header_name.into()).into()]);
    for external in &interface.externals {
        let signature = HostSignature::new(external, types);
        output.blank_line();
        let mut body = Block::default();
        for name in signature.parameter_names().into_iter().skip(1) {
            body.push(Statement::expression(Expr::cast(
                "void",
                Expr::identifier(name),
            )));
        }
        body.push(Statement::expression(Expr::named_call(
            "mal_trap",
            [
                Expr::identifier("context"),
                Expr::string(format!(
                    "external operation `{}` is not implemented",
                    signature.operation_name
                )),
            ],
        )));
        output.push(FunctionDefinition::from_macro(
            macro_invocation(&signature),
            body,
        ));
    }
    output.render()
}

fn emit_prefix() -> TranslationUnit {
    let mut output = TranslationUnit::default();
    for directive in [
        Directive::Ifndef("MAL_PROGRAM_MAL_H".into()),
        Directive::define_empty("MAL_PROGRAM_MAL_H"),
    ] {
        output.push(directive);
    }
    output.blank_line();
    output.push(Directive::IncludeSystem("stdint.h".into()));
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
    output.push(Directive::If(PreprocessorExpr::binary(
        "||",
        PreprocessorExpr::defined("__clang__"),
        PreprocessorExpr::defined("__GNUC__"),
    )));
    output.push(Directive::define_attribute(
        "MAL_DETAIL_MAYBE_UNUSED",
        "unused",
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

fn begin_section(output: &mut TranslationUnit, title: &str) {
    output.blank_line();
    output.push(Comment::new(title));
    output.blank_line();
}

fn emit_external_declaration(output: &mut TranslationUnit, signature: &HostSignature<'_>) {
    output.push(Declaration::function(external_signature(signature, false)));
}

fn emit_definition_macro(output: &mut TranslationUnit, signature: &HostSignature<'_>) {
    output.push(Directive::define_expr(
        format!("MAL_HAS_EXTERN_{}", signature.operation_name),
        Expr::number("1"),
    ));
    output.push(Directive::function_signature_define(
        format!("MAL_DEFINE_{}", signature.operation_name),
        signature.parameter_names(),
        external_signature(signature, true),
    ));
    output.blank_line();
}

fn macro_invocation(signature: &HostSignature<'_>) -> MacroInvocation {
    MacroInvocation::new(
        format!("MAL_DEFINE_{}", signature.operation_name),
        signature
            .parameter_names()
            .into_iter()
            .map(Expr::identifier),
    )
}

fn append_function(output: &mut TranslationUnit, signature: FunctionSignature, result: Expr) {
    output.push(FunctionDefinition::from_signature(
        signature,
        Block::new([Statement::return_value(result)]),
    ));
    output.blank_line();
}

fn external_signature(signature: &HostSignature<'_>, definition: bool) -> FunctionSignature {
    FunctionSignature::new(
        signature.result_type.clone(),
        format!("mal_ext_{}", signature.operation_name),
        if definition {
            signature.definition_parameters()
        } else {
            signature.parameters()
        },
    )
}
