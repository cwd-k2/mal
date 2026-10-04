//! The entry of a native activation: the worker context it unpacks, and the hand-off to the frames version when the native stack is used up.

use super::*;
use crate::backend::llvm::syntax::emit_instruction;
use crate::backend::llvm::syntax::emit_terminator;

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_native_context_entry(&mut self) -> Option<()> {
        let plan = self.native_scalar_parameter()?.clone();
        let context_type = self.native_context_type()?;
        for (index, name) in ["%mal_context", "%mal_control_top", "%mal_environment"]
            .into_iter()
            .enumerate()
        {
            let pointer = self.register();
            emit_instruction! {
                self;
                let { pointer.clone() } = get_element_ptr {
                    inbounds: false,
                    element_type: { context_type.clone() },
                    pointer: "%mal_native_context",
                    indices: [(int(32_u16), "0"), (int(32_u16), { index.to_string() })],
                };
            };
            emit_instruction! {
                self;
                let { name } = load {
                    ty: ptr,
                    pointer: { pointer },
                    alignment: { self.types.pointer_alignment() },
                    metadata: [],
                };
            };
        }
        let parameter = self.types.value(&self.function.parameter.ty)?;
        let pointer = self.register();
        emit_instruction! {
            self;
            let { pointer.clone() } = get_element_ptr {
                inbounds: false,
                element_type: { context_type },
                pointer: "%mal_native_context",
                indices: [(int(32_u16), "0"), (int(32_u16), "3")],
            };
        };
        let mut reconstructed = self.register();
        emit_instruction! {
            self;
            let { reconstructed.clone() } = load {
                ty: { parameter.llvm.clone() },
                pointer: { pointer },
                alignment: { parameter.alignment },
                metadata: [],
            };
        };
        for (index, leaf) in plan.varying.iter().enumerate() {
            let inserted = if index + 1 == plan.varying.len() {
                "%mal_parameter".into()
            } else {
                self.register()
            };
            emit_instruction! {
                self;
                let { inserted.clone() } = insert_value {
                    aggregate: ({ parameter.llvm.clone() }, { reconstructed }),
                    element: (
                        { self.types.value(&leaf.ty)?.llvm },
                        { format!("%mal_native_parameter_{index}") },
                    ),
                    indices: { leaf.path.clone() },
                };
            };
            reconstructed = inserted;
        }
        Some(())
    }

    /// Continues the activation in the frames version when the native stack is used up.
    pub(in crate::backend::llvm::body) fn emit_native_entry_guard(&mut self) -> Option<()> {
        let mut parameters = vec![
            (llvm_type!(ptr), "%mal_context".to_owned()),
            (llvm_type!(ptr), "%mal_control_top".to_owned()),
            (llvm_type!(ptr), "%mal_environment".to_owned()),
        ];
        if self.function.parameter.ty != Type::Unit {
            let value = self.types.value(&self.function.parameter.ty)?;
            parameters.push((value.llvm, "%mal_parameter".to_owned()));
        }
        let result_llvm = self.types.value(&self.result_type)?.llvm;
        let name = super::super::super::function_name(self.function.id);
        let stack = self.register();
        emit_instruction! {
            self;
            let { stack.clone() } = call {
                tail: false,
                result_type: ptr,
                callee: direct("llvm.stacksave"),
                arguments: [],
            };
        };
        let flag = self.register();
        emit_instruction! {
            self;
            let { flag.clone() } = call {
                tail: false,
                result_type: int(8_u16),
                callee: direct("mal_native_stack_is_deep"),
                arguments: [(ptr, "%mal_context"), (ptr, { stack })],
            };
        };
        let deep = self.register();
        emit_instruction! {
            self;
            let { deep.clone() } = compare {
                kind: { ComparisonKind::Integer },
                predicate: { ComparisonPredicate::Ne },
                ty: int(8_u16),
                left: { flag },
                right: "0",
            };
        };
        let expected = self.register();
        emit_instruction! {
            self;
            let { expected.clone() } = call {
                tail: false,
                result_type: int(1_u16),
                callee: direct("llvm.expect.i1"),
                arguments: [(int(1_u16), { deep }), (int(1_u16), "false")],
            };
        };
        emit_terminator! {
            self;
            branch {
                condition: { expected },
                then: "mal_deep_entry",
                otherwise: "mal_native_entry",
            };
        };
        self.block("mal_deep_entry");
        let continued = self.register();
        let parameters = crate::backend::llvm::syntax::TypedValue::from_pairs(parameters)?;
        emit_instruction! {
            self;
            let { continued.clone() } = call {
                tail: false,
                result_type: { result_llvm.clone() },
                callee: direct({ format!("{name}_frames") }),
                arguments: [..{ parameters }],
            };
        };
        emit_terminator! {
            self;
            return ({ result_llvm }, { continued });
        };
        self.block("mal_native_entry");
        Some(())
    }
}
