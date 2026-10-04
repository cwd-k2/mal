use crate::backend::c::syntax::{
    Declaration, FunctionSignature, FunctionSpecifier, Parameter, TranslationUnit, TypeName,
    c_expr, c_invocation, c_items,
};
use crate::core::ast::TypeAlias;
use mal_frontend::check::ast::Type;

use super::super::{HostTypes, TypeRegistry, is_bool};

impl TypeRegistry {
    pub(in crate::backend::c) fn host_value_declarations(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for name in &host.opaque_names {
            let alias = format!("mal_{name}_t");
            output.extend(c_items! {
                type { alias } = struct {
                    bits: uintptr_t,
                };
            });
        }
        for ty in &self.aggregates {
            if !host.contains(ty) || is_bool(ty) {
                continue;
            }
            let kind = match ty {
                Type::Product(_) => "product",
                Type::Sum(_) => "sum",
                Type::Function { .. } => continue,
                _ => unreachable!("only aggregate types have representation identities"),
            };
            let id = self.index(ty);
            let guard = format!("MAL_DETAIL_REPR_{id}_DECLARED");
            let alias = format!("mal_repr_{kind}_{id}_t");
            output.extend(c_items! {
                if !defined({ guard.clone() }) {
                    define!({ guard });
                    type { alias } = Struct<{ format!("mal_detail_repr_{kind}_{id}") }>;
                }
            });
        }
        for alias in aliases {
            if host.exposes_alias(alias) {
                let name = format!("mal_{}_t", alias.name);
                let source = self.host_value_c_type(&alias.ty, None);
                output.extend(c_items! { type { name } = { source }; });
            }
        }
        if !output.is_empty() {
            output.blank_line();
        }
        for ty in &self.aggregates {
            if !host.contains(ty) || is_bool(ty) {
                continue;
            }
            if matches!(ty, Type::Product(_) | Type::Sum(_)) {
                self.append_repr_descriptor(&mut output, ty);
            }
            let guard = format!("MAL_DETAIL_REPR_{}_DEFINED", self.index(ty));
            let mut guarded = c_items! { define!({ guard.clone() }); };
            match ty {
                Type::Product(_) => {
                    let tag = format!("mal_detail_repr_product_{}", self.index(ty));
                    guarded.push(c_invocation!(MAL_DETAIL_DEFINE_PRODUCT_REPR(
                        { tag },
                        { format!("MAL_DETAIL_REPR_FIELDS_{}", self.index(ty)) },
                        MAL_DETAIL_REPR_FIELD,
                    )));
                }
                Type::Sum(members) => {
                    let tag = format!("mal_detail_repr_sum_{}", self.index(ty));
                    let template = if members.is_empty() {
                        "MAL_DETAIL_DEFINE_EMPTY_SUM_REPR"
                    } else {
                        "MAL_DETAIL_DEFINE_SUM_REPR"
                    };
                    let mut arguments = vec![c_expr!({ tag })];
                    if !members.is_empty() {
                        arguments.push(c_expr!({
                            format!("MAL_DETAIL_REPR_FIELDS_{}", self.index(ty))
                        }));
                        arguments.push(c_expr!(MAL_DETAIL_REPR_FIELD));
                    }
                    guarded.push(c_invocation!({ template }(..{ arguments })));
                }
                Type::Function { .. } => continue,
                _ => unreachable!("only aggregates have representation identities"),
            }
            self.append_structural_type_selector(&mut guarded, ty);
            output.extend(c_items! {
                if !defined({ guard }) {
                    ..{ guarded }
                }
            });
            output.blank_line();
        }
        output
    }

    fn append_structural_type_selector(&self, output: &mut TranslationUnit, ty: &Type) {
        let (kind, elements) = match ty {
            Type::Product(elements) => ("product", elements.as_ref()),
            Type::Sum(elements) => ("sum", elements.as_ref()),
            _ => unreachable!("only products and sums have structural type selectors"),
        };
        let id = self.index(ty);
        let key = format!("mal_detail_{kind}_key_{id}_t");
        output.push(Declaration::function_pointer_type_alias(
            TypeName::named("void"),
            key.clone(),
            elements
                .iter()
                .map(|element| Parameter::unnamed(self.host_value_c_type(element, None))),
        ));
        output.push(
            FunctionSignature::new(
                self.host_value_c_type(ty, None).pointer(),
                format!("mal_detail_{kind}_type"),
                [Parameter::unnamed(TypeName::named(key))],
            )
            .with_specifiers([FunctionSpecifier::Overloadable]),
        );
    }
}
