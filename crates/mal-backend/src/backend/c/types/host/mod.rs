use crate::backend::c::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, TranslationUnit, c_block, c_expr,
    c_invocation, c_items, c_signature,
};
use mal_frontend::check::ast::Type;

use super::{HostTypes, RepresentationId, TypeRegistry, is_bool};

mod declaration;
mod lifecycle;

impl TypeRegistry {
    pub(in crate::backend::c) fn host_value_helpers(&self, host: &HostTypes) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for name in &host.opaque_names {
            output.extend(c_items! {
                #[static] #[inline] fn { format!("mal_detail_to_host_{name}") }(
                    #[maybe_unused] call: *mut mal_call_t,
                    value: { format!("MalType_{name}") },
                ) -> { format!("mal_{name}_t") };
                #[static] #[inline] fn { format!("mal_detail_to_raw_{name}") }(
                    #[maybe_unused] call: *mut mal_call_t,
                    value: { format!("mal_{name}_t") },
                ) -> { format!("MalType_{name}") };
            });
        }
        if !host.opaque_names.is_empty() {
            output.blank_line();
        }
        for ty in &self.aggregates {
            if !host.external_contains(ty) || is_bool(ty) {
                continue;
            }
            let guard = format!("MAL_DETAIL_HOST_REPR_{}_HELPERS", self.index(ty));
            let mut guarded = c_items! { define!({ guard.clone() }); };
            match ty {
                Type::Product(_) => {
                    let id = self.index(ty);
                    guarded.push(c_invocation!(MAL_DETAIL_DEFINE_PRODUCT_CONVERSIONS(
                        { format!("mal_detail_to_host_{id}") },
                        { format!("mal_detail_to_raw_{id}") },
                        { format!("MalRepr_Product_{id}") },
                        { format!("mal_repr_product_{id}_t") },
                        { format!("MAL_DETAIL_REPR_FIELDS_{id}") },
                    )));
                }
                Type::Sum(_) => {
                    let id = self.index(ty);
                    guarded.extend(self.host_sum_conversion_helpers(id, ty));
                }
                _ => unreachable!("only aggregates have structural host helpers"),
            }
            output.extend(c_items! {
                if !defined({ guard }) {
                    ..{ guarded }
                }
            });
            output.blank_line();
        }
        for name in &host.opaque_names {
            let host_type = format!("mal_{name}_t");
            append_function(
                &mut output,
                c_signature! {
                    #[static] #[inline] fn { format!("mal_detail_to_host_{name}") }(
                        #[maybe_unused] call: *mut mal_call_t,
                        value: { format!("MalType_{name}") },
                    ) -> { host_type.clone() }
                },
                c_block! { return { host_type.clone() } { mal_detail_bits: value.bits }; },
            );
            append_function(
                &mut output,
                c_signature! {
                    #[static] #[inline] #[overloadable] fn mal_detail_from_bits(
                        #[maybe_unused] type_marker: *mut { host_type.clone() },
                        bits: uintptr_t,
                    ) -> { host_type.clone() }
                },
                c_block! { return { host_type.clone() } { mal_detail_bits: bits }; },
            );
            append_function(
                &mut output,
                c_signature! {
                    #[static] #[inline] #[overloadable] fn mal_detail_bits(
                        value: { host_type.clone() },
                    ) -> uintptr_t
                },
                c_block! { return value.mal_detail_bits; },
            );
            append_function(
                &mut output,
                c_signature! {
                    #[static] #[inline] fn { format!("mal_detail_to_raw_{name}") }(
                        #[maybe_unused] call: *mut mal_call_t,
                        value: { host_type },
                    ) -> { format!("MalType_{name}") }
                },
                c_block! {
                    return { format!("MalType_{name}") } { bits: value.mal_detail_bits };
                },
            );
        }
        output
    }

    fn host_sum_conversion_helpers(&self, index: RepresentationId, ty: &Type) -> TranslationUnit {
        let Type::Sum(_) = ty else {
            unreachable!("sum conversion requires a sum type")
        };
        let mut output = TranslationUnit::default();
        output.push(c_invocation!(MAL_DETAIL_DEFINE_SUM_CONVERSIONS(
            { format!("mal_detail_to_host_{index}") },
            { format!("mal_detail_to_raw_{index}") },
            { self.c_type(ty).to_string() },
            { self.host_value_c_type(ty, None).to_string() },
            { format!("MAL_DETAIL_REPR_FIELDS_{index}") },
        )));
        output.blank_line();
        output
    }

    pub(in crate::backend::c) fn host_to_raw_value(
        &self,
        ty: &Type,
        call: Expr,
        value: Expr,
    ) -> Expr {
        match ty {
            Type::Product(_) => c_expr! {
                { format!("mal_detail_to_raw_{}", self.index(ty)) }({ call }, { value })
            },
            Type::Sum(_) if !is_bool(ty) => c_expr! {
                { format!("mal_detail_to_raw_{}", self.index(ty)) }({ call }, { value })
            },
            Type::External { .. } => c_expr! {
                { self.c_type(ty) } { bits: { value }.mal_detail_bits }
            },
            Type::Unit => c_expr!(MalType_Unit { _0: 0 }),
            Type::Symbol | Type::Buffer(_) => value,
            Type::Function { .. } => {
                unreachable!("type checking excludes functions from extern signatures")
            }
            _ => value,
        }
    }

    pub(in crate::backend::c) fn raw_to_host_value(
        &self,
        ty: &Type,
        alias: Option<&str>,
        call: Expr,
        value: Expr,
    ) -> Expr {
        match ty {
            Type::Product(_) => c_expr! {
                { format!("mal_detail_to_host_{}", self.index(ty)) }({ call }, { value })
            },
            Type::Sum(_) if !is_bool(ty) => c_expr! {
                { format!("mal_detail_to_host_{}", self.index(ty)) }({ call }, { value })
            },
            Type::External { .. } => c_expr! {
                { self.host_value_c_type(ty, alias) } { mal_detail_bits: { value }.bits }
            },
            Type::Symbol | Type::Buffer(_) => value,
            Type::Function { .. } => {
                unreachable!("type checking excludes functions from extern signatures")
            }
            _ => value,
        }
    }
}

fn append_function(output: &mut TranslationUnit, signature: FunctionSignature, body: Block) {
    output.push(FunctionDefinition::from_signature(signature, body));
    output.blank_line();
}
