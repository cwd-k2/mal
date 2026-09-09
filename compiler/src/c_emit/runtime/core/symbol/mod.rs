use super::*;

mod admission;
mod lifetime;
mod materialization;

pub(super) fn append_symbol_admission(output: &mut TranslationUnit) {
    admission::append_symbol_admission(output);
}

pub(super) fn append_host_symbol_lifetime(output: &mut TranslationUnit) {
    lifetime::append_host_symbol_lifetime(output);
}

pub(super) fn append_symbol_lifetime(output: &mut TranslationUnit) {
    lifetime::append_symbol_lifetime(output);
}

pub(super) fn append_symbol_copy(output: &mut TranslationUnit) {
    materialization::append_symbol_copy(output);
}

pub(super) fn append_symbol_materialization(output: &mut TranslationUnit) {
    materialization::append_symbol_materialization(output);
}
