use crate::anf;
use crate::check::ast::Type;
use crate::closure::ast::{self as closure, Atom, Binding, Block, Operation, Pattern};
use crate::resolve::ast::{ExternalOperationId, LambdaId};

use super::types::TypeRegistry;

mod expression;
mod function;
mod pattern;

use self::pattern::pattern_type;

#[derive(Clone, Copy, Default)]
pub(super) struct RuntimeNeeds {
    pub(super) wrap: u16,
    pub(super) divide: u16,
    pub(super) remainder: u16,
    pub(super) shift_left: u16,
    pub(super) shift_right: u16,
    pub(super) string_equality: bool,
    pub(super) string_at: bool,
    pub(super) float_to_integer: u32,
}

pub(super) struct BodyOutput {
    pub(super) environment_declarations: String,
    pub(super) globals: String,
    pub(super) function_declarations: String,
    pub(super) function_definitions: String,
    pub(super) initializer: String,
    pub(super) main: String,
    pub(super) needs: RuntimeNeeds,
}

pub(super) struct BodyEmitter<'a> {
    program: &'a closure::Program,
    types: &'a TypeRegistry,
    needs: RuntimeNeeds,
    next_discard: u32,
}

impl<'a> BodyEmitter<'a> {
    pub(super) fn new(program: &'a closure::Program, types: &'a TypeRegistry) -> Self {
        Self {
            program,
            types,
            needs: RuntimeNeeds::default(),
            next_discard: 0,
        }
    }

    pub(super) fn emit(&mut self, main: &crate::closure::ast::TopLevelBinding) -> BodyOutput {
        let environment_declarations = self.emit_environments();
        let globals = self.emit_globals();
        let function_declarations = self.emit_function_declarations();
        let function_definitions = self.emit_function_definitions();
        let initializer = self.emit_initializer();
        let main = self.emit_main(main);
        BodyOutput {
            environment_declarations,
            globals,
            function_declarations,
            function_definitions,
            initializer,
            main,
            needs: self.needs,
        }
    }

    fn emit_block_bindings(&mut self, output: &mut String, block: &Block, indent: usize) {
        for binding in &block.bindings {
            self.emit_binding(output, binding, indent);
        }
    }

    fn emit_tail_block(
        &mut self,
        output: &mut String,
        block: &Block,
        function: LambdaId,
        parameter_name: &str,
        indent: usize,
    ) {
        let tail = block.bindings.last().filter(|binding| {
            matches!(
                (&block.result.kind, &binding.pattern),
                (
                    closure::AtomKind::Reference(closure::Reference::Binding(result)),
                    Pattern::Binding { id, .. }
                ) if result == id
            )
        });
        let ordinary_count = block.bindings.len() - usize::from(tail.is_some());
        for binding in &block.bindings[..ordinary_count] {
            self.emit_binding(output, binding, indent);
        }

        match tail.map(|binding| &binding.operation) {
            Some(Operation::Call { callee, argument })
                if matches!(
                    callee.kind,
                    closure::AtomKind::Reference(closure::Reference::SelfClosure(id))
                        if id == function
                ) =>
            {
                c_line!(
                    output,
                    indent,
                    "{parameter_name} = {};",
                    self.emit_atom(argument)
                );
                c_line!(output, indent, "goto mal_tail_entry;");
            }
            Some(Operation::Case { scrutinee, arms }) => {
                self.emit_tail_case(output, scrutinee, arms, function, parameter_name, indent)
            }
            Some(_) => {
                self.emit_binding(output, block.bindings.last().expect("tail binding"), indent);
                c_line!(output, indent, "return {};", self.emit_atom(&block.result));
            }
            None => c_line!(output, indent, "return {};", self.emit_atom(&block.result)),
        }
    }

    fn emit_tail_case(
        &mut self,
        output: &mut String,
        scrutinee: &Atom,
        arms: &[closure::CaseArm],
        function: LambdaId,
        parameter_name: &str,
        indent: usize,
    ) {
        let scrutinee_text = self.emit_atom(scrutinee);
        c_line!(output, indent, "switch ({scrutinee_text}.tag) {{");
        for arm in arms {
            c_line!(output, indent + 1, "case UINT32_C({}): {{", arm.index);
            let payload = format!("{scrutinee_text}.payload.variant_{}", arm.index);
            self.emit_simple_result(
                output,
                &arm.pattern,
                pattern_type(&arm.pattern),
                &payload,
                indent + 2,
            );
            self.emit_tail_block(output, &arm.value, function, parameter_name, indent + 2);
            c_line!(output, indent + 1, "}}");
        }
        c_line!(output, indent + 1, "default:");
        c_line!(
            output,
            indent + 2,
            "mal_trap(mal_context, \"invalid sum tag\");"
        );
        c_line!(output, indent, "}}");
    }

    fn emit_binding(&mut self, output: &mut String, binding: &Binding, indent: usize) {
        let ty = pattern_type(&binding.pattern);
        match &binding.operation {
            Operation::Case { scrutinee, arms } => {
                self.emit_case(output, &binding.pattern, ty, scrutinee, arms, indent);
            }
            Operation::MakeClosure { function, captures } => {
                self.emit_make_closure(output, &binding.pattern, ty, *function, captures, indent);
            }
            Operation::ExternalCall { id, argument } if *ty == Type::Unit => {
                let call = self.emit_external_call(*id, argument);
                c_line!(output, indent, "{call};");
                self.emit_unit_result(output, &binding.pattern, indent);
            }
            operation => {
                let expression = self.emit_operation_expression(operation, ty);
                self.emit_simple_result(output, &binding.pattern, ty, &expression, indent);
            }
        }
    }

    fn emit_simple_result(
        &mut self,
        output: &mut String,
        pattern: &Pattern,
        ty: &Type,
        expression: &str,
        indent: usize,
    ) {
        match pattern {
            Pattern::Binding { id, .. } => {
                let name = value_name(*id);
                c_line!(
                    output,
                    indent,
                    "{} {name} = {expression};",
                    self.types.c_type(ty)
                );
                c_line!(output, indent, "(void){name};");
            }
            Pattern::Wildcard { .. } => {
                c_line!(output, indent, "(void)({expression});");
            }
            Pattern::Product { .. } => {
                let target = self.result_target(pattern);
                c_line!(
                    output,
                    indent,
                    "{} {target} = {expression};",
                    self.types.c_type(ty)
                );
                c_line!(output, indent, "(void){target};");
                self.emit_pattern_bindings(output, pattern, &target, indent);
            }
        }
    }

    fn emit_unit_result(&self, output: &mut String, pattern: &Pattern, indent: usize) {
        match pattern {
            Pattern::Binding { id, .. } => {
                let name = value_name(*id);
                c_line!(output, indent, "MalUnit {name} = {{ UINT8_C(0) }};");
                c_line!(output, indent, "(void){name};");
            }
            Pattern::Wildcard { .. } => {}
            Pattern::Product { .. } => {
                unreachable!("a product pattern cannot match a Unit operation")
            }
        }
    }

    fn emit_case(
        &mut self,
        output: &mut String,
        pattern: &Pattern,
        ty: &Type,
        scrutinee: &Atom,
        arms: &[closure::CaseArm],
        indent: usize,
    ) {
        let target = self.result_target(pattern);
        c_line!(output, indent, "{} {target};", self.types.c_type(ty));
        let scrutinee_text = self.emit_atom(scrutinee);
        c_line!(output, indent, "switch ({scrutinee_text}.tag) {{");
        for arm in arms {
            c_line!(output, indent + 1, "case UINT32_C({}): {{", arm.index);
            if let Pattern::Binding { id, ty } = &arm.pattern {
                let name = value_name(*id);
                c_line!(
                    output,
                    indent + 2,
                    "{} {name} = {scrutinee_text}.payload.variant_{};",
                    self.types.c_type(ty),
                    arm.index
                );
                c_line!(output, indent + 2, "(void){name};");
            }
            self.emit_block_bindings(output, &arm.value, indent + 2);
            c_line!(
                output,
                indent + 2,
                "{target} = {};",
                self.emit_atom(&arm.value.result)
            );
            c_line!(output, indent + 2, "break;");
            c_line!(output, indent + 1, "}}");
        }
        c_line!(output, indent + 1, "default:");
        c_line!(
            output,
            indent + 2,
            "mal_trap(mal_context, \"invalid sum tag\");"
        );
        c_line!(output, indent, "}}");
        c_line!(output, indent, "(void){target};");
        if matches!(pattern, Pattern::Product { .. }) {
            self.emit_pattern_bindings(output, pattern, &target, indent);
        }
    }

    fn emit_make_closure(
        &mut self,
        output: &mut String,
        pattern: &Pattern,
        ty: &Type,
        function: LambdaId,
        captures: &[Atom],
        indent: usize,
    ) {
        let target = self.result_target(pattern);
        let environment = if captures.is_empty() {
            "NULL".into()
        } else {
            let allocation = format!("mal_new_environment_{target}");
            c_line!(
                output,
                indent,
                "{} *{allocation} = ({0} *)mal_allocate(mal_context, sizeof({0}));",
                environment_name(function)
            );
            let fields = captures
                .iter()
                .enumerate()
                .map(|(index, atom)| format!(".field_{index} = {}", self.emit_atom(atom)))
                .collect::<Vec<_>>()
                .join(", ");
            c_line!(
                output,
                indent,
                "*{allocation} = ({}){{ {fields} }};",
                environment_name(function)
            );
            allocation
        };
        c_line!(
            output,
            indent,
            "{} {target} = {{ {}, {environment} }};",
            self.types.c_type(ty),
            function_name(function)
        );
        c_line!(output, indent, "(void){target};");
    }

    fn external(&self, id: ExternalOperationId) -> &closure::ExternalOperation {
        self.program
            .externals
            .iter()
            .find(|external| external.id == id)
            .expect("closure conversion preserves external declarations")
    }

    fn result_target(&mut self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Binding { id, .. } => value_name(*id),
            Pattern::Wildcard { .. } | Pattern::Product { .. } => {
                let name = format!("mal_discard_{}", self.next_discard);
                self.next_discard += 1;
                name
            }
        }
    }
}

fn value_name(id: anf::ast::ValueId) -> String {
    match id {
        anf::ast::ValueId::Core(crate::core::ast::ValueId::Source(id)) => {
            format!("mal_value_source_{}", id.0)
        }
        anf::ast::ValueId::Core(crate::core::ast::ValueId::Temporary(id)) => {
            format!("mal_value_core_{id}")
        }
        anf::ast::ValueId::Temporary(id) => format!("mal_value_anf_{id}"),
    }
}

fn function_name(id: LambdaId) -> String {
    format!("mal_function_{}", id.0)
}

fn environment_name(id: LambdaId) -> String {
    format!("MalEnvironment_{}", id.0)
}

fn has_direct_tail_call(block: &Block, function: LambdaId) -> bool {
    let Some(binding) = tail_binding(block) else {
        return false;
    };
    match &binding.operation {
        Operation::Call { callee, .. } => matches!(
            callee.kind,
            closure::AtomKind::Reference(closure::Reference::SelfClosure(id)) if id == function
        ),
        Operation::Case { arms, .. } => arms
            .iter()
            .any(|arm| has_direct_tail_call(&arm.value, function)),
        _ => false,
    }
}

fn tail_binding(block: &Block) -> Option<&Binding> {
    block.bindings.last().filter(|binding| {
        matches!(
            (&block.result.kind, &binding.pattern),
            (
                closure::AtomKind::Reference(closure::Reference::Binding(result)),
                Pattern::Binding { id, .. }
            ) if result == id
        )
    })
}
