//! Transitions between the functions of a common control region: dispatch on the region target and the jump to it.

use crate::backend::llvm::syntax::emit_instruction;
use crate::backend::llvm::syntax::emit_terminator;
use crate::backend::llvm::syntax::{ComparisonKind, ComparisonPredicate, llvm_type};
use crate::closure::ast::{Atom, FunctionId};
use crate::control::ast::StateId;
use mal_frontend::check::ast::Type;

use super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_region_transition(
        &mut self,
        site: StateId,
        callee: &Atom,
        argument: &Atom,
        preserve_environment: bool,
        pending: &[super::super::PreparedValue],
    ) -> Option<()> {
        let callee_effect = self.ownership.terminator_operand_use(
            site,
            match &self.control.states[site.0].terminator {
                crate::control::ast::Terminator::Call { .. } => {
                    crate::execution::ownership::TerminatorOperand::CallCallee
                }
                crate::control::ast::Terminator::TailCall { .. } => {
                    crate::execution::ownership::TerminatorOperand::TailCallee
                }
                _ => return None,
            },
            callee,
        )?;
        let callee = self.prepare_atom_for_use(callee, callee_effect)?;
        let Type::Function { parameter, result } = &callee.value.ty else {
            return None;
        };
        let parameter = parameter.clone();
        let result = result.clone();
        let direct_target = match self.execution.control_calls.mode(site)? {
            crate::execution::ControlCallMode::DirectRegion(target) => Some(target),
            crate::execution::ControlCallMode::Dispatch => None,
            crate::execution::ControlCallMode::Direct(_)
            | crate::execution::ControlCallMode::DirectSelfTail => return None,
        };
        let code = if direct_target.is_none() {
            let closure_type = self.types.value(&callee.value.ty)?;
            let code = self.register();
            emit_instruction! {
                self;
                let { code.clone() } = extract_value {
                    aggregate: ({ closure_type.llvm }, { callee.value.representation.clone() }),
                    indices: [0],
                };
            };
            Some(code)
        } else {
            None
        };
        let environment = self.closure_environment(&callee.value)?;
        let argument_operand = match &self.control.states[site.0].terminator {
            crate::control::ast::Terminator::Call { .. } => {
                crate::execution::ownership::TerminatorOperand::CallArgument
            }
            crate::control::ast::Terminator::TailCall { .. } => {
                crate::execution::ownership::TerminatorOperand::TailArgument
            }
            _ => return None,
        };
        let argument_effect =
            self.ownership
                .terminator_operand_use(site, argument_operand, argument)?;
        let argument = self.prepare_atom_for_use(argument, argument_effect)?;
        if argument.value.ty != *parameter {
            return None;
        }
        for value in pending {
            self.commit_consumes(value)?;
        }
        self.commit_consumes(&callee)?;
        self.commit_consumes(&argument)?;
        self.emit_edge_drops(site, crate::execution::ownership::ControlPath::Single)?;
        if !preserve_environment {
            let previous = self.active_environment();
            emit_instruction! {
                self;
                call {
                    tail: false,
                    result_type: void,
                    callee: direct("mal_runtime_owner_release"),
                    arguments: [(ptr, { previous })],
                };
            };
        }
        emit_instruction! {
            self;
            store {
                value: (ptr, { environment.as_str() }),
                pointer: "%mal_active_environment",
                alignment: { self.types.pointer_alignment() },
                metadata: [],
            };
        };
        let targets = self
            .execution
            .control_regions
            .recursive_targets(site)?
            .to_vec();
        if let Some(target) = direct_target {
            if !targets.contains(&target) {
                return None;
            }
            return self.emit_region_target(target, &argument.value);
        }
        self.emit_region_dispatch(
            site,
            &targets,
            code.as_deref()?,
            &environment,
            &argument.value,
            &result,
        )
    }

    fn emit_region_dispatch(
        &mut self,
        site: StateId,
        targets: &[FunctionId],
        code: &str,
        environment: &str,
        argument: &EmittedValue,
        result: &Type,
    ) -> Option<()> {
        for (index, target) in targets.iter().enumerate() {
            let matched = self.register();
            emit_instruction! {
                self;
                let { matched.clone() } = compare {
                    kind: { ComparisonKind::Integer },
                    predicate: { ComparisonPredicate::Eq },
                    ty: ptr,
                    left: { code },
                    right: { format!("@{}", super::super::function_name(*target)) },
                };
            };
            let next = format!("mal_region_dispatch_{}_{}", site.0, index);
            emit_terminator! {
                self;
                branch {
                    condition: { matched },
                    then: { format!("mal_region_target_{}_{index}", site.0) },
                    otherwise: { next.clone() },
                };
            };
            self.block(next);
        }
        let has_native_target = self
            .execution
            .applications
            .targets(site)?
            .iter()
            .any(|target| !targets.contains(target));
        if has_native_target {
            let result_type = self.types.value(result)?;
            let mut arguments = vec![
                (llvm_type!(ptr), "%mal_context".into()),
                (llvm_type!(ptr), "%mal_control_top".into()),
                (llvm_type!(ptr), environment.into()),
            ];
            if argument.ty != Type::Unit {
                let argument_type = self.types.value(&argument.ty)?;
                arguments.push((argument_type.llvm, argument.representation.clone()));
            }
            let returned = self.register();
            self.sync_control_top()?;
            let arguments = crate::backend::llvm::syntax::TypedValue::from_pairs(arguments)?;
            emit_instruction! {
                self;
                let { returned.clone() } = call {
                    tail: false,
                    result_type: { result_type.llvm },
                    callee: indirect({ code }),
                    arguments: [..{ arguments }],
                };
            };
            if self
                .optimizations
                .site_may_relocate_control_storage(&self.execution.applications, site)
            {
                self.refresh_control_storage();
            }
            if argument.owned {
                self.release_value(&argument.ty, &argument.representation)?;
            }
            self.emit_continuation_return(
                site,
                &EmittedValue {
                    ty: result.clone(),
                    representation: returned,
                    owned: crate::execution::ownership::is_managed(result),
                },
            )?;
        } else {
            emit_terminator! {
                self;
                unreachable;
            };
        }
        for (index, target) in targets.iter().enumerate() {
            self.block(format!("mal_region_target_{}_{index}", site.0));
            self.emit_region_target(*target, argument)?;
        }
        Some(())
    }

    fn emit_region_target(&mut self, target: FunctionId, argument: &EmittedValue) -> Option<()> {
        let function = *self.index.control_functions.get(&target)?;
        let entry = function.entry;
        self.emit_parameter_handoff(
            target,
            argument,
            crate::execution::ownership::ParameterEntry::OwnedHandoff,
        )?;
        emit_terminator! {
            self;
            branch {
                target: { format!("mal_state_{}", entry.0) },
            };
        };
        Some(())
    }
}
