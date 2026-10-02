//! The function signature and the entry block before the first state: control storage, slots, the active
//! environment, and the parameter handoff.

use super::*;
use crate::backend::llvm::syntax::{llvm_function_attributes, llvm_parameters, llvm_signature};

impl FunctionEmitter<'_> {
    /// Starts the definition in this emission mode and opens the entry block.
    pub(super) fn begin_signature(&mut self) -> Option<()> {
        let parameters =
            if self.mode == EmissionMode::Native && self.native_worker_parameters().is_some() {
                self.native_worker_parameters()?
            } else if self.function.parameter.ty == Type::Unit {
                llvm_parameters! {
                    "%mal_context" : ptr,
                    "%mal_control_top" : ptr,
                    "%mal_environment" : ptr,
                }
            } else {
                let parameter = self.types.value(&self.function.parameter.ty)?;
                llvm_parameters! {
                    "%mal_context" : ptr,
                    "%mal_control_top" : ptr,
                    "%mal_environment" : ptr,
                    "%mal_parameter" : #{ parameter.llvm },
                }
            };
        let result = self.types.value(&self.result_type)?;
        let suffix = match self.mode {
            EmissionMode::Frames => "_frames",
            EmissionMode::Native if self.native_worker_parameters().is_some() => "_native",
            EmissionMode::Standard | EmissionMode::Native => "",
        };
        // The native version must stay small on its hot path, so the frames version is never inlined into it.
        let attributes = if self.mode == EmissionMode::Frames {
            llvm_function_attributes!(noinline)
        } else {
            Vec::new()
        };
        self.begin_function(llvm_signature! {
            #[linkage(internal)]
            #[attributes(...#{ attributes })]
            fn #{ format!("{}{suffix}", function_name(self.function.id)) }(
                ...#{ parameters },
            ) -> #{ result.llvm }
        });
        self.block("entry");
        Some(())
    }

    /// Loads the control base and, when this function keeps control state locally, allocates it.
    pub(super) fn emit_control_prologue(&mut self) {
        if self.frame_sites.is_empty() {
            return;
        }
        emit_instruction! {
            self;
            let "%mal_control_base" = load {
                ty: #{ self.types.index_llvm_type() },
                pointer: "%mal_control_top",
                alignment: #{ self.types.index_alignment() },
                metadata: [],
            };
        };
        if self.local_control_top {
            emit_instruction! {
                self;
                let "%mal_local_control_top" = alloca {
                    ty: #{ self.types.index_llvm_type() },
                    alignment: #{ self.types.index_alignment() },
                };
            };
            emit_instruction! {
                self;
                store {
                    value: typed(#{ self.types.index_llvm_type() }, "%mal_control_base"),
                    pointer: "%mal_local_control_top",
                    alignment: #{ self.types.index_alignment() },
                    metadata: [],
                };
            };
        }
        if self.local_control_storage {
            emit_instruction! {
                self;
                let "%mal_local_control_storage" = alloca {
                    ty: (ptr),
                    alignment: #{ self.types.pointer_alignment() },
                };
            };
            emit_instruction! {
                self;
                let "%mal_local_control_capacity" = alloca {
                    ty: #{ self.types.index_llvm_type() },
                    alignment: #{ self.types.index_alignment() },
                };
            };
            self.refresh_control_storage();
        }
    }

    /// Allocates every slot in index order and starts managed slots vacant.
    pub(super) fn emit_slot_allocas(&mut self) -> Option<()> {
        let mut slots = self.slots.values().cloned().collect::<Vec<_>>();
        slots.sort_by_key(|slot| slot.index);
        for slot in slots {
            let value_type = self.types.value(&slot.ty)?;
            emit_instruction! {
                self;
                let #{ format!("%mal_slot_{}", slot.index) } = alloca {
                    ty: #{ value_type.llvm.clone() },
                    alignment: #{ value_type.alignment },
                };
            };
            if crate::execution::ownership::is_managed(&slot.ty) {
                self.vacate_slot(&slot)?;
            }
        }
        Some(())
    }

    /// In a common control region, the environment of the function running now, retained for the region.
    pub(super) fn emit_active_environment(&mut self) {
        if self.common_region.is_none() {
            return;
        }
        emit_instruction! {
            self;
            let "%mal_active_environment" = alloca {
                ty: (ptr),
                alignment: #{ self.types.pointer_alignment() },
            };
        };
        let environment = self.register();
        emit_instruction! {
            self;
            let #{ environment.clone() } = call {
                tail: false,
                result_type: (ptr),
                callee: direct("mal_runtime_environment_retain"),
                arguments: [typed((ptr), "%mal_context"), typed((ptr), "%mal_environment")],
            };
        };
        emit_instruction! {
            self;
            store {
                value: typed((ptr), #{ environment }),
                pointer: "%mal_active_environment",
                alignment: #{ self.types.pointer_alignment() },
                metadata: [],
            };
        };
    }

    /// Hands the parameter to its destination when it is bound or managed.
    pub(super) fn emit_entry_parameter(&mut self) -> Option<()> {
        let parameter_destination = self.execution.parameters.destination(self.function.id)?;
        if matches!(parameter_destination, ParameterDestination::Bind(_))
            || crate::execution::ownership::is_managed(&self.function.parameter.ty)
        {
            let entry = self.ownership.native_entry(self.function.id);
            let parameter = EmittedValue {
                ty: self.function.parameter.ty.clone(),
                representation: if self.function.parameter.ty == Type::Unit {
                    "0".into()
                } else {
                    "%mal_parameter".into()
                },
                owned: entry == crate::execution::ownership::ParameterEntry::OwnedAbi,
            };
            self.emit_parameter_handoff(self.function.id, &parameter, entry)?;
        }
        Some(())
    }
}
