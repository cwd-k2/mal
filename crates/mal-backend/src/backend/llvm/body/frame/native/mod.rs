//! Native recursion for a self-recursive function.
//!
//! The function is emitted twice. The frames version keeps the suspended callers of a recursion in the control arena,
//! so it never nests native calls. The native version, which carries the function's own name, nests a native call
//! for each recursive call and hands the whole activation to the frames version at its entry once the native stack
//! is used up. A native activation keeps its live values in its own slots and touches no control arena, so the
//! optimizer sees an ordinary recursive function whose only extra work is one stack check per activation.

use crate::backend::llvm::syntax::{
    ComparisonKind, ComparisonPredicate, Parameter, Type as LlvmType, llvm_parameters,
    llvm_signature, llvm_type,
};
use crate::closure::ast::Atom;
use crate::control::ast::{StateId, Terminator};
use mal_frontend::check::ast::Type;

use super::super::{EmissionMode, EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    fn native_scalar_parameter(&self) -> Option<&crate::execution::NativeScalarParameter> {
        self.execution
            .native_recursion
            .scalar_parameter(self.function.id)
    }

    fn native_context_type(&self) -> Option<LlvmType> {
        let parameter = self.types.value(&self.function.parameter.ty)?;
        Some(LlvmType::structure([
            llvm_type!(ptr),
            llvm_type!(ptr),
            llvm_type!(ptr),
            parameter.llvm,
        ]))
    }

    fn native_worker_name(&self) -> String {
        let name = super::super::function_name(self.function.id);
        if self.native_scalar_parameter().is_some() {
            format!("{name}_native")
        } else {
            name
        }
    }

    pub(in crate::backend::llvm::body) fn native_worker_parameters(
        &self,
    ) -> Option<Vec<Parameter>> {
        let plan = self.native_scalar_parameter()?;
        let mut parameters = vec![Parameter::named(llvm_type!(ptr), "%mal_native_context")];
        for (index, leaf) in plan.varying.iter().enumerate() {
            parameters.push(Parameter::named(
                self.types.value(&leaf.ty)?.llvm,
                format!("%mal_native_parameter_{index}"),
            ));
        }
        Some(parameters)
    }

    /// Whether the function can also be emitted as a native version: every framed recursion edge targets the
    /// function itself, so the same body can use native calls until it reaches the stack budget.
    pub(in crate::backend::llvm::body) fn has_native_version(&self) -> bool {
        self.mode == EmissionMode::Standard
            && self.common_region.is_none()
            && !self.frame_sites.is_empty()
            && self
                .execution
                .native_recursion
                .has_native_version(self.function.id)
    }

    /// Turns a standard emitter into the frames version, which is emitted under another name.
    pub(in crate::backend::llvm::body) fn into_frames_version(mut self) -> Self {
        self.mode = EmissionMode::Frames;
        self
    }

    /// Turns a standard emitter into the native version, which no frame belongs to.
    pub(in crate::backend::llvm::body) fn into_native_version(mut self) -> Self {
        self.mode = EmissionMode::Native;
        self.frame_sites.clear();
        self.frame_tags.clear();
        self.local_control_top = false;
        self.local_control_storage = false;
        self
    }

    pub(in crate::backend::llvm::body) fn emit_native_self_call(
        &mut self,
        site: StateId,
        callee: &Atom,
        argument: &Atom,
    ) -> Option<()> {
        let Terminator::Call { resume, .. } = &self.control.states[site.0].terminator else {
            return None;
        };
        let resume = *resume;
        self.require_terminator_borrow(
            site,
            crate::execution::ownership::TerminatorOperand::CallCallee,
            callee,
        )?;
        let callee = self.atom(callee)?;
        let environment = self.closure_environment(&callee)?;
        // The callee's entry keeps its own reference to a managed argument, so a reference this call site takes for the
        // ownership plan's handoff (a share or a move) is released once the call returns.
        let mut handed_over = None;
        let scalar_parameter = self.native_scalar_parameter().cloned();
        let mut arguments = if scalar_parameter.is_some() {
            vec![(llvm_type!(ptr), "%mal_native_context".into())]
        } else {
            vec![
                (llvm_type!(ptr), "%mal_context".into()),
                (llvm_type!(ptr), "%mal_control_top".into()),
                (llvm_type!(ptr), environment),
            ]
        };
        if self.function.parameter.ty != Type::Unit {
            let argument = if crate::execution::ownership::is_managed(&argument.ty) {
                let effect = self.ownership.terminator_use(
                    site,
                    crate::execution::ownership::TerminatorOperand::CallArgument,
                )?;
                if effect == crate::execution::ownership::UseEffect::Borrow {
                    self.atom(argument)?
                } else {
                    let prepared = self.prepare_atom_for_use(argument, effect)?;
                    self.commit_consumes(&prepared)?;
                    handed_over = Some(prepared.value.clone());
                    prepared.value
                }
            } else {
                self.atom(argument)?
            };
            let argument_type = self.types.value(&argument.ty)?;
            if let Some(plan) = &scalar_parameter {
                for leaf in &plan.varying {
                    let register = self.register();
                    emit_instruction!(
                        self;
                        let {{ register.clone() }} = extract_value {
                            aggregate: typed({{ argument_type.llvm.clone() }}, {{ argument.representation.clone() }}),
                            indices: {{ leaf.path.clone() }},
                        };
                    );
                    arguments.push((self.types.value(&leaf.ty)?.llvm, register));
                }
            } else {
                arguments.push((argument_type.llvm, argument.representation));
            }
        }
        let result_type = self.current_result_type()?;
        let result_llvm = self.types.value(&result_type)?.llvm;
        let name = self.native_worker_name();
        let register = self.register();
        emit_instruction!(
            self;
            let {{ register.clone() }} = call {
                tail: false,
                result_type: {{ result_llvm }},
                callee: direct({{ name }}),
                arguments: pairs({{ arguments }}),
            };
        );
        if let Some(value) = handed_over {
            self.release_value(&value.ty, &value.representation)?;
        }
        let result = EmittedValue {
            owned: crate::execution::ownership::is_managed(&result_type),
            ty: result_type,
            representation: register,
        };
        self.store_input_pattern(resume, Some(&result))?;
        emit_terminator!(self; branch { format!("mal_state_{}", resume.0) });
        Some(())
    }
}

impl FunctionEmitter<'_> {
    /// Emits the stable function ABI once and moves recursive execution into a worker whose ABI
    /// carries only the changing parameter leaves. The context remains live in this wrapper's
    /// activation and also supplies the runtime pointers needed by a deep frames fallback.
    pub(in crate::backend::llvm::body) fn emit_native_wrapper(
        mut self,
    ) -> Option<super::super::EmittedFunction> {
        let plan = self.native_scalar_parameter()?.clone();
        let parameter = self.types.value(&self.function.parameter.ty)?;
        let result = self.types.value(&self.result_type)?;
        let parameters = llvm_parameters!(
            "%mal_context" : ptr,
            "%mal_control_top" : ptr,
            "%mal_environment" : ptr,
            "%mal_parameter" : {{ parameter.llvm.clone() }},
        );
        self.begin_function(llvm_signature!(
            #[linkage(internal)]
            fn {{ super::super::function_name(self.function.id) }}(
                ...{{ parameters }},
            ) -> {{ result.llvm.clone() }}
        ));
        self.block("entry");
        let context_type = self.native_context_type()?;
        let context_alignment = self.types.pointer_alignment().max(parameter.alignment);
        emit_instruction!(
            self;
            let "%mal_native_context_storage" = alloca {
                ty: {{ context_type.clone() }},
                alignment: {{ context_alignment }},
            };
        );
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
            emit_instruction!(
                self;
                let {{ inserted.clone() }} = insert_value {
                    aggregate: typed({{ context_type.clone() }}, {{ context }}),
                    element: typed({{ ty }}, {{ value }}),
                    indices: [{{ index }}],
                };
            );
            context = inserted;
        }
        emit_instruction!(
            self;
            store {
                value: typed({{ context_type }}, {{ context }}),
                pointer: "%mal_native_context_storage",
                alignment: {{ context_alignment }},
                metadata: [],
            };
        );
        let mut arguments = vec![(llvm_type!(ptr), "%mal_native_context_storage".into())];
        for leaf in &plan.varying {
            let register = self.register();
            emit_instruction!(
                self;
                let {{ register.clone() }} = extract_value {
                    aggregate: typed({{ parameter.llvm.clone() }}, "%mal_parameter"),
                    indices: {{ leaf.path.clone() }},
                };
            );
            arguments.push((self.types.value(&leaf.ty)?.llvm, register));
        }
        let returned = self.register();
        emit_instruction!(
            self;
            let {{ returned.clone() }} = call {
                tail: false,
                result_type: {{ result.llvm.clone() }},
                callee: direct({{ self.native_worker_name() }}),
                arguments: pairs({{ arguments }}),
            };
        );
        emit_terminator!(self; return { result.llvm } => { returned });
        self.finish_function()?;
        (!self.emission_failed).then_some(super::super::EmittedFunction {
            globals: self.globals,
            definitions: self.definitions,
        })
    }

    pub(in crate::backend::llvm::body) fn emit_native_context_entry(&mut self) -> Option<()> {
        let plan = self.native_scalar_parameter()?.clone();
        let context_type = self.native_context_type()?;
        for (index, name) in ["%mal_context", "%mal_control_top", "%mal_environment"]
            .into_iter()
            .enumerate()
        {
            let pointer = self.register();
            emit_instruction!(
                self;
                let {{ pointer.clone() }} = get_element_ptr {
                    inbounds: false,
                    element_type: {{ context_type.clone() }},
                    pointer: "%mal_native_context",
                    indices: [typed((int(32_u16)), "0"), typed((int(32_u16)), {{ index.to_string() }})],
                };
            );
            emit_instruction!(
                self;
                let {{ name }} = load {
                    ty: (ptr),
                    pointer: {{ pointer }},
                    alignment: {{ self.types.pointer_alignment() }},
                    metadata: [],
                };
            );
        }
        let parameter = self.types.value(&self.function.parameter.ty)?;
        let pointer = self.register();
        emit_instruction!(
            self;
            let {{ pointer.clone() }} = get_element_ptr {
                inbounds: false,
                element_type: {{ context_type }},
                pointer: "%mal_native_context",
                indices: [typed((int(32_u16)), "0"), typed((int(32_u16)), "3")],
            };
        );
        let mut reconstructed = self.register();
        emit_instruction!(
            self;
            let {{ reconstructed.clone() }} = load {
                ty: {{ parameter.llvm.clone() }},
                pointer: {{ pointer }},
                alignment: {{ parameter.alignment }},
                metadata: [],
            };
        );
        for (index, leaf) in plan.varying.iter().enumerate() {
            let inserted = if index + 1 == plan.varying.len() {
                "%mal_parameter".into()
            } else {
                self.register()
            };
            emit_instruction!(
                self;
                let {{ inserted.clone() }} = insert_value {
                    aggregate: typed({{ parameter.llvm.clone() }}, {{ reconstructed }}),
                    element: typed({{ self.types.value(&leaf.ty)?.llvm }}, {{ format!("%mal_native_parameter_{index}") }}),
                    indices: {{ leaf.path.clone() }},
                };
            );
            reconstructed = inserted;
        }
        Some(())
    }

    /// Continues the activation in the frames version when the native stack is used up.
    pub(in crate::backend::llvm::body) fn emit_native_entry_guard(&mut self) -> Option<()> {
        let mut parameters = vec![
            (llvm_type!(ptr), "%mal_context".into()),
            (llvm_type!(ptr), "%mal_control_top".into()),
            (llvm_type!(ptr), "%mal_environment".into()),
        ];
        if self.function.parameter.ty != Type::Unit {
            let value = self.types.value(&self.function.parameter.ty)?;
            parameters.push((value.llvm, "%mal_parameter".into()));
        }
        let result_llvm = self.types.value(&self.result_type)?.llvm;
        let name = super::super::function_name(self.function.id);
        let stack = self.register();
        emit_instruction!(
            self;
            let {{ stack.clone() }} = call {
                tail: false,
                result_type: (ptr),
                callee: direct("llvm.stacksave"),
                arguments: [],
            };
        );
        let flag = self.register();
        emit_instruction!(
            self;
            let {{ flag.clone() }} = call {
                tail: false,
                result_type: (int(8_u16)),
                callee: direct("mal_native_stack_is_deep"),
                arguments: [typed((ptr), "%mal_context"), typed((ptr), {{ stack }})],
            };
        );
        let deep = self.register();
        emit_instruction!(
            self;
            let {{ deep.clone() }} = compare {
                kind: {{ ComparisonKind::Integer }},
                predicate: {{ ComparisonPredicate::Ne }},
                ty: (int(8_u16)),
                left: {{ flag }},
                right: "0",
            };
        );
        let expected = self.register();
        emit_instruction!(
            self;
            let {{ expected.clone() }} = call {
                tail: false,
                result_type: (int(1_u16)),
                callee: direct("llvm.expect.i1"),
                arguments: [typed((int(1_u16)), {{ deep }}), typed((int(1_u16)), "false")],
            };
        );
        emit_terminator!(self; conditional
            { expected } => "mal_deep_entry", "mal_native_entry"
        );
        self.block("mal_deep_entry");
        let continued = self.register();
        emit_instruction!(
            self;
            let {{ continued.clone() }} = call {
                tail: false,
                result_type: {{ result_llvm.clone() }},
                callee: direct({{ format!("{name}_frames") }}),
                arguments: pairs({{ parameters }}),
            };
        );
        emit_terminator!(self; return { result_llvm } => { continued });
        self.block("mal_native_entry");
        Some(())
    }
}
