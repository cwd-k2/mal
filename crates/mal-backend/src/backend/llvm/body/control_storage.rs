use crate::backend::llvm::syntax::llvm_type;

use super::FunctionEmitter;

pub(super) struct ControlReservation {
    pub(super) top: String,
    pub(super) next_top: String,
    pub(super) storage: String,
}

impl FunctionEmitter<'_> {
    pub(super) fn refresh_control_storage(&mut self) {
        if !self.local_control_storage {
            return;
        }
        let storage = self.register();
        self.direct_call(
            Some(storage.clone()),
            false,
            llvm_type!(ptr),
            "mal_control_storage",
            [(llvm_type!(ptr), "%mal_context".into())],
        );
        self.store(
            llvm_type!(ptr),
            storage,
            "%mal_local_control_storage",
            self.types.pointer_alignment(),
            [],
        );
        let capacity = self.register();
        self.direct_call(
            Some(capacity.clone()),
            false,
            self.types.index_llvm_type(),
            "mal_control_capacity",
            [(llvm_type!(ptr), "%mal_context".into())],
        );
        self.store(
            self.types.index_llvm_type(),
            capacity,
            "%mal_local_control_capacity",
            self.types.index_alignment(),
            [],
        );
    }

    pub(super) fn current_control_storage(&mut self) -> String {
        let storage = self.register();
        if self.local_control_storage {
            self.load(
                storage.clone(),
                llvm_type!(ptr),
                "%mal_local_control_storage",
                self.types.pointer_alignment(),
                [],
            );
        } else {
            self.direct_call(
                Some(storage.clone()),
                false,
                llvm_type!(ptr),
                "mal_control_storage",
                [(llvm_type!(ptr), "%mal_context".into())],
            );
        }
        storage
    }

    pub(super) fn reserve_control_frame(
        &mut self,
        frame_size: usize,
        replacement: bool,
    ) -> Option<ControlReservation> {
        let top = self.register();
        self.load(
            top.clone(),
            self.types.index_llvm_type(),
            self.control_top_pointer(),
            self.types.index_alignment(),
            [],
        );
        let next_top = self.register();
        self.binary(
            next_top.clone(),
            crate::backend::llvm::syntax::BinaryOperator::Add,
            self.types.index_llvm_type(),
            top.clone(),
            frame_size.to_string(),
        );
        if replacement {
            let storage = self.current_control_storage();
            return Some(ControlReservation {
                top,
                next_top,
                storage,
            });
        }
        if !self.local_control_storage {
            let storage = self.register();
            self.direct_call(
                Some(storage.clone()),
                false,
                llvm_type!(ptr),
                "mal_control_reserve_frame",
                [
                    (llvm_type!(ptr), "%mal_context".into()),
                    (self.types.index_llvm_type(), top.clone()),
                    (self.types.index_llvm_type(), frame_size.to_string()),
                ],
            );
            return Some(ControlReservation {
                top,
                next_top,
                storage,
            });
        }

        let cached_storage = self.current_control_storage();
        let capacity = self.register();
        self.load(
            capacity.clone(),
            self.types.index_llvm_type(),
            "%mal_local_control_capacity",
            self.types.index_alignment(),
            [],
        );
        let no_overflow = self.register();
        let frame_size = u64::try_from(frame_size).ok()?;
        let maximum_top = match self.types.index_size() {
            1 => u64::from(u8::MAX).checked_sub(frame_size)?,
            2 => u64::from(u16::MAX).checked_sub(frame_size)?,
            4 => u64::from(u32::MAX).checked_sub(frame_size)?,
            8 => u64::MAX.checked_sub(frame_size)?,
            _ => unreachable!("target layout admits only supported index widths"),
        };
        self.compare(
            no_overflow.clone(),
            crate::backend::llvm::syntax::ComparisonKind::Integer,
            crate::backend::llvm::syntax::ComparisonPredicate::Ule,
            self.types.index_llvm_type(),
            top.clone(),
            maximum_top.to_string(),
        );
        let within_capacity = self.register();
        self.compare(
            within_capacity.clone(),
            crate::backend::llvm::syntax::ComparisonKind::Integer,
            crate::backend::llvm::syntax::ComparisonPredicate::Ule,
            self.types.index_llvm_type(),
            next_top.clone(),
            capacity,
        );
        let fast = self.register();
        self.binary(
            fast.clone(),
            crate::backend::llvm::syntax::BinaryOperator::And,
            llvm_type!(int(1_u16)),
            no_overflow,
            within_capacity,
        );
        let label = self.label_id();
        self.conditional_branch(
            fast,
            format!("mal_control_fast_{label}"),
            format!("mal_control_slow_{label}"),
        );
        self.block(format!("mal_control_fast_{label}"));
        self.branch(format!("mal_control_ready_{label}"));
        self.block(format!("mal_control_slow_{label}"));
        let grown = self.register();
        self.direct_call(
            Some(grown.clone()),
            false,
            llvm_type!(ptr),
            "mal_control_reserve_frame",
            [
                (llvm_type!(ptr), "%mal_context".into()),
                (self.types.index_llvm_type(), top.clone()),
                (self.types.index_llvm_type(), frame_size.to_string()),
            ],
        );
        self.store(
            llvm_type!(ptr),
            grown.as_str(),
            "%mal_local_control_storage",
            self.types.pointer_alignment(),
            [],
        );
        let grown_capacity = self.register();
        self.direct_call(
            Some(grown_capacity.clone()),
            false,
            self.types.index_llvm_type(),
            "mal_control_capacity",
            [(llvm_type!(ptr), "%mal_context".into())],
        );
        self.store(
            self.types.index_llvm_type(),
            grown_capacity,
            "%mal_local_control_capacity",
            self.types.index_alignment(),
            [],
        );
        self.branch(format!("mal_control_ready_{label}"));
        self.block(format!("mal_control_ready_{label}"));
        let storage = self.register();
        self.phi(
            storage.clone(),
            llvm_type!(ptr),
            [
                (cached_storage, format!("mal_control_fast_{label}")),
                (grown, format!("mal_control_slow_{label}")),
            ],
        );
        Some(ControlReservation {
            top,
            next_top,
            storage,
        })
    }
}
