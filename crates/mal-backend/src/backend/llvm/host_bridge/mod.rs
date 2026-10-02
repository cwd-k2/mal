//! Shared marshalling plan and generated C bridge between public host carriers and internal LLVM values.

mod plan;
mod read;
mod write;

use std::collections::HashMap;

use super::body;
use crate::backend::abi::Function as AbiFunction;
use crate::backend::c::syntax::{
    Expr, Statement, TranslationUnit, TypeName, c_expr, c_function, c_initializer, c_statement,
    c_switch_case, c_type,
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
        plan::Kind::Unit => (
            vec![c_statement!(cast((named("void")), (id("mal_argument")));)],
            Vec::new(),
        ),
        plan::Kind::Product(fields) => {
            let arguments = fields
                .iter()
                .map(|field| {
                    marshalling.read(
                        &field.value,
                        identifier("mal_argument"),
                        field.offset,
                        context_cast(),
                    )
                })
                .collect::<Option<Vec<_>>>()?;
            (Vec::new(), arguments)
        }
        _ => (
            Vec::new(),
            vec![marshalling.read(&parameter, identifier("mal_argument"), 0, context_cast())?],
        ),
    };
    let mut call_arguments = vec![context_cast()];
    call_arguments.extend(arguments);
    let call = c_expr! {
        call(
            #{ format!("mal_ext_{}", external.name) },
            [...#{ call_arguments }]
        )
    };
    match &result.kind {
        plan::Kind::Unit => {
            statements.push(c_statement!(#{ call };));
            statements.push(store(
                "uint8_t",
                identifier("mal_result"),
                c_expr!(call("UINT8_C", [number(0)])),
            ));
        }
        plan::Kind::Product(_) | plan::Kind::Sum { .. } | plan::Kind::External => {
            statements.push(c_statement! {
                let "result": #{ raw_types.c_type(&external.result) } = #{ call };
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
    marshalling.helpers.push(c_function! {
        signature #{ bridge.c_signature() } {
            ...#{ statements }
        }
    });
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
        c_type!(ptr(const(named("uint8_t"))))
    } else {
        c_type!(ptr(named("uint8_t")))
    };
    let pointer = c_expr!(cast(#{ ty }, #{ base }));
    if offset == 0 {
        pointer
    } else {
        c_expr!(add(#{ pointer }, (number(#{ offset }))))
    }
}

fn context_cast() -> Expr {
    c_expr! {
        cast(
            #{ c_type!(ptr(named("MalContext"))) },
            (id("mal_context"))
        )
    }
}

fn load(ty: TypeName, pointer: Expr) -> Expr {
    c_expr!(dereference((cast(#{ ty }, #{ pointer }))))
}

fn store(ty: impl Into<TypeName>, pointer: Expr, value: Expr) -> Statement {
    c_statement! {
        assign(
            (dereference((cast(#{ c_type!(#{ ty.into().pointer() }) }, #{ pointer })))),
            #{ value }
        );
    }
}

fn identifier(name: impl Into<crate::backend::c::syntax::Identifier>) -> Expr {
    c_expr!(id(#{ name }))
}

fn trap(context: Expr, message: &str) -> Statement {
    c_statement!(call("mal_trap", [#{ context }, string(#{ message })]);)
}

fn c_scalar_type(ty: &Type) -> Option<&'static str> {
    if body::types::is_bool(ty) {
        return Some("MalType_Bool");
    }
    match ty {
        Type::Int8 => Some("MalType_Int8"),
        Type::Int16 => Some("MalType_Int16"),
        Type::Int32 => Some("MalType_Int32"),
        Type::Int64 => Some("MalType_Int64"),
        Type::UInt8 => Some("MalType_UInt8"),
        Type::UInt16 => Some("MalType_UInt16"),
        Type::UInt32 => Some("MalType_UInt32"),
        Type::UInt64 => Some("MalType_UInt64"),
        Type::Float32 => Some("MalType_Float32"),
        Type::Float64 => Some("MalType_Float64"),
        Type::Address => Some("MalType_Address"),
        Type::ByteSize => Some("MalType_ByteSize"),
        Type::USize => Some("MalType_USize"),
        _ => None,
    }
}
