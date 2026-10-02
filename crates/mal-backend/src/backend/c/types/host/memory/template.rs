//! Canonical-memory templates for products and sums.

use super::*;

impl TypeRegistry {
    pub(super) fn append_product_memory_template(
        &self,
        output: &mut TranslationUnit,
        index: RepresentationId,
        ty: &Type,
        elements: &[Type],
        layouts: SourceLayouts,
    ) {
        let fields = layouts
            .product_fields(ty)
            .expect("checker-approved memory product has a layout");
        let descriptor = format!("MAL_DETAIL_MEMORY_FIELDS_{index}");
        let invocations =
            elements
                .iter()
                .zip(fields)
                .enumerate()
                .map(|(field, (element, layout))| {
                    let member = c_expr!(id(#{ format!("field_{field}") }));
                    if matches!(element, Type::Unit) {
                        return MacroInvocation::new("unit", [member]);
                    }
                    let helper = match element {
                        Type::Product(_) | Type::Sum(_) if !is_bool(element) => {
                            self.index(element).to_string()
                        }
                        _ => scalar_name(element).into(),
                    };
                    MacroInvocation::new(
                        "value",
                        [
                            member,
                            c_expr!(id(#{ format!("mal_detail_memory_read_{helper}") })),
                            c_expr!(id(#{ format!("mal_detail_memory_write_{helper}") })),
                            c_expr!(number(#{ layout.offset })),
                        ],
                    )
                });
        output.push(Directive::invocations_define(
            descriptor.clone(),
            ["unit", "value"],
            invocations,
        ));
        output.push(c_macro_invocation! {
            "MAL_DETAIL_DEFINE_MEMORY_PRODUCT"([
                id(#{ format!("mal_detail_memory_read_{index}") }),
                id(#{ format!("mal_detail_memory_write_{index}") }),
                id(#{ self.host_value_c_type(ty, None).to_string() }),
                id(#{ descriptor }),
            ])
        });
    }

    pub(super) fn append_sum_memory_template(
        &self,
        output: &mut TranslationUnit,
        index: RepresentationId,
        ty: &Type,
        members: &[Type],
        layouts: SourceLayouts,
    ) {
        let layout = layouts
            .sum(ty)
            .expect("checker-approved memory sum has a layout");
        let tag_type = integer_type(layout.tag_bits);
        let tag_name = scalar_name(&tag_type);
        let value_type = self.host_value_c_type(ty, None).to_string();
        let descriptor = format!("MAL_DETAIL_MEMORY_MEMBERS_{index}");
        let invocations = members.iter().enumerate().map(|(variant, member)| {
            let helper = match member {
                Type::Unit => "Unit".into(),
                Type::Product(_) | Type::Sum(_) if !is_bool(member) => {
                    self.index(member).to_string()
                }
                _ => scalar_name(member).into(),
            };
            MacroInvocation::new(
                "member",
                [
                    c_expr!(id(#{ value_type.clone() })),
                    c_expr!(id(#{ format!("mal_{tag_name}_t") })),
                    c_expr!(id(#{ format!("mal_detail_memory_write_{tag_name}") })),
                    c_expr!(number(#{ variant })),
                    c_expr!(id(#{ format!("variant_{variant}") })),
                    c_expr!(id(#{ format!("mal_detail_memory_read_{helper}") })),
                    c_expr!(id(#{ format!("mal_detail_memory_write_{helper}") })),
                    c_expr!(number(#{ layout.payload_offset })),
                ],
            )
        });
        output.push(Directive::invocations_define(
            descriptor.clone(),
            ["member"],
            invocations,
        ));
        output.push(c_macro_invocation! {
            "MAL_DETAIL_DEFINE_MEMORY_SUM"([
                id(#{ format!("mal_detail_memory_read_{index}") }),
                id(#{ format!("mal_detail_memory_write_{index}") }),
                id(#{ value_type }),
                id(#{ format!("mal_detail_memory_read_{tag_name}") }),
                id(#{ descriptor }),
            ])
        });
    }

    pub(in crate::backend::c) fn common_scalar_memory_helpers(&self) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for ty in scalar_types() {
            self.append_scalar_memory_helpers(&mut output, &ty);
        }
        output
    }
}
