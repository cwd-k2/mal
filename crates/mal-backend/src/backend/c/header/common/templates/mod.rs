//! Program-independent aggregate conversion, canonical-memory, and public sum API templates.

use crate::backend::c::syntax::{Comment, Directive, TranslationUnit, c_function};

mod aggregate;
mod memory;
mod sum;

use self::{
    aggregate::append_aggregate_templates,
    memory::{append_product_memory_template, append_sum_memory_template},
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
    let memory_read = c_function! {
        #[static] #[inline] fn read_name(
            call: *mut mal_call_t,
            address: mal_Address_t,
            index: mal_USize_t,
        ) -> value_type {
            mal_Address_return(call, address);
            return reader(call, (address as *const uint8_t) + index * stride);
        }
    };
    let memory_write = c_function! {
        #[static] #[inline] fn write_name(
            call: *mut mal_call_t,
            address: mal_Address_t,
            index: mal_USize_t,
            value: value_type,
        ) -> void {
            mal_Address_return(call, address);
            writer(call, (address as *mut uint8_t) + index * stride, value);
        }
    };
    output.push(Directive::function_definitions_define(
        "MAL_DETAIL_DEFINE_MEMORY_ALIAS",
        [
            "read_name",
            "write_name",
            "value_type",
            "stride",
            "reader",
            "writer",
        ],
        [memory_read, memory_write],
    ));
    append_product_memory_template(output);
    append_sum_memory_template(output);
    append_sum_conversion_template(output);
    append_sum_api_templates(output);
}
