use crate::core::ast::ProgramInterface;

use super::{
    HostTypes, TypeRegistry,
    host_signature::{CompilerSignature, ExternalSignatures},
};
use crate::c_emit::syntax::{
    Block, Comment, Declaration, Directive, Expr, FunctionDefinition, FunctionSignature,
    MacroInvocation, Statement, TranslationUnit,
};

mod prefix;

use self::prefix::emit_prefix;

pub(super) fn emit(interface: &ProgramInterface, types: &TypeRegistry, host: &HostTypes) -> String {
    let signatures: Vec<_> = interface
        .externals
        .iter()
        .map(|external| ExternalSignatures::new(external, types))
        .collect();
    let mut output = emit_prefix();
    let mut declarations = types.header_declarations(host);
    declarations.extend(types.header_alias_declarations(host, &interface.type_aliases));
    if !declarations.is_empty() {
        begin_section(&mut output, "Host-visible types");
        output.extend(declarations);
    }

    let mut helpers = types.header_opaque_helpers(host);
    helpers.extend(types.header_lifetime_helpers(host, &interface.type_aliases));
    helpers.extend(types.header_alias_helpers(host, &interface.type_aliases));
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
        for signatures in &signatures {
            emit_definition_macro(&mut output, &signatures.compiler);
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
        let signature = &signatures.compiler;
        output.blank_line();
        let mut body = Block::default();
        for name in signature.parameter_names().into_iter().skip(1) {
            body.push(Statement::expression(Expr::cast(
                "void",
                Expr::identifier(name),
            )));
        }
        body.push(Statement::call(
            "mal_trap",
            [
                Expr::identifier("context"),
                Expr::string(format!(
                    "external operation `{}` is not implemented",
                    signatures.host_body.operation_name
                )),
            ],
        ));
        output.push(FunctionDefinition::from_macro(
            macro_invocation(signature),
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

fn emit_definition_macro(output: &mut TranslationUnit, signature: &CompilerSignature<'_>) {
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

fn macro_invocation(signature: &CompilerSignature<'_>) -> MacroInvocation {
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
