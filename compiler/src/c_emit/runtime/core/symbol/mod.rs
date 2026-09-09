use super::*;

mod lifetime;
mod materialization;

pub(super) fn append_symbol_lifetime(output: &mut TranslationUnit) {
    lifetime::append_symbol_lifetime(output);
}

pub(super) fn append_symbol_copy(output: &mut TranslationUnit) {
    materialization::append_symbol_copy(output);
}

pub(super) fn append_symbol_materialization(output: &mut TranslationUnit) {
    materialization::append_symbol_materialization(output);
}
