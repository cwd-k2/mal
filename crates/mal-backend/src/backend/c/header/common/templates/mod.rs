//! Program-independent aggregate conversion and public sum API templates.

use crate::backend::c::syntax::{Comment, Directive, TranslationUnit, c_function};

mod aggregate;
mod sum;

use self::{
    aggregate::append_aggregate_templates,
    sum::{append_sum_api_templates, append_sum_conversion_template},
};

pub(super) fn append_generated_header_templates(output: &mut TranslationUnit) {
    output.push(Comment::new("Generated header templates"));
    output.blank_line();
    append_aggregate_templates(output);
    let conversion = c_function! {
        #[static] #[inline] fn function_name(
            #[maybe_unused] call: *mut mal_call_t,
            value: value_type,
        ) -> result_type {
            return conversion;
        }
    };
    output.push(Directive::function_definitions_define(
        "MAL_DETAIL_DEFINE_CONVERSION",
        ["function_name", "result_type", "value_type", "conversion"],
        [conversion],
    ));
    append_sum_conversion_template(output);
    append_sum_api_templates(output);
}
