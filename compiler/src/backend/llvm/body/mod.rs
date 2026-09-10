use std::collections::HashMap;

use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::closure::ast::{Atom, FunctionId};
use crate::control::ast::{Operation, Program, StateId, Terminator};
use crate::core::ast::UnaryPrimitive;
use crate::execution::ControlCallMode;

mod aggregate;
mod bridge;
mod frame;
mod plan;
mod scalar;
mod types;
mod value;

use plan::{
    collect_pattern_slot, insert_slot, main_function, pattern_value_type, reachable_states,
    top_levels_are_capture_free_closures,
};
use scalar::{arithmetic_instruction, comparison_predicate, scalar_type};
use types::{is_bool, value_type};

pub(super) struct Output {
    pub(super) definitions: String,
    pub(super) main: FunctionId,
    pub(super) uses_control: bool,
}

pub(super) fn generate(execution: &crate::execution::Program) -> Option<Output> {
    let main = main_function(execution)?;
    if !top_levels_are_capture_free_closures(execution) {
        return None;
    }

    let mut definitions = String::new();
    let mut uses_control = false;
    for function in &execution.control.functions {
        let emitter = FunctionEmitter::new(execution, function.id)?;
        uses_control |= emitter.has_frames;
        definitions.push_str(&emitter.emit()?);
        definitions.push('\n');
    }
    Some(Output {
        definitions,
        main,
        uses_control,
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
    has_external_calls: bool,
    next_register: usize,
    output: String,
}

#[derive(Clone)]
pub(super) struct Slot {
    index: usize,
    ty: Type,
}

struct EmittedValue {
    ty: Type,
    representation: String,
}

impl<'a> FunctionEmitter<'a> {
    fn new(execution: &'a crate::execution::Program, id: FunctionId) -> Option<Self> {
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
        if !lowered.environment.is_empty()
            || value_type(&function.parameter.ty).is_none()
            || value_type(&lowered.body.result.ty).is_none()
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
                collect_pattern_slot(pattern, &mut slots)?;
            }
            for binding in &state.bindings {
                collect_pattern_slot(&binding.pattern, &mut slots)?;
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
                    .any(|field| field.managed || value_type(&field.value.ty).is_none())
        }) {
            return None;
        }
        let has_external_calls = states.iter().any(|site| {
            execution.control.states[site.0]
                .bindings
                .iter()
                .any(|binding| matches!(binding.operation, Operation::ExternalCall { .. }))
        });
        Some(Self {
            execution,
            control: &execution.control,
            function,
            result_type: lowered.body.result.ty.clone(),
            states,
            slots,
            has_frames: !frame_sites.is_empty(),
            has_external_calls,
            next_register: 0,
            output: String::new(),
        })
    }

    fn emit(mut self) -> Option<String> {
        let parameter = if self.function.parameter.ty == Type::Unit {
            "ptr %mal_context".to_string()
        } else {
            let parameter = value_type(&self.function.parameter.ty)?;
            format!("ptr %mal_context, {} %mal_parameter", parameter.llvm)
        };
        let result = value_type(&self.result_type)?;
        self.line(format!(
            "define internal {} @{}({parameter}) {{",
            result.llvm,
            function_name(self.function.id)?
        ));
        self.line("entry:");
        let mut slots = self.slots.values().cloned().collect::<Vec<_>>();
        slots.sort_by_key(|slot| slot.index);
        for slot in slots {
            let value_type = value_type(&slot.ty)?;
            self.line(format!(
                "  %mal_slot_{} = alloca {}, align {}",
                slot.index, value_type.llvm, value_type.alignment
            ));
        }
        if self.has_frames {
            self.line("  %mal_control_top = alloca i64, align 8");
        }
        if self.has_external_calls {
            self.line("  %mal_bridge_argument = alloca [8 x i8], align 8");
            self.line("  %mal_bridge_result = alloca [8 x i8], align 8");
        }
        if let Some(id) = self.function.parameter.binding {
            let slot = self.slots.get(&id)?.clone();
            let value_type = value_type(&slot.ty)?;
            self.line(format!(
                "  store {} %mal_parameter, ptr %mal_slot_{}, align {}",
                value_type.llvm, slot.index, value_type.alignment
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
        Some(self.output)
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

    fn emit_operation(
        &mut self,
        operation: &Operation,
        result_type: Option<&Type>,
    ) -> Option<Option<EmittedValue>> {
        match operation {
            Operation::Atom(atom) => self.atom(atom).map(Some),
            Operation::PrimitiveUnary { operator, operand } => {
                let operand = self.atom(operand)?;
                let scalar = scalar_type(&operand.ty)?;
                let register = self.register();
                let instruction = match operator {
                    UnaryPrimitive::Negate if scalar.floating => {
                        format!("fneg {} {}", scalar.llvm, operand.representation)
                    }
                    UnaryPrimitive::Negate => {
                        format!("sub {} 0, {}", scalar.llvm, operand.representation)
                    }
                    UnaryPrimitive::BitwiseNot if !scalar.floating => {
                        format!("xor {} {}, -1", scalar.llvm, operand.representation)
                    }
                    UnaryPrimitive::BitwiseNot => return None,
                };
                self.line(format!("  {register} = {instruction}"));
                Some(Some(EmittedValue {
                    ty: operand.ty,
                    representation: register,
                }))
            }
            Operation::PrimitiveBinary {
                operator,
                left,
                right,
            } => {
                let left = self.atom(left)?;
                let right = self.atom(right)?;
                if left.ty != right.ty {
                    return None;
                }
                let scalar = scalar_type(&left.ty)?;
                let instruction = arithmetic_instruction(*operator, scalar)?;
                let register = self.register();
                self.line(format!(
                    "  {register} = {instruction} {} {}, {}",
                    scalar.llvm, left.representation, right.representation
                ));
                Some(Some(EmittedValue {
                    ty: left.ty,
                    representation: register,
                }))
            }
            Operation::NumericConversion { operand } => {
                let operand = self.atom(operand)?;
                let source = scalar_type(&operand.ty)?;
                let result_type = result_type?.clone();
                let target = scalar_type(&result_type)?;
                if source.floating == target.floating && source.bits == target.bits {
                    return Some(Some(EmittedValue {
                        ty: result_type,
                        representation: operand.representation,
                    }));
                }
                let instruction = if source.floating && target.floating {
                    if source.bits > target.bits {
                        "fptrunc"
                    } else {
                        "fpext"
                    }
                } else if source.floating {
                    if target.signed { "fptosi" } else { "fptoui" }
                } else if target.floating {
                    if source.signed { "sitofp" } else { "uitofp" }
                } else if source.bits > target.bits {
                    "trunc"
                } else if source.signed {
                    "sext"
                } else {
                    "zext"
                };
                let register = self.register();
                self.line(format!(
                    "  {register} = {instruction} {} {} to {}",
                    source.llvm, operand.representation, target.llvm
                ));
                Some(Some(EmittedValue {
                    ty: result_type,
                    representation: register,
                }))
            }
            Operation::ExternalCall { id, argument } => self
                .emit_external_call(*id, argument, result_type?)
                .map(Some),
            Operation::Product(elements) => self.emit_product(elements, result_type?).map(Some),
            Operation::SumInjection { index, value } => {
                self.emit_sum(*index, value, result_type?).map(Some)
            }
            _ => None,
        }
    }

    fn emit_terminator(&mut self, site: StateId, terminator: &Terminator) -> Option<()> {
        match terminator {
            Terminator::Return(value) => {
                let value = self.atom(value)?;
                if self.has_frames {
                    if value.ty != self.result_type {
                        return None;
                    }
                    self.emit_frame_return(site, &value.representation)?;
                } else {
                    let value_type = value_type(&value.ty)?;
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
                if is_bool(&left.ty) {
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
                argument, resume, ..
            } => match self.execution.control_calls.mode(site)? {
                ControlCallMode::Direct(target) => {
                    let result = self.emit_call(target, argument, false)?;
                    let input = self.control.states[resume.0].input.as_ref()?;
                    self.store_pattern(input, Some(&result))?;
                    self.line(format!("  br label %mal_state_{}", resume.0));
                }
                ControlCallMode::Dispatch
                    if self.execution.control_frames.frame(site).is_some() =>
                {
                    self.emit_frame_call(site, argument)?;
                }
                ControlCallMode::DirectSelfTail | ControlCallMode::Dispatch => return None,
            },
            Terminator::TailCall { argument, .. } => {
                match self.execution.control_calls.mode(site)? {
                    ControlCallMode::DirectSelfTail => {
                        let argument = self
                            .execution
                            .tail_calls
                            .forwarded_self_argument(site)
                            .unwrap_or(argument);
                        let value = self.atom(argument)?;
                        let parameter = self.function.parameter.binding?;
                        let slot = self.slots.get(&parameter)?.clone();
                        if slot.ty != value.ty {
                            return None;
                        }
                        let value_type = value_type(&slot.ty)?;
                        self.line(format!(
                            "  store {} {}, ptr %mal_slot_{}, align {}",
                            value_type.llvm, value.representation, slot.index, value_type.alignment
                        ));
                        self.line(format!("  br label %mal_state_{}", self.function.entry.0));
                    }
                    ControlCallMode::Direct(target) => {
                        let result = self.emit_call(target, argument, true)?;
                        let value_type = value_type(&result.ty)?;
                        self.line(format!(
                            "  ret {} {}",
                            value_type.llvm, result.representation
                        ));
                    }
                    ControlCallMode::Dispatch => return None,
                }
            }
            Terminator::Case { scrutinee, arms } => self.emit_case(site, scrutinee, arms)?,
        }
        Some(())
    }

    fn emit_call(
        &mut self,
        target: FunctionId,
        argument: &Atom,
        tail: bool,
    ) -> Option<EmittedValue> {
        let target = self
            .control
            .functions
            .iter()
            .find(|function| function.id == target)?;
        let arguments = if target.parameter.ty == Type::Unit {
            "ptr %mal_context".into()
        } else {
            let argument = self.atom(argument)?;
            if argument.ty != target.parameter.ty {
                return None;
            }
            let value_type = value_type(&argument.ty)?;
            format!(
                "ptr %mal_context, {} {}",
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
        let result_value_type = value_type(&result_type)?;
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
        })
    }

    fn register(&mut self) -> String {
        let register = format!("%mal_value_{}", self.next_register);
        self.next_register += 1;
        register
    }

    fn line(&mut self, line: impl AsRef<str>) {
        self.output.push_str(line.as_ref());
        self.output.push('\n');
    }
}

fn function_name(id: FunctionId) -> Option<String> {
    match id {
        FunctionId::Lambda(id) => Some(format!("mal_function_{}", id.0)),
        FunctionId::Memory(_) => None,
    }
}
