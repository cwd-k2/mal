use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, FunctionId, Pattern, Reference, TopLevelPattern};
use crate::control::ast::{Operation, Program, StateId, Terminator};
use crate::core::ast::{BinaryPrimitive, UnaryPrimitive};
use crate::execution::ControlCallMode;

mod frame;

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

fn main_function(execution: &crate::execution::Program) -> Option<FunctionId> {
    let binding = execution.lowered.bindings.iter().find(|binding| {
        matches!(&binding.pattern, TopLevelPattern::Binding { name, .. } if name == "main")
    })?;
    let TopLevelPattern::Binding { ty, .. } = &binding.pattern else {
        return None;
    };
    if *ty
        != (Type::Function {
            parameter: Box::new(Type::Unit),
            result: Box::new(Type::Int32),
        })
    {
        return None;
    }
    closure_binding_function(binding)
}

fn top_levels_are_capture_free_closures(execution: &crate::execution::Program) -> bool {
    execution.lowered.bindings.iter().all(|binding| {
        closure_binding_function(binding).is_some_and(|function| {
            execution
                .lowered
                .functions
                .iter()
                .find(|candidate| candidate.id == function)
                .is_some_and(|function| function.environment.is_empty())
        })
    })
}

fn closure_binding_function(binding: &crate::closure::ast::TopLevelBinding) -> Option<FunctionId> {
    let AtomKind::Reference(Reference::Binding(result)) = binding.value.result.kind else {
        return None;
    };
    binding.value.bindings.iter().find_map(|binding| {
        let Pattern::Binding { id, .. } = binding.pattern else {
            return None;
        };
        match &binding.operation {
            crate::closure::ast::Operation::MakeClosure { function, captures }
                if id == result && captures.is_empty() =>
            {
                Some(*function)
            }
            _ => None,
        }
    })
}

struct FunctionEmitter<'a> {
    execution: &'a crate::execution::Program,
    control: &'a Program,
    function: &'a crate::control::ast::Function,
    states: Vec<StateId>,
    slots: HashMap<ValueId, usize>,
    has_frames: bool,
    next_register: usize,
    output: String,
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
            || function_result_type(lowered) != Some(&Type::Int32)
        {
            return None;
        }
        let states = reachable_states(&execution.control, function.entry);
        let mut slots = HashMap::new();
        if let Some(id) = function.parameter.binding {
            insert_slot(&mut slots, id);
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
            states,
            slots,
            has_frames: !frame_sites.is_empty(),
            next_register: 0,
            output: String::new(),
        })
    }

    fn emit(mut self) -> Option<String> {
        let parameter: String = match self.function.parameter.ty {
            Type::Unit => "ptr %mal_context".into(),
            Type::Int32 => "ptr %mal_context, i32 %mal_parameter".into(),
            _ => return None,
        };
        self.line(format!(
            "define internal i32 @{}({parameter}) {{",
            function_name(self.function.id)?
        ));
        self.line("entry:");
        let mut slots = self.slots.values().copied().collect::<Vec<_>>();
        slots.sort_unstable();
        for slot in slots {
            self.line(format!("  %mal_slot_{slot} = alloca i32, align 4"));
        }
        if self.has_frames {
            self.line("  %mal_control_top = alloca i64, align 8");
        }
        if let Some(id) = self.function.parameter.binding {
            let slot = self.slots[&id];
            self.line(format!(
                "  store i32 %mal_parameter, ptr %mal_slot_{slot}, align 4"
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
            let value = self.emit_operation(&binding.operation)?;
            self.store_pattern(&binding.pattern, value.as_deref())?;
        }
        self.emit_terminator(site, &state.terminator)
    }

    fn emit_operation(&mut self, operation: &Operation) -> Option<Option<String>> {
        match operation {
            Operation::Atom(atom) => self.atom(atom).map(Some),
            Operation::PrimitiveUnary { operator, operand } => {
                let operand = self.atom(operand)?;
                let register = self.register();
                let instruction = match operator {
                    UnaryPrimitive::Negate => format!("sub i32 0, {operand}"),
                    UnaryPrimitive::BitwiseNot => format!("xor i32 {operand}, -1"),
                };
                self.line(format!("  {register} = {instruction}"));
                Some(Some(register))
            }
            Operation::PrimitiveBinary {
                operator,
                left,
                right,
            } => {
                let instruction = arithmetic_instruction(*operator)?;
                let left = self.atom(left)?;
                let right = self.atom(right)?;
                let register = self.register();
                self.line(format!("  {register} = {instruction} i32 {left}, {right}"));
                Some(Some(register))
            }
            _ => None,
        }
    }

    fn emit_terminator(&mut self, site: StateId, terminator: &Terminator) -> Option<()> {
        match terminator {
            Terminator::Return(value) => {
                let value = self.atom(value)?;
                if self.has_frames {
                    self.emit_frame_return(site, &value)?;
                } else {
                    self.line(format!("  ret i32 {value}"));
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
                let condition = self.register();
                self.line(format!(
                    "  {condition} = icmp {predicate} i32 {left}, {right}"
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
                        let slot = self.slots[&parameter];
                        self.line(format!(
                            "  store i32 {value}, ptr %mal_slot_{slot}, align 4"
                        ));
                        self.line(format!("  br label %mal_state_{}", self.function.entry.0));
                    }
                    ControlCallMode::Direct(target) => {
                        let result = self.emit_call(target, argument, true)?;
                        self.line(format!("  ret i32 {result}"));
                    }
                    ControlCallMode::Dispatch => return None,
                }
            }
            Terminator::Case { .. } => return None,
        }
        Some(())
    }

    fn emit_call(&mut self, target: FunctionId, argument: &Atom, tail: bool) -> Option<String> {
        let target = self
            .control
            .functions
            .iter()
            .find(|function| function.id == target)?;
        let arguments = match target.parameter.ty {
            Type::Unit => "ptr %mal_context".into(),
            Type::Int32 => format!("ptr %mal_context, i32 {}", self.atom(argument)?),
            _ => return None,
        };
        let register = self.register();
        let tail = if tail { "tail " } else { "" };
        self.line(format!(
            "  {register} = {tail}call i32 @{}({arguments})",
            function_name(target.id)?
        ));
        Some(register)
    }

    fn atom(&mut self, atom: &Atom) -> Option<String> {
        match (&atom.ty, &atom.kind) {
            (Type::Int32, AtomKind::Integer(value)) => Some((*value as i32).to_string()),
            (Type::Int32, AtomKind::Reference(Reference::Binding(id))) => {
                let slot = self.slots.get(id).copied()?;
                let register = self.register();
                self.line(format!(
                    "  {register} = load i32, ptr %mal_slot_{slot}, align 4"
                ));
                Some(register)
            }
            (Type::Unit, AtomKind::Unit) => Some(String::new()),
            _ => None,
        }
    }

    fn store_pattern(&mut self, pattern: &Pattern, value: Option<&str>) -> Option<()> {
        match pattern {
            Pattern::Binding {
                id,
                ty: Type::Int32,
            } => {
                let slot = self.slots[id];
                self.line(format!(
                    "  store i32 {}, ptr %mal_slot_{slot}, align 4",
                    value?
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

fn function_result_type(function: &crate::closure::ast::Function) -> Option<&Type> {
    Some(&function.body.result.ty)
}

fn is_scalar_parameter(ty: &Type) -> bool {
    matches!(ty, Type::Unit | Type::Int32)
}

fn collect_pattern_slot(pattern: &Pattern, slots: &mut HashMap<ValueId, usize>) -> Option<()> {
    match pattern {
        Pattern::Binding {
            id,
            ty: Type::Int32,
        } => insert_slot(slots, *id),
        Pattern::Binding { ty: Type::Unit, .. } | Pattern::Wildcard { .. } => {}
        _ => return None,
    }
    Some(())
}

fn insert_slot(slots: &mut HashMap<ValueId, usize>, id: ValueId) {
    if !slots.contains_key(&id) {
        slots.insert(id, slots.len());
    }
}

fn arithmetic_instruction(operator: BinaryPrimitive) -> Option<&'static str> {
    match operator {
        BinaryPrimitive::Multiply => Some("mul"),
        BinaryPrimitive::Add => Some("add"),
        BinaryPrimitive::Subtract => Some("sub"),
        BinaryPrimitive::BitwiseAnd => Some("and"),
        BinaryPrimitive::BitwiseXor => Some("xor"),
        BinaryPrimitive::BitwiseOr => Some("or"),
        _ => None,
    }
}

fn comparison_predicate(operator: BinaryPrimitive) -> Option<&'static str> {
    match operator {
        BinaryPrimitive::Less => Some("slt"),
        BinaryPrimitive::LessEqual => Some("sle"),
        BinaryPrimitive::Greater => Some("sgt"),
        BinaryPrimitive::GreaterEqual => Some("sge"),
        BinaryPrimitive::Equal => Some("eq"),
        BinaryPrimitive::NotEqual => Some("ne"),
        _ => None,
    }
}

fn function_name(id: FunctionId) -> Option<String> {
    match id {
        FunctionId::Lambda(id) => Some(format!("mal_function_{}", id.0)),
        FunctionId::Memory(_) => None,
    }
}

fn reachable_states(program: &Program, entry: StateId) -> Vec<StateId> {
    let mut pending = vec![entry];
    let mut seen = HashSet::new();
    let mut states = Vec::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        states.push(id);
        match &program.states[id.0].terminator {
            Terminator::Return(_) | Terminator::TailCall { .. } => {}
            Terminator::Goto(target) | Terminator::Jump { target, .. } => pending.push(*target),
            Terminator::Call { resume, .. } => pending.push(*resume),
            Terminator::Case { arms, .. } => pending.extend(arms.iter().map(|arm| arm.target)),
            Terminator::PrimitiveBranch {
                otherwise, then, ..
            } => pending.extend([*otherwise, *then]),
        }
    }
    states
}
