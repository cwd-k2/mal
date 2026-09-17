use crate::core::ast::ProgramInterface;

use super::{
    HostTypes, TypeRegistry,
    host_signature::{CompilerSignature, ExternalSignatures},
};
use crate::backend::c::syntax::{
    Block, Comment, Declaration, Directive, Expr, FunctionDefinition, FunctionSignature,
    Initializer, MacroInvocation, Statement, TranslationUnit,
};

mod prefix;

use self::prefix::emit_prefix;

pub(super) fn emit(
    interface: &ProgramInterface,
    types: &TypeRegistry,
    host: &HostTypes,
    index_bits: usize,
) -> String {
    let signatures: Vec<_> = interface
        .externals
        .iter()
        .map(|external| ExternalSignatures::new(external, types))
        .collect();
    let mut output = emit_prefix(index_bits);
    let mut declarations = types.header_declarations(host);
    declarations.extend(types.header_alias_declarations(host, &interface.type_aliases));
    declarations.extend(types.host_value_declarations(host, &interface.type_aliases));
    if !declarations.is_empty() {
        begin_section(&mut output, "Host-visible types");
        output.extend(declarations);
    }

    let helpers = types.host_value_helpers(host, &interface.type_aliases);
    if !helpers.is_empty() {
        begin_section(&mut output, "Type helpers");
        output.extend(helpers);
    }

    if !interface.externals.is_empty() {
        begin_section(&mut output, "External operations");
        for signatures in &signatures {
            emit_external_declaration(&mut output, &signatures.compiler);
        }
        begin_section(&mut output, "External definition helpers");
        for (external, signatures) in interface.externals.iter().zip(&signatures) {
            emit_definition_macro(&mut output, signatures, external, types);
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
    let mut output = TranslationUnit::new([Directive::include_quoted(header_name).into()]);
    for external in &interface.externals {
        let signatures = ExternalSignatures::new(external, types);
        let signature = &signatures.host_body;
        output.blank_line();
        let mut body = Block::default();
        for name in signature.parameter_names().into_iter().skip(1) {
            body.push(Statement::expression(Expr::cast(
                "void",
                Expr::identifier(name),
            )));
        }
        body.push(Statement::call(
            "mal_call_trap",
            [
                Expr::identifier("call"),
                Expr::string(format!(
                    "external operation `{}` is not implemented",
                    signatures.host_body.operation_name
                )),
            ],
        ));
        output.push(FunctionDefinition::from_macro(
            host_macro_invocation(signature),
            body,
        ));
    }
    output.render()
}

fn begin_section(output: &mut TranslationUnit, title: &str) {
    output.blank_line();
    output.push(Comment::new(title));
    output.blank_line();
}

fn emit_external_declaration(output: &mut TranslationUnit, signature: &CompilerSignature<'_>) {
    output.push(Declaration::function(external_signature(signature, false)));
}

fn emit_definition_macro(
    output: &mut TranslationUnit,
    signatures: &ExternalSignatures<'_>,
    external: &crate::core::ast::ExternalOperation,
    types: &TypeRegistry,
) {
    let signature = &signatures.compiler;
    output.push(Directive::define_expr(
        format!("MAL_HAS_EXTERN_{}", signature.operation_name),
        Expr::number("1"),
    ));
    output.push(Directive::function_items_define(
        format!("MAL_DEFINE_{}", signature.operation_name),
        signatures.host_body.parameter_names(),
        [signatures.host_body.signature()],
        [wrapper_definition(signatures, external, types)],
        signatures.host_body.signature(),
    ));
    output.blank_line();
}

fn wrapper_definition(
    signatures: &ExternalSignatures<'_>,
    external: &crate::core::ast::ExternalOperation,
    types: &TypeRegistry,
) -> FunctionDefinition {
    let mut body = Block::new([Statement::variable(
        "mal_call_t",
        "call",
        Some(Expr::compound_literal(
            "mal_call_t",
            [Initializer::designated(
                "mal_detail_context",
                Expr::identifier("context"),
            )],
        )),
    )]);
    let mut arguments = vec![Expr::address_of(Expr::identifier("call"))];
    match &external.parameter {
        crate::check::ast::Type::Unit => {}
        crate::check::ast::Type::Product(elements) => {
            let raw = Expr::compound_literal(
                types.c_type(&external.parameter),
                elements.iter().enumerate().map(|(field, _)| {
                    Initializer::designated(
                        format!("field_{field}"),
                        Expr::identifier(format!("argument_{field}")),
                    )
                }),
            );
            arguments.push(types.raw_to_host_value(
                &external.parameter,
                external.parameter_alias.as_deref(),
                Expr::address_of(Expr::identifier("call")),
                raw,
            ));
        }
        ty => arguments.push(types.raw_to_host_value(
            ty,
            external.parameter_alias.as_deref(),
            Expr::address_of(Expr::identifier("call")),
            Expr::identifier("value"),
        )),
    }
    let call = Expr::named_call(format!("mal_detail_{}", external.name), arguments);
    if external.result == crate::check::ast::Type::Unit {
        body.push(Statement::expression(call));
    } else {
        body.push(Statement::return_value(call));
    }
    FunctionDefinition::from_signature(external_signature(&signatures.compiler, true), body)
}

fn host_macro_invocation(
    signature: &super::host_signature::HostBodySignature<'_>,
) -> MacroInvocation {
    MacroInvocation::new(
        format!("MAL_DEFINE_{}", signature.operation_name),
        signature
            .parameter_names()
            .into_iter()
            .map(Expr::identifier),
    )
}

fn external_signature(signature: &CompilerSignature<'_>, definition: bool) -> FunctionSignature {
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
