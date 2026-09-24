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
        let index_type = self.types.pointer_integer()?;
        let top = self.register();
        self.line(format!(
            "  {top} = load {index_type}, ptr %mal_local_control_top, align {}",
            self.types.index_alignment()
        ));
        self.line(format!(
            "  store {index_type} {top}, ptr %mal_control_top, align {}",
            self.types.index_alignment()
        ));
        Some(())
    }
}
