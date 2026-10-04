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
                    let member = c_expr!({ format!("field_{field}") });
                    if matches!(element, Type::Unit) {
                        return c_invocation!(unit({ member }));
                    }
                    let helper = match element {
                        Type::Product(_) | Type::Sum(_) if !is_bool(element) => {
                            self.index(element).to_string()
                        }
                        _ => scalar_name(element).into(),
                    };
                    c_invocation!(value(
                        { member },
                        { format!("mal_detail_memory_read_{helper}") },
                        { format!("mal_detail_memory_write_{helper}") },
                        { layout.offset },
                    ))
                });
        output.push(Directive::invocations_define(
            descriptor.clone(),
            ["unit", "value"],
            invocations,
        ));
        output.push(c_invocation!(MAL_DETAIL_DEFINE_MEMORY_PRODUCT(
            { format!("mal_detail_memory_read_{index}") },
            { format!("mal_detail_memory_write_{index}") },
            { self.host_value_c_type(ty, None).to_string() },
            { descriptor },
        )));
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
            c_invocation!(member(
                { value_type.clone() },
                { format!("mal_{tag_name}_t") },
                { format!("mal_detail_memory_write_{tag_name}") },
                { variant },
                { format!("variant_{variant}") },
                { format!("mal_detail_memory_read_{helper}") },
                { format!("mal_detail_memory_write_{helper}") },
                { layout.payload_offset },
            ))
        });
        output.push(Directive::invocations_define(
            descriptor.clone(),
            ["member"],
            invocations,
        ));
        output.push(c_invocation!(MAL_DETAIL_DEFINE_MEMORY_SUM(
            { format!("mal_detail_memory_read_{index}") },
            { format!("mal_detail_memory_write_{index}") },
            { value_type },
            { format!("mal_detail_memory_read_{tag_name}") },
            { descriptor },
        )));
    }

    pub(in crate::backend::c) fn common_scalar_memory_helpers(&self) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for ty in scalar_types() {
            self.append_scalar_memory_helpers(&mut output, &ty);
        }
        output
    }
}
