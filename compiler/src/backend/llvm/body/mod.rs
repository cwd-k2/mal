use std::collections::HashMap;

use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::closure::ast::{Atom, FunctionId};
use crate::control::ast::{Operation, Program, StateId, Terminator};
use crate::execution::{ControlCallMode, ControlRegionId, ParameterDestination};

mod aggregate;
mod bridge;
mod frame;
mod memory;
mod operation;
pub(in crate::backend::llvm) mod ownership;
mod plan;
mod scalar;
mod symbol;
pub(super) mod types;
mod value;

use plan::{
    TopLevelConstants, collect_pattern_ids, collect_pattern_slot, insert_slot, main_function,
    pattern_value_type, reachable_states,
};
use scalar::{comparison_predicate, scalar_type};
use types::{Types, is_bool};

pub(super) struct Output {
    pub(super) globals: String,
    pub(super) definitions: String,
    pub(super) main: FunctionId,
    pub(super) main_parameter: Type,
    pub(super) uses_control: bool,
    pub(super) uses_symbol_runtime: bool,
}

#[cfg(test)]
pub(super) fn supports(execution: &crate::execution::Program) -> bool {
    generate(
        execution,
        8,
        super::optimization::OptimizationSet::production(),
    )
    .is_some()
}

pub(super) fn generate(
    execution: &crate::execution::Program,
    pointer_size: usize,
    enabled: super::optimization::OptimizationSet,
) -> Option<Output> {
    let (main, main_parameter) = main_function(execution)?;
    let types = Types::new(pointer_size)?;
    let top_levels = TopLevelConstants::new(execution, types)?;
    let ownership = ownership::Plan::new(&execution.control);
    let optimizations =
        super::optimization::OptimizationPlan::new(&execution.control, &ownership, enabled);
    debug_assert!(optimizations.is_valid(&execution.control, &ownership, enabled));
    let mut globals = top_levels.globals().to_string();
    let mut definitions = String::new();
    let mut uses_control = false;
    for function in &execution.control.functions {
        let emitter = FunctionEmitter::new(
            execution,
            function.id,
            types,
            &top_levels,
            &ownership,
            &optimizations,
        )?;
        uses_control |= !emitter.frame_sites.is_empty();
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
        uses_symbol_runtime: symbol::program_uses_runtime(execution),
    })
}

struct FunctionEmitter<'a> {
    execution: &'a crate::execution::Program,
    control: &'a Program,
    function: &'a crate::control::ast::Function,
    current_function: FunctionId,
    common_region: Option<ControlRegionId>,
    states: Vec<StateId>,
    state_functions: HashMap<StateId, FunctionId>,
    result_type: Type,
    slots: HashMap<ValueId, Slot>,
    function_slots: HashMap<FunctionId, Vec<ValueId>>,
    frame_sites: Vec<StateId>,
    external_storage: Option<(usize, usize)>,
    types: Types,
    top_levels: &'a TopLevelConstants,
    ownership: &'a ownership::Plan,
    optimizations: &'a super::optimization::OptimizationPlan,
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
    fn new(
        execution: &'a crate::execution::Program,
        id: FunctionId,
        types: Types,
        top_levels: &'a TopLevelConstants,
        ownership: &'a ownership::Plan,
        optimizations: &'a super::optimization::OptimizationPlan,
    ) -> Option<Self> {
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
        let common_region = execution
            .control_regions
            .function_region(id)
            .filter(|region| execution.control_calls.requires_common_control(*region));
        let functions = common_region.map_or_else(
            || vec![id],
            |region| execution.control_regions.functions(region).to_vec(),
        );
        let function_states = functions
            .iter()
            .map(|function| {
                let entry = execution
                    .control
                    .functions
                    .iter()
                    .find(|candidate| candidate.id == *function)
                    .expect("control region function has an entry")
                    .entry;
                (*function, reachable_states(&execution.control, entry))
            })
            .collect::<Vec<_>>();
        let states = function_states
            .iter()
            .flat_map(|(_, states)| states.iter().copied())
            .collect::<Vec<_>>();
        let state_functions = function_states
            .iter()
            .flat_map(|(function, states)| states.iter().map(|state| (*state, *function)))
            .collect::<HashMap<_, _>>();
        if state_functions.len() != states.len() {
            return None;
        }
        let mut slots = HashMap::new();
        let mut function_slots = HashMap::new();
        for (function_id, function_states) in &function_states {
            let region_function = execution
                .control
                .functions
                .iter()
                .find(|candidate| candidate.id == *function_id)?;
            let mut ids = Vec::new();
            if let ParameterDestination::Bind(id) =
                execution.parameters.destination(*function_id)?
            {
                insert_slot(&mut slots, id, region_function.parameter.ty.clone());
                ids.push(id);
            }
            for state in function_states {
                let state = &execution.control.states[state.0];
                if let Some(pattern) = &state.input {
                    collect_pattern_slot(pattern, &mut slots, types)?;
                    collect_pattern_ids(pattern, &mut ids);
                }
                for binding in &state.bindings {
                    collect_pattern_slot(&binding.pattern, &mut slots, types)?;
                    collect_pattern_ids(&binding.pattern, &mut ids);
                }
            }
            let mut unique = Vec::new();
            for id in ids {
                if !unique.contains(&id) {
                    unique.push(id);
                }
            }
            function_slots.insert(*function_id, unique);
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
            (common_region.is_none()
                && (execution.control_calls.mode(*site) != Some(ControlCallMode::DirectRegion(id))
                    || frame.carries_environment))
                || common_region.is_some_and(|region| {
                    execution.control_regions.site_region(*site) != Some(region)
                })
                || frame
                    .fields
                    .iter()
                    .any(|field| types.value(&field.ty).is_none())
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
        Some(Self {
            execution,
            control: &execution.control,
            function,
            current_function: id,
            common_region,
            result_type: lowered.body.result.ty.clone(),
            states,
            state_functions,
            slots,
            function_slots,
            frame_sites,
            external_storage,
            types,
            top_levels,
            ownership,
            optimizations,
            next_register: 0,
            globals: String::new(),
            output: String::new(),
        })
    }

    fn emit(mut self) -> Option<EmittedFunction> {
        self.emit_environment_destructor()?;
        let parameter = if self.function.parameter.ty == Type::Unit {
            "ptr %mal_context, ptr %mal_control_top, ptr %mal_environment".to_string()
        } else {
            let parameter = self.types.value(&self.function.parameter.ty)?;
            format!(
                "ptr %mal_context, ptr %mal_control_top, ptr %mal_environment, {} %mal_parameter",
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
        if !self.frame_sites.is_empty() {
            self.line(format!(
                "  %mal_control_base = load {}, ptr %mal_control_top, align {}",
                self.types.pointer_integer()?,
                self.types.pointer_size()
            ));
        }
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
        if self.common_region.is_some() {
            self.line(format!(
                "  %mal_active_environment = alloca ptr, align {}",
                self.types.pointer_size()
            ));
            let environment = self.register();
            self.line(format!(
                "  {environment} = call ptr @mal_runtime_environment_retain(ptr %mal_context, ptr %mal_environment)"
            ));
            self.line(format!(
                "  store ptr {environment}, ptr %mal_active_environment, align {}",
                self.types.pointer_size()
            ));
        }
        if let Some((size, alignment)) = self.external_storage {
            self.line(format!(
                "  %mal_bridge_argument = alloca [{size} x i8], align {alignment}"
            ));
            self.line(format!(
                "  %mal_bridge_result = alloca [{size} x i8], align {alignment}"
            ));
        }
        if let ParameterDestination::Bind(_) =
            self.execution.parameters.destination(self.function.id)?
        {
            let mut parameter = EmittedValue {
                ty: self.function.parameter.ty.clone(),
                representation: "%mal_parameter".into(),
                owned: false,
            };
            self.retain_if_borrowed(&mut parameter)?;
            self.emit_parameter_handoff(self.function.id, &parameter)?;
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
        self.current_function = self.function_for_state(site)?;
        let state = &self.control.states[site.0];
        self.line(format!("mal_state_{}:", site.0));
        for (binding_index, binding) in state.bindings.iter().enumerate() {
            let value = self.emit_operation(
                &binding.operation,
                pattern_value_type(&binding.pattern),
                self.optimizations.symbol_concat_mode(site, binding_index),
            )?;
            self.store_pattern(&binding.pattern, value.as_ref())?;
            let mut dead = self.ownership.dead_values(site, binding_index).to_vec();
            dead.sort_by_key(|id| self.slots.get(id).map_or(usize::MAX, |slot| slot.index));
            for id in dead {
                self.release_dead_slot(id)?;
            }
        }
        self.emit_terminator(site, &state.terminator)
    }

    fn emit_terminator(&mut self, site: StateId, terminator: &Terminator) -> Option<()> {
        match terminator {
            Terminator::Return(value) => {
                let mut value = self.atom(value)?;
                self.retain_if_borrowed(&mut value)?;
                let result_type = self.current_result_type()?;
                if value.ty != result_type {
                    return None;
                }
                self.emit_continuation_return(site, &value)?;
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
                    self.emit_frame_call(site, callee, argument)?;
                }
                ControlCallMode::DirectRegion(_)
                    if self.execution.control_frames.frame(site).is_some() =>
                {
                    self.emit_frame_call(site, callee, argument)?;
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
                ControlCallMode::DirectRegion(_) => return None,
            },
            Terminator::TailCall { callee, argument } => {
                match self.execution.control_calls.mode(site)? {
                    ControlCallMode::DirectSelfTail => {
                        let argument = self
                            .execution
                            .control_calls
                            .forwarded_self_argument(site)
                            .unwrap_or(argument);
                        let mut value = self.atom(argument)?;
                        let function = self.current_function()?.clone();
                        if function.parameter.ty != value.ty {
                            return None;
                        }
                        self.retain_if_borrowed(&mut value)?;
                        self.release_local_managed();
                        self.emit_parameter_handoff(function.id, &value)?;
                        self.line(format!("  br label %mal_state_{}", function.entry.0));
                    }
                    ControlCallMode::Direct(target) => {
                        let result = self.emit_call(target, callee, argument, true)?;
                        self.emit_continuation_return(site, &result)?;
                    }
                    ControlCallMode::DirectRegion(_) => {
                        self.emit_region_transition(site, callee, argument, false)?;
                    }
                    ControlCallMode::Dispatch => {
                        if self.common_region.is_some()
                            && self.execution.control_regions.site_region(site)
                                == self.common_region
                        {
                            self.emit_region_transition(site, callee, argument, false)?;
                        } else {
                            let result = self.emit_indirect_call(callee, argument, true)?;
                            self.emit_continuation_return(site, &result)?;
                        }
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
            format!("ptr %mal_context, ptr %mal_control_top, ptr {environment}")
        } else {
            let argument = self.atom(argument)?;
            if argument.ty != target.parameter.ty {
                return None;
            }
            let value_type = self.types.value(&argument.ty)?;
            format!(
                "ptr %mal_context, ptr %mal_control_top, ptr {environment}, {} {}",
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

    fn emit_parameter_handoff(&mut self, target: FunctionId, value: &EmittedValue) -> Option<()> {
        let function = self
            .control
            .functions
            .iter()
            .find(|function| function.id == target)?;
        if function.parameter.ty != value.ty
            || (crate::execution::ownership::is_managed(&value.ty) && !value.owned)
        {
            return None;
        }
        match self.execution.parameters.destination(target)? {
            ParameterDestination::Bind(binding) => {
                let slot = self.slots.get(&binding)?.clone();
                if slot.ty != value.ty {
                    return None;
                }
                let value_type = self.types.value(&slot.ty)?;
                self.line(format!(
                    "  store {} {}, ptr %mal_slot_{}, align {}",
                    value_type.llvm, value.representation, slot.index, value_type.alignment
                ));
            }
            ParameterDestination::Discard if value.owned => {
                self.release_value(&value.ty, &value.representation)?;
            }
            ParameterDestination::Discard => {}
        }
        Some(())
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
            format!("ptr %mal_context, ptr %mal_control_top, ptr {environment}")
        } else {
            let argument = self.atom(argument)?;
            if argument.ty != **parameter {
                return None;
            }
            let value_type = self.types.value(parameter)?;
            format!(
                "ptr %mal_context, ptr %mal_control_top, ptr {environment}, {} {}",
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

    fn current_function(&self) -> Option<&crate::control::ast::Function> {
        self.control
            .functions
            .iter()
            .find(|function| function.id == self.current_function)
    }

    fn current_result_type(&self) -> Option<Type> {
        self.execution
            .lowered
            .functions
            .iter()
            .find(|function| function.id == self.current_function)
            .map(|function| function.body.result.ty.clone())
    }

    fn function_for_state(&self, site: StateId) -> Option<FunctionId> {
        self.state_functions.get(&site).copied()
    }

    fn active_environment(&mut self) -> String {
        if self.common_region.is_none() {
            return "%mal_environment".into();
        }
        let environment = self.register();
        self.line(format!(
            "  {environment} = load ptr, ptr %mal_active_environment, align {}",
            self.types.pointer_size()
        ));
        environment
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
    let name = match id {
        FunctionId::Lambda(id) => format!("mal_function_{}", id.0),
        FunctionId::Memory(primitive) => {
            format!("mal_memory_function_{}", memory_primitive_name(primitive)?)
        }
    };
    Some(name)
}

fn function_number(id: FunctionId) -> Option<u32> {
    match id {
        FunctionId::Lambda(id) => Some(id.0),
        FunctionId::Memory(_) => None,
    }
}

fn memory_primitive_name(primitive: crate::check::ast::MemoryPrimitive) -> Option<&'static str> {
    use crate::check::ast::{MemoryPrimitive, MemoryScalar};

    Some(match primitive {
        MemoryPrimitive::Load(MemoryScalar::Int8) => "load_int8",
        MemoryPrimitive::Load(MemoryScalar::Int16) => "load_int16",
        MemoryPrimitive::Load(MemoryScalar::Int32) => "load_int32",
        MemoryPrimitive::Load(MemoryScalar::Int64) => "load_int64",
        MemoryPrimitive::Load(MemoryScalar::UInt8) => "load_uint8",
        MemoryPrimitive::Load(MemoryScalar::UInt16) => "load_uint16",
        MemoryPrimitive::Load(MemoryScalar::UInt32) => "load_uint32",
        MemoryPrimitive::Load(MemoryScalar::UInt64) => "load_uint64",
        MemoryPrimitive::Load(MemoryScalar::Float32) => "load_float32",
        MemoryPrimitive::Load(MemoryScalar::Float64) => "load_float64",
        MemoryPrimitive::Store(MemoryScalar::Int8) => "store_int8",
        MemoryPrimitive::Store(MemoryScalar::Int16) => "store_int16",
        MemoryPrimitive::Store(MemoryScalar::Int32) => "store_int32",
        MemoryPrimitive::Store(MemoryScalar::Int64) => "store_int64",
        MemoryPrimitive::Store(MemoryScalar::UInt8) => "store_uint8",
        MemoryPrimitive::Store(MemoryScalar::UInt16) => "store_uint16",
        MemoryPrimitive::Store(MemoryScalar::UInt32) => "store_uint32",
        MemoryPrimitive::Store(MemoryScalar::UInt64) => "store_uint64",
        MemoryPrimitive::Store(MemoryScalar::Float32) => "store_float32",
        MemoryPrimitive::Store(MemoryScalar::Float64) => "store_float64",
        MemoryPrimitive::LoadPtr => "load_ptr",
        MemoryPrimitive::StorePtr => "store_ptr",
        MemoryPrimitive::LoadSymbol => "load_symbol",
        MemoryPrimitive::StoreSymbol => "store_symbol",
        MemoryPrimitive::OffsetForward | MemoryPrimitive::OffsetBackward => return None,
    })
}
