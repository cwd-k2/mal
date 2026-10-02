use crate::backend::c::syntax::{
    Directive, Expr, MacroInvocation, Statement, TranslationUnit, c_block, c_directive, c_expr,
    c_macro_invocation, c_parameter, c_signature, c_statement, c_type,
};
use crate::backend::source_layout::SourceLayouts;
use crate::core::ast::TypeAlias;
use mal_frontend::check::ast::Type;

use super::super::{HostTypes, RepresentationId, TypeRegistry, is_bool};
use super::append_function;

mod alias;
mod scalar;
mod template;

impl TypeRegistry {
    pub(in crate::backend::c) fn memory_helpers(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
        layouts: SourceLayouts,
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        if !aliases.iter().any(|alias| alias.host_memory_access) {
            return output;
        }
        for ty in &self.aggregates {
            if host.memory_contains(ty) && !is_bool(ty) {
                let guard = format!("MAL_DETAIL_MEMORY_REPR_{}_HELPERS", self.index(ty));
                output.push(c_directive!(ifndef #{ guard.clone() }));
                output.push(c_directive!(define #{ guard };));
                match ty {
                    Type::Product(elements) => self.append_product_memory_template(
                        &mut output,
                        self.index(ty),
                        ty,
                        elements,
                        layouts,
                    ),
                    Type::Sum(members) => self.append_sum_memory_template(
                        &mut output,
                        self.index(ty),
                        ty,
                        members,
                        layouts,
                    ),
                    _ => unreachable!("only aggregate types have representation identities"),
                }
                output.push(c_directive!(endif));
                output.blank_line();
            }
        }
        for alias in aliases.iter().filter(|alias| alias.host_memory_access) {
            self.append_alias_memory_helpers(&mut output, alias, layouts);
        }
        output
    }

    fn memory_read_value(&self, ty: &Type, call: Expr, source: Expr) -> Expr {
        match ty {
            Type::Unit => c_expr!(compound((named("mal_Unit_t")), [positional((number(0)))])),
            Type::Product(_) | Type::Sum(_) if !is_bool(ty) => c_expr! {
                call(
                    #{ format!("mal_detail_memory_read_{}", self.index(ty)) },
                    [#{ call }, #{ source }]
                )
            },
            _ => c_expr! {
                call(
                    #{ format!("mal_detail_memory_read_{}", scalar_name(ty)) },
                    [#{ call }, #{ source }]
                )
            },
        }
    }

    fn memory_write_statement(
        &self,
        ty: &Type,
        call: Expr,
        destination: Expr,
        value: Expr,
    ) -> Statement {
        if matches!(ty, Type::Unit) {
            return c_statement!(cast((named("void")), #{ value }););
        }
        let name = match ty {
            Type::Product(_) | Type::Sum(_) if !is_bool(ty) => self.index(ty).to_string(),
            _ => scalar_name(ty).into(),
        };
        c_statement! {
            call(#{ format!("mal_detail_memory_write_{name}") }, [
                #{ call },
                #{ destination },
                #{ value },
            ]);
        }
    }
}

fn scalar_types() -> Vec<Type> {
    vec![
        Type::Sum(vec![Type::Unit, Type::Unit].into()),
        Type::Int8,
        Type::Int16,
        Type::Int32,
        Type::Int64,
        Type::UInt8,
        Type::UInt16,
        Type::UInt32,
        Type::UInt64,
        Type::Float32,
        Type::Float64,
        Type::Address,
        Type::ByteSize,
        Type::USize,
    ]
}

fn scalar_name(ty: &Type) -> &'static str {
    if is_bool(ty) {
        return "Bool";
    }
    match ty {
        Type::Int8 => "Int8",
        Type::Int16 => "Int16",
        Type::Int32 => "Int32",
        Type::Int64 => "Int64",
        Type::UInt8 => "UInt8",
        Type::UInt16 => "UInt16",
        Type::UInt32 => "UInt32",
        Type::UInt64 => "UInt64",
        Type::Float32 => "Float32",
        Type::Float64 => "Float64",
        Type::Address => "Address",
        Type::ByteSize => "ByteSize",
        Type::USize => "USize",
        _ => unreachable!("only canonical scalar types have scalar memory helpers"),
    }
}

fn integer_type(bits: usize) -> Type {
    match bits {
        8 => Type::UInt8,
        16 => Type::UInt16,
        32 => Type::UInt32,
        64 => Type::UInt64,
        _ => unreachable!("canonical sum tag widths are fixed-width integers"),
    }
}

fn offset(base: Expr, offset: impl Into<Offset>) -> Expr {
    c_expr!(add(#{ base }, #{ offset.into().0 }))
}

struct Offset(Expr);

impl From<usize> for Offset {
    fn from(value: usize) -> Self {
        Self(c_expr!(number(#{ value })))
    }
}

impl From<Expr> for Offset {
    fn from(value: Expr) -> Self {
        Self(value)
    }
}
