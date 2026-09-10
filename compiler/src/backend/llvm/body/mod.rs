use std::collections::HashMap;

use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, FunctionId, Pattern, Reference};
use crate::control::ast::{Operation, Program, StateId, Terminator};
use crate::core::ast::UnaryPrimitive;
use crate::execution::ControlCallMode;

mod frame;
mod plan;
mod scalar;

use plan::{
    collect_pattern_slot, insert_slot, main_function, pattern_value_type, reachable_states,
    top_levels_are_capture_free_closures,
};
use scalar::{arithmetic_instruction, comparison_predicate, integer_literal, scalar_type};

pub(super) struct Output {
    pub(super) definitions: String,
    pub(super) main: FunctionId,
    pub(super) uses_control: bool,
}

pub(super) fn generate(execution: &crate::execution::Program) -> Option<Output> {
    let main = main_function(execution)?;
    if !execution.lowered.interface.externals.is_empty()
        || !top_levels_are_capture_free_closures(execution)
    {
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
            || !is_scalar_parameter(&function.parameter.ty)
            || scalar_type(&lowered.body.result.ty).is_none()
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
                    .any(|field| field.managed || field.value.ty != Type::Int32)
        }) {
            return None;
        }
        Some(Self {
            execution,
            control: &execution.control,
            function,
            result_type: lowered.body.result.ty.clone(),
            states,
            slots,
            has_frames: !frame_sites.is_empty(),
            next_register: 0,
            output: String::new(),
        })
    }

    fn emit(mut self) -> Option<String> {
        let parameter = scalar_type(&self.function.parameter.ty).map_or_else(
            || Some("ptr %mal_context".to_string()),
            |scalar| Some(format!("ptr %mal_context, {} %mal_parameter", scalar.llvm)),
        )?;
        let result = scalar_type(&self.result_type)?;
        self.line(format!(
            "define internal {} @{}({parameter}) {{",
            result.llvm,
            function_name(self.function.id)?
        ));
        self.line("entry:");
        let mut slots = self.slots.values().cloned().collect::<Vec<_>>();
        slots.sort_by_key(|slot| slot.index);
        for slot in slots {
            let scalar = scalar_type(&slot.ty)?;
            self.line(format!(
                "  %mal_slot_{} = alloca {}, align {}",
                slot.index, scalar.llvm, scalar.alignment
            ));
        }
        if self.has_frames {
            self.line("  %mal_control_top = alloca i64, align 8");
        }
        if let Some(id) = self.function.parameter.binding {
            let slot = self.slots.get(&id)?.clone();
            let scalar = scalar_type(&slot.ty)?;
            self.line(format!(
                "  store {} %mal_parameter, ptr %mal_slot_{}, align {}",
                scalar.llvm, slot.index, scalar.alignment
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
                    UnaryPrimitive::Negate => {
                        format!("sub {} 0, {}", scalar.llvm, operand.representation)
                    }
                    UnaryPrimitive::BitwiseNot => {
                        format!("xor {} {}, -1", scalar.llvm, operand.representation)
                    }
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
                let instruction = arithmetic_instruction(*operator, scalar.signed)?;
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
                if source.bits == target.bits {
                    return Some(Some(EmittedValue {
                        ty: result_type,
                        representation: operand.representation,
                    }));
                }
                let instruction = if source.bits > target.bits {
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
            _ => None,
        }
    }

    fn emit_terminator(&mut self, site: StateId, terminator: &Terminator) -> Option<()> {
        match terminator {
            Terminator::Return(value) => {
                let value = self.atom(value)?;
                if self.has_frames {
                    if value.ty != Type::Int32 {
                        return None;
                    }
                    self.emit_frame_return(site, &value.representation)?;
                } else {
                    let scalar = scalar_type(&value.ty)?;
                    self.line(format!("  ret {} {}", scalar.llvm, value.representation));
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
                let predicate = comparison_predicate(*operator)?;
                let left = self.atom(left)?;
                let right = self.atom(right)?;
                if left.ty != right.ty {
                    return None;
                }
                let scalar = scalar_type(&left.ty)?;
                let predicate = predicate.for_signedness(scalar.signed);
                let condition = self.register();
                self.line(format!(
                    "  {condition} = icmp {predicate} {} {}, {}",
                    scalar.llvm, left.representation, right.representation
                ));
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
                        let scalar = scalar_type(&slot.ty)?;
                        self.line(format!(
                            "  store {} {}, ptr %mal_slot_{}, align {}",
                            scalar.llvm, value.representation, slot.index, scalar.alignment
                        ));
                        self.line(format!("  br label %mal_state_{}", self.function.entry.0));
                    }
                    ControlCallMode::Direct(target) => {
                        let result = self.emit_call(target, argument, true)?;
                        let scalar = scalar_type(&result.ty)?;
                        self.line(format!("  ret {} {}", scalar.llvm, result.representation));
                    }
                    ControlCallMode::Dispatch => return None,
                }
            }
            Terminator::Case { .. } => return None,
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
            let scalar = scalar_type(&argument.ty)?;
            format!(
                "ptr %mal_context, {} {}",
                scalar.llvm, argument.representation
            )
        };
        let lowered = self
            .execution
            .lowered
            .functions
            .iter()
            .find(|function| function.id == target.id)?;
        let result_type = lowered.body.result.ty.clone();
        let result_scalar = scalar_type(&result_type)?;
        let register = self.register();
        let tail = if tail { "tail " } else { "" };
        self.line(format!(
            "  {register} = {tail}call {} @{}({arguments})",
            result_scalar.llvm,
            function_name(target.id)?
        ));
        Some(EmittedValue {
            ty: result_type,
            representation: register,
        })
    }

    fn atom(&mut self, atom: &Atom) -> Option<EmittedValue> {
        match (&atom.ty, &atom.kind) {
            (ty, AtomKind::Integer(value)) if scalar_type(ty).is_some() => Some(EmittedValue {
                ty: ty.clone(),
                representation: integer_literal(ty, *value)?,
            }),
            (ty, AtomKind::Reference(Reference::Binding(id))) if scalar_type(ty).is_some() => {
                let slot = self.slots.get(id)?.clone();
                if slot.ty != *ty {
                    return None;
                }
                let scalar = scalar_type(ty)?;
                let register = self.register();
                self.line(format!(
                    "  {register} = load {}, ptr %mal_slot_{}, align {}",
                    scalar.llvm, slot.index, scalar.alignment
                ));
                Some(EmittedValue {
                    ty: ty.clone(),
                    representation: register,
                })
            }
            (Type::Unit, AtomKind::Unit) => Some(EmittedValue {
                ty: Type::Unit,
                representation: String::new(),
            }),
            _ => None,
        }
    }

    fn store_pattern(&mut self, pattern: &Pattern, value: Option<&EmittedValue>) -> Option<()> {
        match pattern {
            Pattern::Binding { id, ty } if scalar_type(ty).is_some() => {
                let value = value?;
                if value.ty != *ty {
                    return None;
                }
                let slot = self.slots.get(id)?.clone();
                let scalar = scalar_type(ty)?;
                self.line(format!(
                    "  store {} {}, ptr %mal_slot_{}, align {}",
                    scalar.llvm, value.representation, slot.index, scalar.alignment
                ));
            }
            Pattern::Binding { ty: Type::Unit, .. } | Pattern::Wildcard { .. } => {}
            _ => return None,
        }
        Some(())
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

fn is_scalar_parameter(ty: &Type) -> bool {
    *ty == Type::Unit || scalar_type(ty).is_some()
}

fn function_name(id: FunctionId) -> Option<String> {
    match id {
        FunctionId::Lambda(id) => Some(format!("mal_function_{}", id.0)),
        FunctionId::Memory(_) => None,
    }
}
