//! Program-independent aggregate declaration templates.

use crate::backend::c::syntax::{Comment, TranslationUnit};

mod aggregate;

use self::aggregate::append_aggregate_templates;

pub(super) fn append_generated_header_templates(output: &mut TranslationUnit) {
    output.push(Comment::new("Generated header templates"));
    output.blank_line();
    append_aggregate_templates(output);
    output.push(crate::backend::c::syntax::Directive::aggregate_lifecycle_templates());
}
