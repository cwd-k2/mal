//! Generated C bridge between public host carriers and internal pointer-based calls.

use crate::backend::abi::Function as AbiFunction;
use crate::backend::c::syntax::{
    Expr, FunctionDefinition, Statement, TranslationUnit, TypeName, c_block, c_expr, c_statement,
};
use mal_frontend::check::ast::Type;

pub(super) struct Bridge {
    pub(super) llvm_declaration: super::syntax::FunctionDeclaration,
    pub(super) c_definitions: TranslationUnit,
}

pub(super) fn generate(
    external: &crate::core::ast::ExternalOperation,
    raw_types: &crate::backend::c::RawHostTypes,
) -> Bridge {
    let bridge = AbiFunction::external_bridge(external.id);
    let llvm_declaration = bridge.llvm_declaration();
    let (mut statements, arguments) = match &external.parameter {
        Type::Unit => (vec![c_statement!(mal_argument as void;)], Vec::new()),
        parameter => {
            let ty = raw_types.c_type(parameter);
            (
                Vec::new(),
                vec![load(ty.const_pointee_pointer(), identifier("mal_argument"))],
            )
        }
    };
    let mut call_arguments = vec![context_cast()];
    call_arguments.extend(arguments);
    let call = c_expr!({ format!("mal_ext_{}", external.name) }(
        ..{ call_arguments }
    ));
    statements.push(store(
        raw_types.c_type(&external.result),
        identifier("mal_result"),
        call,
    ));
    let mut c_definitions = TranslationUnit::default();
    c_definitions.push(FunctionDefinition::from_signature(
        bridge.c_signature(),
        c_block! { ..{ statements } },
    ));
    Bridge {
        llvm_declaration,
        c_definitions,
    }
}

fn context_cast() -> Expr {
    c_expr!(mal_context as *mut MalContext)
}

fn load(ty: TypeName, pointer: Expr) -> Expr {
    c_expr!(*({ pointer } as { ty }))
}

fn store(ty: impl Into<TypeName>, pointer: Expr, value: Expr) -> Statement {
    let pointer_type = ty.into().pointer();
    c_statement!(*({ pointer } as { pointer_type }) = { value };)
}

fn identifier(name: impl Into<crate::backend::c::syntax::Identifier>) -> Expr {
    Expr::identifier(name)
}
