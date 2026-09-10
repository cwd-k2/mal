use std::collections::HashMap;

use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::closure::ast::{Atom, FunctionId};
use crate::control::ast::{Operation, Program, StateId, Terminator};
use crate::execution::ControlCallMode;

mod aggregate;
mod bridge;
mod frame;
mod memory;
mod operation;
mod plan;
mod scalar;
mod symbol;
pub(super) mod types;
mod value;

use plan::{
    collect_pattern_slot, insert_slot, main_function, pattern_value_type, reachable_states,
    top_levels_are_capture_free_closures,
};
use scalar::{comparison_predicate, scalar_type};
use types::{Types, is_bool};

pub(super) struct Output {
    pub(super) globals: String,
    pub(super) definitions: String,
    pub(super) main: FunctionId,
    pub(super) main_parameter: Type,
    pub(super) uses_control: bool,
    pub(super) uses_symbols: bool,
}

pub(super) fn supports(execution: &crate::execution::Program) -> bool {
    generate(execution, 8).is_some()
}

pub(super) fn generate(
    execution: &crate::execution::Program,
    pointer_size: usize,
) -> Option<Output> {
    let (main, main_parameter) = main_function(execution)?;
    if !top_levels_are_capture_free_closures(execution) {
        return None;
    }

    let types = Types::new(pointer_size)?;
    let mut globals = String::new();
    let mut definitions = String::new();
    let mut uses_control = false;
    let mut uses_symbols = false;
    for function in &execution.control.functions {
        let emitter = FunctionEmitter::new(execution, function.id, types)?;
        uses_control |= emitter.has_frames;
        uses_symbols |= emitter.uses_symbols;
        let emitted = emitter.emit()?;
        globals.push_str(&emitted.globals);
        definitions.push_str(&emitted.definition);
        definitions.push('\n');
    }
    Some(Output {
        globals,
        definitions,
        main,
        main_parameter,
        uses_control,
        uses_symbols,
    })
}

struct FunctionEmitter<'a> {
    execution: &'a crate::execution::Program,
    control: &'a Program,
    function: &'a crate::control::ast::Function,
    states: Vec<StateId>,
    result_type: Type,
    slots: HashMap<ValueId, Slot>,
    has_frames: bool,
    external_storage: Option<(usize, usize)>,
    types: Types,
    uses_symbols: bool,
    next_register: usize,
    globals: String,
    output: String,
}

#[derive(Clone)]
pub(super) struct Slot {
    index: usize,
    ty: Type,
}

#[derive(Clone)]
struct EmittedValue {
    ty: Type,
    representation: String,
    owned: bool,
}

struct EmittedFunction {
    globals: String,
    definition: String,
}

impl<'a> FunctionEmitter<'a> {
    fn new(execution: &'a crate::execution::Program, id: FunctionId, types: Types) -> Option<Self> {
        let function = execution
            .control
            .functions
            .iter()
            .find(|function| function.id == id)?;
        let lowered = execution
            .lowered
            .functions
            .iter()
            .find(|function| function.id == id)?;
        if types.value(&function.parameter.ty).is_none()
            || types.value(&lowered.body.result.ty).is_none()
        {
            return None;
        }
        let states = reachable_states(&execution.control, function.entry);
        let mut slots = HashMap::new();
        if let Some(id) = function.parameter.binding {
            insert_slot(&mut slots, id, function.parameter.ty.clone());
        }
        for state in &states {
            let state = &execution.control.states[state.0];
            if let Some(pattern) = &state.input {
                collect_pattern_slot(pattern, &mut slots, types)?;
            }
            for binding in &state.bindings {
                collect_pattern_slot(&binding.pattern, &mut slots, types)?;
            }
        }
        let frame_sites = states
            .iter()
            .filter(|site| execution.control_frames.frame(**site).is_some())
            .copied()
            .collect::<Vec<_>>();
        if frame_sites.iter().any(|site| {
            let frame = execution
                .control_frames
                .frame(*site)
                .expect("collected frame site");
            execution.applications.direct_target(*site) != Some(id)
                || frame.carries_environment
                || frame
                    .fields
                    .iter()
                    .any(|field| types.value(&field.value.ty).is_none())
        }) {
            return None;
        }
        let external_ids = states.iter().flat_map(|site| {
            execution.control.states[site.0]
                .bindings
                .iter()
                .filter_map(|binding| match binding.operation {
                    Operation::ExternalCall { id, .. } => Some(id),
                    _ => None,
                })
        });
        let mut external_storage = None;
        for id in external_ids {
            let external = execution
                .lowered
                .interface
                .externals
                .iter()
                .find(|external| external.id == id)?;
            for ty in [&external.parameter, &external.result] {
                if !super::bridge_type_supported(ty) {
                    return None;
                }
                let value = types.value(ty)?;
                let (size, alignment) = external_storage.unwrap_or((0usize, 1usize));
                external_storage = Some((size.max(value.size), alignment.max(value.alignment)));
            }
        }
        let uses_symbols = slots
            .values()
            .any(|slot| crate::execution::ownership::is_managed(&slot.ty))
            || crate::execution::ownership::is_managed(&function.parameter.ty)
            || crate::execution::ownership::is_managed(&lowered.body.result.ty);
        Some(Self {
            execution,
            control: &execution.control,
            function,
            result_type: lowered.body.result.ty.clone(),
            states,
            slots,
            has_frames: !frame_sites.is_empty(),
            external_storage,
            types,
            uses_symbols,
            next_register: 0,
            globals: String::new(),
            output: String::new(),
        })
    }

    fn emit(mut self) -> Option<EmittedFunction> {
        self.emit_environment_destructor()?;
        let parameter = if self.function.parameter.ty == Type::Unit {
            "ptr %mal_context, ptr %mal_environment".to_string()
        } else {
            let parameter = self.types.value(&self.function.parameter.ty)?;
            format!(
                "ptr %mal_context, ptr %mal_environment, {} %mal_parameter",
                parameter.llvm
            )
        };
        let result = self.types.value(&self.result_type)?;
        self.line(format!(
            "define internal {} @{}({parameter}) {{",
            result.llvm,
            function_name(self.function.id)?
        ));
        self.line("entry:");
        let mut slots = self.slots.values().cloned().collect::<Vec<_>>();
        slots.sort_by_key(|slot| slot.index);
        for slot in slots {
            let value_type = self.types.value(&slot.ty)?;
            self.line(format!(
                "  %mal_slot_{} = alloca {}, align {}",
                slot.index, value_type.llvm, value_type.alignment
            ));
            if crate::execution::ownership::is_managed(&slot.ty) {
                self.line(format!(
                    "  store {} zeroinitializer, ptr %mal_slot_{}, align {}",
                    value_type.llvm, slot.index, value_type.alignment
                ));
            }
        }
        if self.has_frames {
            self.line("  %mal_control_top = alloca i64, align 8");
        }
        if let Some((size, alignment)) = self.external_storage {
            self.line(format!(
                "  %mal_bridge_argument = alloca [{size} x i8], align {alignment}"
            ));
            self.line(format!(
                "  %mal_bridge_result = alloca [{size} x i8], align {alignment}"
            ));
        }
        if let Some(id) = self.function.parameter.binding {
            let slot = self.slots.get(&id)?.clone();
            let value_type = self.types.value(&slot.ty)?;
            let mut parameter = EmittedValue {
                ty: slot.ty.clone(),
                representation: "%mal_parameter".into(),
                owned: false,
            };
            self.retain_if_borrowed(&mut parameter)?;
            self.line(format!(
                "  store {} {}, ptr %mal_slot_{}, align {}",
                value_type.llvm, parameter.representation, slot.index, value_type.alignment
            ));
        }
        if self.has_frames {
            self.line("  store i64 0, ptr %mal_control_top, align 8");
        }
        self.line(format!("  br label %mal_state_{}", self.function.entry.0));

        for site in self.states.clone() {
            self.emit_state(site)?;
        }
        self.line("}");
        Some(EmittedFunction {
            globals: self.globals,
            definition: self.output,
        })
    }

    fn emit_state(&mut self, site: StateId) -> Option<()> {
        let state = &self.control.states[site.0];
        self.line(format!("mal_state_{}:", site.0));
        for binding in &state.bindings {
            let value =
                self.emit_operation(&binding.operation, pattern_value_type(&binding.pattern))?;
            self.store_pattern(&binding.pattern, value.as_ref())?;
        }
        self.emit_terminator(site, &state.terminator)
    }

    fn emit_terminator(&mut self, site: StateId, terminator: &Terminator) -> Option<()> {
        match terminator {
            Terminator::Return(value) => {
                let mut value = self.atom(value)?;
                self.retain_if_borrowed(&mut value)?;
                if self.has_frames {
                    if value.ty != self.result_type {
                        return None;
                    }
                    self.emit_frame_return(site, &value)?;
                } else {
                    self.release_local_managed();
                    let value_type = self.types.value(&value.ty)?;
                    self.line(format!(
                        "  ret {} {}",
                        value_type.llvm, value.representation
                    ));
                }
            }
            Terminator::Goto(target) => {
                self.line(format!("  br label %mal_state_{}", target.0));
            }
            Terminator::Jump { target, value } => {
                let value = self.atom(value)?;
                let input = self.control.states[target.0].input.as_ref()?;
                self.store_pattern(input, Some(&value))?;
                self.line(format!("  br label %mal_state_{}", target.0));
            }
            Terminator::PrimitiveBranch {
                operator,
                left,
                right,
                otherwise,
                then,
            } => {
                let left = self.atom(left)?;
                let right = self.atom(right)?;
                if left.ty != right.ty {
                    return None;
                }
                let condition = self.register();
                if left.ty == Type::Symbol {
                    let equality = self.register();
                    self.line(format!(
                        "  {equality} = call i8 @mal_runtime_symbol_equal(ptr {}, ptr {})",
                        left.representation, right.representation
                    ));
                    let predicate = match operator {
                        crate::core::ast::BinaryPrimitive::Equal => "ne",
                        crate::core::ast::BinaryPrimitive::NotEqual => "eq",
                        _ => return None,
                    };
                    self.line(format!("  {condition} = icmp {predicate} i8 {equality}, 0"));
                } else if is_bool(&left.ty) {
                    let predicate = match operator {
                        crate::core::ast::BinaryPrimitive::Equal => "eq",
                        crate::core::ast::BinaryPrimitive::NotEqual => "ne",
                        _ => return None,
                    };
                    self.line(format!(
                        "  {condition} = icmp {predicate} i1 {}, {}",
                        left.representation, right.representation
                    ));
                } else {
                    let predicate = comparison_predicate(*operator)?;
                    let scalar = scalar_type(&left.ty)?;
                    let predicate = predicate.for_scalar(scalar);
                    let instruction = if scalar.floating { "fcmp" } else { "icmp" };
                    self.line(format!(
                        "  {condition} = {instruction} {predicate} {} {}, {}",
                        scalar.llvm, left.representation, right.representation
                    ));
                }
                self.line(format!(
                    "  br i1 {condition}, label %mal_state_{}, label %mal_state_{}",
                    then.0, otherwise.0
                ));
            }
            Terminator::Call {
                callee,
                argument,
                resume,
            } => match self.execution.control_calls.mode(site)? {
                ControlCallMode::Direct(target) => {
                    let result = self.emit_call(target, callee, argument, false)?;
                    let input = self.control.states[resume.0].input.as_ref()?;
                    self.store_pattern(input, Some(&result))?;
                    self.line(format!("  br label %mal_state_{}", resume.0));
                }
                ControlCallMode::Dispatch
                    if self.execution.control_frames.frame(site).is_some() =>
                {
                    self.emit_frame_call(site, argument)?;
                }
                ControlCallMode::Dispatch => {
                    let Terminator::Call { callee, .. } = terminator else {
                        unreachable!()
                    };
                    let result = self.emit_indirect_call(callee, argument, false)?;
                    let input = self.control.states[resume.0].input.as_ref()?;
                    self.store_pattern(input, Some(&result))?;
                    self.line(format!("  br label %mal_state_{}", resume.0));
                }
                ControlCallMode::DirectSelfTail => return None,
            },
            Terminator::TailCall { callee, argument } => {
                match self.execution.control_calls.mode(site)? {
                    ControlCallMode::DirectSelfTail => {
                        let argument = self
                            .execution
                            .tail_calls
                            .forwarded_self_argument(site)
                            .unwrap_or(argument);
                        let mut value = self.atom(argument)?;
                        let parameter = self.function.parameter.binding?;
                        let slot = self.slots.get(&parameter)?.clone();
                        if slot.ty != value.ty {
                            return None;
                        }
                        self.retain_if_borrowed(&mut value)?;
                        self.release_local_managed();
                        let value_type = self.types.value(&slot.ty)?;
                        self.line(format!(
                            "  store {} {}, ptr %mal_slot_{}, align {}",
                            value_type.llvm, value.representation, slot.index, value_type.alignment
                        ));
                        self.line(format!("  br label %mal_state_{}", self.function.entry.0));
                    }
                    ControlCallMode::Direct(target) => {
                        let result = self.emit_call(target, callee, argument, true)?;
                        self.release_local_managed();
                        let value_type = self.types.value(&result.ty)?;
                        self.line(format!(
                            "  ret {} {}",
                            value_type.llvm, result.representation
                        ));
                    }
                    ControlCallMode::Dispatch => {
                        let Terminator::TailCall { callee, .. } = terminator else {
                            unreachable!()
                        };
                        let result = self.emit_indirect_call(callee, argument, true)?;
                        self.release_local_managed();
                        let value_type = self.types.value(&result.ty)?;
                        self.line(format!(
                            "  ret {} {}",
                            value_type.llvm, result.representation
                        ));
                    }
                }
            }
            Terminator::Case { scrutinee, arms } => self.emit_case(site, scrutinee, arms)?,
        }
        Some(())
    }

    fn emit_call(
        &mut self,
        target: FunctionId,
        callee: &Atom,
        argument: &Atom,
        tail: bool,
    ) -> Option<EmittedValue> {
        let target = self
            .control
            .functions
            .iter()
            .find(|function| function.id == target)?;
        let callee = self.atom(callee)?;
        let closure_type = self.types.value(&callee.ty)?;
        let environment = self.register();
        self.line(format!(
            "  {environment} = extractvalue {} {}, 1",
            closure_type.llvm, callee.representation
        ));
        let arguments = if target.parameter.ty == Type::Unit {
            format!("ptr %mal_context, ptr {environment}")
        } else {
            let argument = self.atom(argument)?;
            if argument.ty != target.parameter.ty {
                return None;
            }
            let value_type = self.types.value(&argument.ty)?;
            format!(
                "ptr %mal_context, ptr {environment}, {} {}",
                value_type.llvm, argument.representation
            )
        };
        let lowered = self
            .execution
            .lowered
            .functions
            .iter()
            .find(|function| function.id == target.id)?;
        let result_type = lowered.body.result.ty.clone();
        let result_value_type = self.types.value(&result_type)?;
        let register = self.register();
        let tail = if tail { "tail " } else { "" };
        self.line(format!(
            "  {register} = {tail}call {} @{}({arguments})",
            result_value_type.llvm,
            function_name(target.id)?
        ));
        Some(EmittedValue {
            ty: result_type,
            representation: register,
            owned: crate::execution::ownership::is_managed(&lowered.body.result.ty),
        })
    }

    fn emit_environment_destructor(&mut self) -> Option<()> {
        if self.function.environment.is_empty() {
            return Some(());
        }
        let environment_type = Type::Product(
            self.function
                .environment
                .iter()
                .map(|field| field.ty.clone())
                .collect(),
        );
        let value_type = self.types.value(&environment_type)?;
        self.line(format!(
            "define internal void @mal_destroy_environment_{}(ptr %mal_environment) {{",
            function_number(self.function.id)?
        ));
        self.line("entry:");
        let environment = self.register();
        self.line(format!(
            "  {environment} = load {}, ptr %mal_environment, align {}",
            value_type.llvm, value_type.alignment
        ));
        self.release_value(&environment_type, &environment)?;
        self.line("  ret void");
        self.line("}");
        self.line("");
        self.next_register = 0;
        Some(())
    }

    fn emit_indirect_call(
        &mut self,
        callee: &Atom,
        argument: &Atom,
        tail: bool,
    ) -> Option<EmittedValue> {
        let callee = self.atom(callee)?;
        let Type::Function { parameter, result } = &callee.ty else {
            return None;
        };
        let closure_type = self.types.value(&callee.ty)?;
        let code = self.register();
        self.line(format!(
            "  {code} = extractvalue {} {}, 0",
            closure_type.llvm, callee.representation
        ));
        let environment = self.register();
        self.line(format!(
            "  {environment} = extractvalue {} {}, 1",
            closure_type.llvm, callee.representation
        ));
        let arguments = if **parameter == Type::Unit {
            if argument.ty != Type::Unit {
                return None;
            }
            format!("ptr %mal_context, ptr {environment}")
        } else {
            let argument = self.atom(argument)?;
            if argument.ty != **parameter {
                return None;
            }
            let value_type = self.types.value(parameter)?;
            format!(
                "ptr %mal_context, ptr {environment}, {} {}",
                value_type.llvm, argument.representation
            )
        };
        let result_type = self.types.value(result)?;
        let register = self.register();
        let tail = if tail { "tail " } else { "" };
        self.line(format!(
            "  {register} = {tail}call {} {code}({arguments})",
            result_type.llvm
        ));
        Some(EmittedValue {
            ty: (**result).clone(),
            representation: register,
            owned: crate::execution::ownership::is_managed(result),
        })
    }

    fn register(&mut self) -> String {
        let register = format!("%mal_value_{}", self.next_register);
        self.next_register += 1;
        register
    }

    fn label_id(&mut self) -> usize {
        let id = self.next_register;
        self.next_register += 1;
        id
    }

    fn line(&mut self, line: impl AsRef<str>) {
        self.output.push_str(line.as_ref());
        self.output.push('\n');
    }
}

fn function_name(id: FunctionId) -> Option<String> {
    Some(format!("mal_function_{}", function_number(id)?))
}

fn function_number(id: FunctionId) -> Option<u32> {
    match id {
        FunctionId::Lambda(id) => Some(id.0),
        FunctionId::Memory(_) => None,
    }
}
