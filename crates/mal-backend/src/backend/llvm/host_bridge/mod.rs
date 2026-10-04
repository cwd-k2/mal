//! Shared marshalling plan and generated C bridge between public host carriers and internal LLVM values.

mod plan;
mod read;
mod write;

use std::collections::HashMap;

use super::body;
use crate::backend::abi::Function as AbiFunction;
use crate::backend::c::syntax::{
    Expr, FunctionDefinition, Initializer, Statement, TranslationUnit, TypeName, c_block, c_expr,
    c_signature, c_statement, c_switch_cases, c_type,
};
use mal_frontend::check::ast::{SharedTypeId, Type};

pub(super) struct Bridge {
    pub(super) llvm_declaration: super::syntax::FunctionDeclaration,
    pub(super) c_definitions: TranslationUnit,
}

pub(super) fn generate(
    external: &crate::core::ast::ExternalOperation,
    target: super::TargetLayout,
    raw_types: &crate::backend::c::RawHostTypes,
) -> Option<Bridge> {
    let types = body::types::Types::for_target(target);
    let parameter = plan::Value::new(&external.parameter, types.clone())?;
    let result = plan::Value::new(&external.result, types)?;
    let mut marshalling = Marshalling::new(external.id.0, raw_types);
    let bridge = AbiFunction::external_bridge(external.id);
    let llvm_declaration = bridge.llvm_declaration();
    let (mut statements, arguments) = match &parameter.kind {
        plan::Kind::Unit => (vec![c_statement!(mal_argument as void;)], Vec::new()),
        _ => (
            Vec::new(),
            vec![marshalling.read(&parameter, identifier("mal_argument"), 0, context_cast())?],
        ),
    };
    let mut call_arguments = vec![context_cast()];
    call_arguments.extend(arguments);
    let call = c_expr!({ format!("mal_ext_{}", external.name) }(
        ..{ call_arguments }
    ));
    match &result.kind {
        plan::Kind::Unit => {
            statements.push(c_statement!(({ call });));
            statements.push(store(
                "uint8_t",
                identifier("mal_result"),
                c_expr!(UINT8_C(0)),
            ));
        }
        plan::Kind::Product(_)
        | plan::Kind::Sum { .. }
        | plan::Kind::External
        | plan::Kind::Symbol { .. } => {
            statements.push(c_statement! {
                let result: { raw_types.c_type(&external.result) } = { call };
            });
            statements.extend(marshalling.write(
                &result,
                identifier("result"),
                0,
                context_cast(),
            )?);
        }
        plan::Kind::Scalar => {
            statements.push(store(
                c_scalar_type(result.ty)?,
                identifier("mal_result"),
                call,
            ));
        }
    }
    marshalling.helpers.blank_line();
    marshalling.helpers.push(FunctionDefinition::from_signature(
        bridge.c_signature(),
        c_block! { ..{ statements } },
    ));
    Some(Bridge {
        llvm_declaration,
        c_definitions: marshalling.helpers,
    })
}

struct Marshalling<'a> {
    external: u32,
    raw_types: &'a crate::backend::c::RawHostTypes,
    next_helper: usize,
    read_helpers: HashMap<SharedTypeId, String>,
    write_helpers: HashMap<SharedTypeId, String>,
    helpers: TranslationUnit,
}

impl<'a> Marshalling<'a> {
    fn new(external: u32, raw_types: &'a crate::backend::c::RawHostTypes) -> Self {
        Self {
            external,
            raw_types,
            next_helper: 0,
            read_helpers: HashMap::new(),
            write_helpers: HashMap::new(),
            helpers: TranslationUnit::default(),
        }
    }

    fn helper_name(&mut self, direction: &str) -> String {
        let helper = self.next_helper;
        self.next_helper += 1;
        format!(
            "mal_bridge_external_{}_{}_sum_{}",
            self.external, direction, helper
        )
    }
}

fn bridge_pointer(base: Expr, offset: usize, read_only: bool) -> Expr {
    let ty = if read_only {
        c_type!(*const uint8_t)
    } else {
        c_type!(*mut uint8_t)
    };
    let pointer = c_expr!({ base } as { ty });
    if offset == 0 {
        pointer
    } else {
        c_expr!({ pointer } + { offset })
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

fn trap(context: Expr, message: &str) -> Statement {
    let message = Expr::string(message);
    c_statement!(mal_trap({ context }, { message });)
}

fn c_scalar_type(ty: &Type) -> Option<&'static str> {
    if body::types::is_bool(ty) {
        return Some("mal_Bool_t");
    }
    match ty {
        Type::Int8 => Some("mal_Int8_t"),
        Type::Int16 => Some("mal_Int16_t"),
        Type::Int32 => Some("mal_Int32_t"),
        Type::Int64 => Some("mal_Int64_t"),
        Type::UInt8 => Some("mal_UInt8_t"),
        Type::UInt16 => Some("mal_UInt16_t"),
        Type::UInt32 => Some("mal_UInt32_t"),
        Type::UInt64 => Some("mal_UInt64_t"),
        Type::Float32 => Some("mal_Float32_t"),
        Type::Float64 => Some("mal_Float64_t"),
        Type::Buffer(_) => Some("mal_Buffer_t"),
        Type::ByteSize => Some("mal_ByteSize_t"),
        Type::USize => Some("mal_USize_t"),
        _ => None,
    }
}
