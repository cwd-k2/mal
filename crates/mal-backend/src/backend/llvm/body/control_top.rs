use super::FunctionEmitter;

impl FunctionEmitter<'_> {
    pub(super) fn control_top_pointer(&self) -> &'static str {
        if self.local_control_top {
            "%mal_local_control_top"
        } else {
            "%mal_control_top"
        }
    }

    pub(super) fn sync_control_top(&mut self) -> Option<()> {
        if !self.local_control_top {
            return Some(());
        }
        let top = self.register();
        self.load(
            top.clone(),
            self.types.index_llvm_type(),
            "%mal_local_control_top",
            self.types.index_alignment(),
            [],
        );
        self.store(
            self.types.index_llvm_type(),
            top,
            "%mal_control_top",
            self.types.index_alignment(),
            [],
        );
        Some(())
    }
}
