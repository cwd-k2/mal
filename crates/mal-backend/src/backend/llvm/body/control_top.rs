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
        emit_instruction! {
            self;
            let #{ top.clone() } = load {
                ty: #{ self.types.index_llvm_type() },
                pointer: "%mal_local_control_top",
                alignment: #{ self.types.index_alignment() },
                metadata: [],
            };
        };
        emit_instruction! {
            self;
            store {
                value: typed(#{ self.types.index_llvm_type() }, #{ top }),
                pointer: "%mal_control_top",
                alignment: #{ self.types.index_alignment() },
                metadata: [],
            };
        };
        Some(())
    }
}
