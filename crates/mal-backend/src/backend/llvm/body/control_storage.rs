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
        emit_instruction! {
            self;
            let #{ storage.clone() } = call {
                tail: false,
                result_type: (ptr),
                callee: direct("mal_control_storage"),
                arguments: [typed((ptr), "%mal_context")],
            };
        };
        emit_instruction! {
            self;
            store {
                value: typed((ptr), #{ storage }),
                pointer: "%mal_local_control_storage",
                alignment: #{ self.types.pointer_alignment() },
                metadata: [],
            };
        };
        let capacity = self.register();
        emit_instruction! {
            self;
            let #{ capacity.clone() } = call {
                tail: false,
                result_type: #{ self.types.index_llvm_type() },
                callee: direct("mal_control_capacity"),
                arguments: [typed((ptr), "%mal_context")],
            };
        };
        emit_instruction! {
            self;
            store {
                value: typed(#{ self.types.index_llvm_type() }, #{ capacity }),
                pointer: "%mal_local_control_capacity",
                alignment: #{ self.types.index_alignment() },
                metadata: [],
            };
        };
    }

    pub(super) fn current_control_storage(&mut self) -> String {
        let storage = self.register();
        if self.local_control_storage {
            emit_instruction! {
                self;
                let #{ storage.clone() } = load {
                    ty: (ptr),
                    pointer: "%mal_local_control_storage",
                    alignment: #{ self.types.pointer_alignment() },
                    metadata: [],
                };
            };
        } else {
            emit_instruction! {
                self;
                let #{ storage.clone() } = call {
                    tail: false,
                    result_type: (ptr),
                    callee: direct("mal_control_storage"),
                    arguments: [typed((ptr), "%mal_context")],
                };
            };
        }
        storage
    }

    pub(super) fn reserve_control_frame(
        &mut self,
        frame_size: usize,
        replacement: bool,
    ) -> Option<ControlReservation> {
        let top = self.register();
        emit_instruction! {
            self;
            let #{ top.clone() } = load {
                ty: #{ self.types.index_llvm_type() },
                pointer: #{ self.control_top_pointer() },
                alignment: #{ self.types.index_alignment() },
                metadata: [],
            };
        };
        let next_top = self.register();
        emit_instruction! {
            self;
            let #{ next_top.clone() } = binary {
                operator: #{ BinaryOperator::Add },
                ty: #{ self.types.index_llvm_type() },
                left: #{ top.clone() },
                right: #{ frame_size.to_string() },
            };
        };
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
            emit_instruction! {
                self;
                let #{ storage.clone() } = call {
                    tail: false,
                    result_type: (ptr),
                    callee: direct("mal_control_reserve_frame"),
                    arguments: [
                        typed((ptr), "%mal_context"),
                        typed(#{ self.types.index_llvm_type() }, #{ top.clone() }),
                        typed(#{ self.types.index_llvm_type() }, #{ frame_size.to_string() }),
                    ],
                };
            };
            return Some(ControlReservation {
                top,
                next_top,
                storage,
            });
        }

        let cached_storage = self.current_control_storage();
        let capacity = self.register();
        emit_instruction! {
            self;
            let #{ capacity.clone() } = load {
                ty: #{ self.types.index_llvm_type() },
                pointer: "%mal_local_control_capacity",
                alignment: #{ self.types.index_alignment() },
                metadata: [],
            };
        };
        let no_overflow = self.register();
        let frame_size = u64::try_from(frame_size).ok()?;
        let maximum_top = match self.types.index_size() {
            1 => u64::from(u8::MAX).checked_sub(frame_size)?,
            2 => u64::from(u16::MAX).checked_sub(frame_size)?,
            4 => u64::from(u32::MAX).checked_sub(frame_size)?,
            8 => u64::MAX.checked_sub(frame_size)?,
            _ => unreachable!("target layout admits only supported index widths"),
        };
        emit_instruction! {
            self;
            let #{ no_overflow.clone() } = compare {
                kind: #{ ComparisonKind::Integer },
                predicate: #{ ComparisonPredicate::Ule },
                ty: #{ self.types.index_llvm_type() },
                left: #{ top.clone() },
                right: #{ maximum_top.to_string() },
            };
        };
        let within_capacity = self.register();
        emit_instruction! {
            self;
            let #{ within_capacity.clone() } = compare {
                kind: #{ ComparisonKind::Integer },
                predicate: #{ ComparisonPredicate::Ule },
                ty: #{ self.types.index_llvm_type() },
                left: #{ next_top.clone() },
                right: #{ capacity },
            };
        };
        let fast = self.register();
        emit_instruction! {
            self;
            let #{ fast.clone() } = binary {
                operator: #{ BinaryOperator::And },
                ty: (int(1_u16)),
                left: #{ no_overflow },
                right: #{ within_capacity },
            };
        };
        let label = self.label_id();
        emit_terminator! {
            self;
            branch {
                condition: #{ fast },
                then: #{ format!("mal_control_fast_{label}") },
                otherwise: #{ format!("mal_control_slow_{label}") },
            };
        };
        self.block(format!("mal_control_fast_{label}"));
        emit_terminator! {
            self;
            branch {
                target: #{ format!("mal_control_ready_{label}") },
            };
        };
        self.block(format!("mal_control_slow_{label}"));
        let grown = self.register();
        emit_instruction! {
            self;
            let #{ grown.clone() } = call {
                tail: false,
                result_type: (ptr),
                callee: direct("mal_control_reserve_frame"),
                arguments: [
                    typed((ptr), "%mal_context"),
                    typed(#{ self.types.index_llvm_type() }, #{ top.clone() }),
                    typed(#{ self.types.index_llvm_type() }, #{ frame_size.to_string() }),
                ],
            };
        };
        emit_instruction! {
            self;
            store {
                value: typed((ptr), #{ grown.as_str() }),
                pointer: "%mal_local_control_storage",
                alignment: #{ self.types.pointer_alignment() },
                metadata: [],
            };
        };
        let grown_capacity = self.register();
        emit_instruction! {
            self;
            let #{ grown_capacity.clone() } = call {
                tail: false,
                result_type: #{ self.types.index_llvm_type() },
                callee: direct("mal_control_capacity"),
                arguments: [typed((ptr), "%mal_context")],
            };
        };
        emit_instruction! {
            self;
            store {
                value: typed(#{ self.types.index_llvm_type() }, #{ grown_capacity }),
                pointer: "%mal_local_control_capacity",
                alignment: #{ self.types.index_alignment() },
                metadata: [],
            };
        };
        emit_terminator! {
            self;
            branch {
                target: #{ format!("mal_control_ready_{label}") },
            };
        };
        self.block(format!("mal_control_ready_{label}"));
        let storage = self.register();
        emit_instruction! {
            self;
            let #{ storage.clone() } = phi {
                ty: (ptr),
                incoming: #{ [
                    (cached_storage, format!("mal_control_fast_{label}")),
                    (grown, format!("mal_control_slow_{label}")),
                ] },
            };
        };
        Some(ControlReservation {
            top,
            next_top,
            storage,
        })
    }
}
