//! Program-independent aggregate conversion, canonical-memory, and public sum API templates.

use crate::backend::c::syntax::{TranslationUnit, c_comment, c_directive, c_function};

mod aggregate;
mod memory;
mod sum;

use self::{
    aggregate::append_aggregate_templates,
    memory::{append_product_memory_template, append_sum_memory_template},
    sum::{append_sum_api_templates, append_sum_conversion_template},
};

pub(super) fn append_generated_header_templates(output: &mut TranslationUnit) {
    output.push(c_comment!("Generated header templates"));
    output.blank_line();
    append_aggregate_templates(output);
    let conversion = c_function! {
        #[static] #[inline] fn "function_name"(
            #[maybe_unused] "call": ptr(named("mal_call_t")),
            "value": named("value_type"),
        ) -> named("result_type") {
            return (id("conversion"));
        }
    };
    output.push(c_directive! {
        define_functions "MAL_DETAIL_DEFINE_CONVERSION" {
            parameters: #{ ["function_name", "result_type", "value_type", "conversion"] },
            definitions: #{ [conversion] },
        }
    });
    let memory_read = c_function! {
        #[static] #[inline] fn "read_name"(
            "call": ptr(named("mal_call_t")),
            "address": named("mal_Address_t"),
            "index": named("mal_USize_t"),
        ) -> named("value_type") {
            call("mal_Address_return", [id("call"), id("address")]);
            return (call("reader", [
                id("call"),
                add(
                    (cast(
                        (ptr(const(named("uint8_t")))),
                        (id("address"))
                    )),
                    (multiply((id("index")), (id("stride"))))
                ),
            ]));
        }
    };
    let memory_write = c_function! {
        #[static] #[inline] fn "write_name"(
            "call": ptr(named("mal_call_t")),
            "address": named("mal_Address_t"),
            "index": named("mal_USize_t"),
            "value": named("value_type"),
        ) -> named("void") {
            call("mal_Address_return", [id("call"), id("address")]);
            call("writer", [
                id("call"),
                add(
                    (cast((ptr(named("uint8_t"))), (id("address")))),
                    (multiply((id("index")), (id("stride"))))
                ),
                id("value"),
            ]);
        }
    };
    output.push(c_directive! {
        define_functions "MAL_DETAIL_DEFINE_MEMORY_ALIAS" {
            parameters: #{ [
                "read_name", "write_name", "value_type", "stride", "reader", "writer",
            ] },
            definitions: #{ [memory_read, memory_write] },
        }
    });
    append_product_memory_template(output);
    append_sum_memory_template(output);
    append_sum_conversion_template(output);
    append_sum_api_templates(output);
}
