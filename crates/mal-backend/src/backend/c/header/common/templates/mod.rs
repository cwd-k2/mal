//! Program-independent aggregate declaration and bridge-conversion templates.

use crate::backend::c::syntax::{Comment, TranslationUnit};

mod aggregate;
mod sum;

use self::{aggregate::append_aggregate_templates, sum::append_sum_conversion_template};

pub(super) fn append_generated_header_templates(output: &mut TranslationUnit) {
    output.push(Comment::new("Generated header templates"));
    output.blank_line();
    append_aggregate_templates(output);
    append_sum_conversion_template(output);
}
