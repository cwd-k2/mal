use crate::backend::c::syntax::{
    Block, Directive, Expr, FunctionDefinition, FunctionSignature, TranslationUnit, TypeName,
    c_block, c_expr, c_invocation, c_items, c_signature,
};
use crate::core::ast::TypeAlias;
use mal_frontend::check::ast::Type;

use super::{HostTypes, RepresentationId, TypeRegistry, is_bool};

mod declaration;

impl TypeRegistry {
    pub(in crate::backend::c) fn host_value_helpers(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for name in &host.opaque_names {
            output.extend(c_items! {
                #[static] #[inline] fn { format!("mal_detail_to_host_{name}") }(
                    #[maybe_unused] call: *mut mal_call_t,
                    value: { format!("MalType_{name}") },
                ) -> { format!("mal_{name}_t") };
                #[static] #[inline] fn { format!("mal_{name}_return") }(
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
                        { format!("mal_repr_product_{id}_return") },
                        { format!("MalRepr_Product_{id}") },
                        { format!("mal_repr_product_{id}_t") },
                        { format!("MAL_DETAIL_REPR_FIELDS_{id}") },
                    )));
                }
                Type::Sum(members) => {
                    let id = self.index(ty);
                    guarded.extend(self.host_sum_conversion_helpers(id, ty));
                    self.append_host_sum_helpers(
                        &mut guarded,
                        &format!("repr_sum_{id}"),
                        &format!("mal_repr_sum_{id}_t"),
                        self.c_type(ty),
                        ty,
                        &vec![None; members.len()],
                    );
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
                    #[static] #[inline] fn { format!("mal_{name}_from_bits") }(
                        bits: uintptr_t,
                    ) -> { host_type.clone() }
                },
                c_block! { return { host_type.clone() } { mal_detail_bits: bits }; },
            );
            append_function(
                &mut output,
                c_signature! {
                    #[static] #[inline] fn { format!("mal_detail_to_host_{name}") }(
                        #[maybe_unused] call: *mut mal_call_t,
                        value: { format!("MalType_{name}") },
                    ) -> { host_type.clone() }
                },
                c_block! { return { format!("mal_{name}_from_bits") }(value.bits); },
            );
            append_function(
                &mut output,
                c_signature! {
                    #[static] #[inline] fn { format!("mal_{name}_to_bits") }(
                        value: { host_type.clone() },
                    ) -> uintptr_t
                },
                c_block! { return value.mal_detail_bits; },
            );
            append_function(
                &mut output,
                c_signature! {
                    #[static] #[inline] fn { format!("mal_{name}_return") }(
                        #[maybe_unused] call: *mut mal_call_t,
                        value: { host_type },
                    ) -> { format!("MalType_{name}") }
                },
                c_block! {
                    return { format!("MalType_{name}") } { bits: value.mal_detail_bits };
                },
            );
        }
        for alias in aliases {
            if !host.exposes_external_alias(alias) {
                continue;
            }
            if matches!(&alias.ty, Type::Sum(_)) {
                self.append_host_sum_helpers(
                    &mut output,
                    &alias.name,
                    &format!("mal_{}_t", alias.name),
                    self.header_c_type(&alias.ty, Some(&alias.name)),
                    &alias.ty,
                    &alias.element_aliases,
                );
                continue;
            }
            if matches!(&alias.ty, Type::Product(_)) {
                output.push(c_invocation!(MAL_DETAIL_DEFINE_CONVERTING_RETURN(
                    { format!("mal_{}_return", alias.name) },
                    { format!("MalType_{}", alias.name) },
                    { format!("mal_{}_t", alias.name) },
                    { format!("mal_repr_product_{}_return", self.index(&alias.ty)) },
                )));
                continue;
            }
            let conversion = {
                let conversion = format!("MAL_DETAIL_TO_RAW_ALIAS_{}", alias.name);
                let value = self.host_to_raw_value(&alias.ty, c_expr!(call), c_expr!(value));
                output.extend(c_items! { define!({ conversion.clone() } = { value }); });
                conversion
            };
            output.push(c_invocation!(MAL_DETAIL_DEFINE_CONVERSION(
                { format!("mal_{}_return", alias.name) },
                { format!("MalType_{}", alias.name) },
                { format!("mal_{}_t", alias.name) },
                { conversion },
            )));
        }
        output
    }

    fn append_host_sum_helpers(
        &self,
        output: &mut TranslationUnit,
        public_name: &str,
        host_type: &str,
        raw_type: TypeName,
        ty: &Type,
        element_aliases: &[Option<String>],
    ) {
        let Type::Sum(members) = ty else {
            unreachable!("sum helpers require a sum type")
        };
        let mut variants = Vec::new();
        for (variant, member) in members.iter().enumerate() {
            let tag_name = format!("mal_{public_name}_tag_{variant}");
            let macro_name = tag_name.clone();
            output.extend(c_items! { define!({ macro_name } = UINT32_C({ variant })); });
            let make_name = format!("mal_{public_name}_make_{variant}");
            let return_name = format!("mal_{public_name}_return_{variant}");
            let member_name = format!("variant_{variant}");
            let to_raw = format!("mal_detail_to_raw_{}", self.index(ty));
            let invocation = if *member == Type::Unit {
                c_invocation!(unit(
                    { make_name },
                    { return_name },
                    { host_type },
                    { raw_type.to_string() },
                    { tag_name },
                    { member_name },
                    { to_raw },
                ))
            } else {
                c_invocation!(value(
                    { make_name },
                    { return_name },
                    { host_type },
                    { raw_type.to_string() },
                    {
                        self.host_value_c_type(member, element_aliases[variant].as_deref())
                            .to_string()
                    },
                    { tag_name },
                    { member_name },
                    { to_raw },
                ))
            };
            variants.push(invocation);
        }
        let descriptor = format!("MAL_DETAIL_SUM_API_{public_name}");
        output.push(Directive::invocations_define(
            descriptor.clone(),
            ["unit", "value"],
            variants,
        ));
        output.push(c_invocation!({ descriptor }(
            MAL_DETAIL_DEFINE_SUM_UNIT_API,
            MAL_DETAIL_DEFINE_SUM_VALUE_API,
        )));
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

    fn host_to_raw_value(&self, ty: &Type, call: Expr, value: Expr) -> Expr {
        match ty {
            Type::Product(_) => c_expr! {
                { format!("mal_repr_product_{}_return", self.index(ty)) }({ call }, { value })
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
