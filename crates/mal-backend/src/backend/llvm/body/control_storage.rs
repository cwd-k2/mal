use crate::backend::llvm::syntax::{BinaryOperator, ComparisonKind, ComparisonPredicate};

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
        emit_instruction!(
            self;
            call { Some(storage.clone()) },
            false,
            (ptr),
            direct "mal_control_storage";
            [
                (typed (ptr) => "%mal_context"),
            ]
        );
        emit_instruction!(
            self;
            store (ptr),
            { storage },
            "%mal_local_control_storage",
            { self.types.pointer_alignment() },
            []
        );
        let capacity = self.register();
        emit_instruction!(
            self;
            call { Some(capacity.clone()) },
            false,
            { self.types.index_llvm_type() },
            direct "mal_control_capacity";
            [
                (typed (ptr) => "%mal_context"),
            ]
        );
        emit_instruction!(
            self;
            store { self.types.index_llvm_type() },
            { capacity },
            "%mal_local_control_capacity",
            { self.types.index_alignment() },
            []
        );
    }

    pub(super) fn current_control_storage(&mut self) -> String {
        let storage = self.register();
        if self.local_control_storage {
            emit_instruction!(
                self;
                load { storage.clone() },
                (ptr),
                "%mal_local_control_storage",
                { self.types.pointer_alignment() },
                []
            );
        } else {
            emit_instruction!(
                self;
                call { Some(storage.clone()) },
                false,
                (ptr),
                direct "mal_control_storage";
                [
                    (typed (ptr) => "%mal_context"),
                ]
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
        emit_instruction!(
            self;
            load { top.clone() },
            { self.types.index_llvm_type() },
            { self.control_top_pointer() },
            { self.types.index_alignment() },
            []
        );
        let next_top = self.register();
        emit_instruction!(
            self;
            binary { next_top.clone() },
            { BinaryOperator::Add },
            { self.types.index_llvm_type() },
            { top.clone() },
            { frame_size.to_string() }
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
            emit_instruction!(
                self;
                call { Some(storage.clone()) },
                false,
                (ptr),
                direct "mal_control_reserve_frame";
                [
                    (typed (ptr) => "%mal_context"),
                    (typed { self.types.index_llvm_type() } => { top.clone() }),
                    (typed { self.types.index_llvm_type() } => { frame_size.to_string() }),
                ]
            );
            return Some(ControlReservation {
                top,
                next_top,
                storage,
            });
        }

        let cached_storage = self.current_control_storage();
        let capacity = self.register();
        emit_instruction!(
            self;
            load { capacity.clone() },
            { self.types.index_llvm_type() },
            "%mal_local_control_capacity",
            { self.types.index_alignment() },
            []
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
        emit_instruction!(
            self;
            compare { no_overflow.clone() },
            { ComparisonKind::Integer },
            { ComparisonPredicate::Ule },
            { self.types.index_llvm_type() },
            { top.clone() },
            { maximum_top.to_string() }
        );
        let within_capacity = self.register();
        emit_instruction!(
            self;
            compare { within_capacity.clone() },
            { ComparisonKind::Integer },
            { ComparisonPredicate::Ule },
            { self.types.index_llvm_type() },
            { next_top.clone() },
            { capacity }
        );
        let fast = self.register();
        emit_instruction!(
            self;
            binary { fast.clone() },
            { BinaryOperator::And },
            (int(1_u16)),
            { no_overflow },
            { within_capacity }
        );
        let label = self.label_id();
        emit_terminator!(self; conditional
            { fast } =>
            { format!("mal_control_fast_{label}") },
            { format!("mal_control_slow_{label}") }
        );
        self.block(format!("mal_control_fast_{label}"));
        emit_terminator!(self; branch { format!("mal_control_ready_{label}") });
        self.block(format!("mal_control_slow_{label}"));
        let grown = self.register();
        emit_instruction!(
            self;
            call { Some(grown.clone()) },
            false,
            (ptr),
            direct "mal_control_reserve_frame";
            [
                (typed (ptr) => "%mal_context"),
                (typed { self.types.index_llvm_type() } => { top.clone() }),
                (typed { self.types.index_llvm_type() } => { frame_size.to_string() }),
            ]
        );
        emit_instruction!(
            self;
            store (ptr),
            { grown.as_str() },
            "%mal_local_control_storage",
            { self.types.pointer_alignment() },
            []
        );
        let grown_capacity = self.register();
        emit_instruction!(
            self;
            call { Some(grown_capacity.clone()) },
            false,
            { self.types.index_llvm_type() },
            direct "mal_control_capacity";
            [
                (typed (ptr) => "%mal_context"),
            ]
        );
        emit_instruction!(
            self;
            store { self.types.index_llvm_type() },
            { grown_capacity },
            "%mal_local_control_capacity",
            { self.types.index_alignment() },
            []
        );
        emit_terminator!(self; branch { format!("mal_control_ready_{label}") });
        self.block(format!("mal_control_ready_{label}"));
        let storage = self.register();
        emit_instruction!(
            self;
            phi { storage.clone() },
            (ptr),
            {{ [
                (cached_storage, format!("mal_control_fast_{label}")),
                (grown, format!("mal_control_slow_{label}")),
            ] }}
        );
        Some(ControlReservation {
            top,
            next_top,
            storage,
        })
    }
}
