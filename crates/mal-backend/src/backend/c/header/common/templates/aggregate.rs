//! Aggregate representation templates.

use crate::backend::c::syntax::{
    Directive, TranslationUnit, c_invocation, c_record, c_record_fields,
};

pub(super) fn append_aggregate_templates(output: &mut TranslationUnit) {
    output.push(Directive::record_fields_define(
        "MAL_DETAIL_REPR_FIELD",
        ["context", "index", "member", "type"],
        c_record_fields! { member: type },
    ));

    let descriptor_field = || c_invocation!(fields(field, type_tag));
    output.push(Directive::record_define(
        "MAL_DETAIL_DEFINE_PRODUCT_REPR",
        ["type_tag", "fields", "field"],
        c_record! {
            struct type_tag {
                { descriptor_field() },
            }
        },
    ));
    let members = c_invocation!(members(member, type_tag));
    output.push(Directive::record_define(
        "MAL_DETAIL_DEFINE_SUM_REPR",
        ["type_tag", "members", "member"],
        c_record! {
            struct type_tag {
                tag: uint32_t,
                payload: union {
                    { members },
                },
            }
        },
    ));
    output.push(Directive::record_define(
        "MAL_DETAIL_DEFINE_EMPTY_SUM_REPR",
        ["type_tag"],
        c_record! {
            struct type_tag {
                tag: uint32_t,
            }
        },
    ));
    output.blank_line();
}
