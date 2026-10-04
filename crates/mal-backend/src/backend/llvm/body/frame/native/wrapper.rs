//! The wrapper that keeps the function's own name and calls the native worker with its context.

use super::*;
use crate::backend::llvm::syntax::emit_instruction;
use crate::backend::llvm::syntax::emit_terminator;

impl FunctionEmitter<'_> {
    /// Emits the stable function ABI once and moves recursive execution into a worker whose ABI
    /// carries only the changing parameter leaves. The context remains live in this wrapper's
    /// activation and also supplies the runtime pointers needed by a deep frames fallback.
    pub(in crate::backend::llvm::body) fn emit_native_wrapper(
        mut self,
    ) -> Option<super::super::super::EmittedFunction> {
        let plan = self.native_scalar_parameter()?.clone();
        let parameter = self.types.value(&self.function.parameter.ty)?;
        let result = self.types.value(&self.result_type)?;
        let parameters = llvm_parameters! {
            "%mal_context" : ptr,
            "%mal_control_top" : ptr,
            "%mal_environment" : ptr,
            "%mal_parameter" : { parameter.llvm.clone() },
        };
        self.begin_function(llvm_signature! {
            #[linkage(internal)]
            fn { super::super::super::function_name(self.function.id) }(
                ..{ parameters },
            ) -> { result.llvm.clone() }
        });
        self.block("entry");
        let context_type = self.native_context_type()?;
        let context_alignment = self.types.pointer_alignment().max(parameter.alignment);
        emit_instruction! {
            self;
            let "%mal_native_context_storage" = alloca {
                ty: { context_type.clone() },
                alignment: { context_alignment },
            };
        };
        let mut context = "poison".to_string();
        for (index, (ty, value)) in [
            (llvm_type!(ptr), "%mal_context"),
            (llvm_type!(ptr), "%mal_control_top"),
            (llvm_type!(ptr), "%mal_environment"),
            (parameter.llvm.clone(), "%mal_parameter"),
        ]
        .into_iter()
        .enumerate()
        {
            let inserted = self.register();
            emit_instruction! {
                self;
                let { inserted.clone() } = insert_value {
                    aggregate: ({ context_type.clone() }, { context }),
                    element: ({ ty }, { value }),
                    indices: [{ index }],
                };
            };
            context = inserted;
        }
        emit_instruction! {
            self;
            store {
                value: ({ context_type }, { context }),
                pointer: "%mal_native_context_storage",
                alignment: { context_alignment },
                metadata: [],
            };
        };
        let mut arguments = vec![(llvm_type!(ptr), "%mal_native_context_storage".into())];
        for leaf in &plan.varying {
            let register = self.register();
            emit_instruction! {
                self;
                let { register.clone() } = extract_value {
                    aggregate: ({ parameter.llvm.clone() }, "%mal_parameter"),
                    indices: { leaf.path.clone() },
                };
            };
            arguments.push((self.types.value(&leaf.ty)?.llvm, register));
        }
        let returned = self.register();
        let arguments = crate::backend::llvm::syntax::TypedValue::from_pairs(arguments)?;
        emit_instruction! {
            self;
            let { returned.clone() } = call {
                tail: false,
                result_type: { result.llvm.clone() },
                callee: direct({ self.native_worker_name() }),
                arguments: [..{ arguments }],
            };
        };
        emit_terminator! {
            self;
            return ({ result.llvm }, { returned });
        };
        self.finish_function()?;
        (!self.emission_failed).then_some(super::super::super::EmittedFunction {
            globals: self.globals,
            definitions: self.definitions,
        })
    }
}
