use std::fmt::Write;

use crate::anf;
use crate::check::ast::Type;
use crate::closure::ast::{
    self as closure, Atom, Binding, Block, Operation, Pattern, TopLevelPattern,
};
use crate::resolve::ast::{ExternalOperationId, LambdaId};

use super::types::TypeRegistry;

mod expression;
mod pattern;

use self::pattern::pattern_type;

#[derive(Clone, Copy, Default)]
pub(super) struct RuntimeNeeds {
    pub(super) allocation: bool,
    pub(super) wrap: u16,
    pub(super) divide: u16,
    pub(super) remainder: u16,
    pub(super) shift_left: u16,
    pub(super) shift_right: u16,
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

    fn emit_environments(&self) -> String {
        let mut output = String::new();
        for function in &self.program.functions {
            if function.environment.is_empty() {
                continue;
            }
            writeln!(
                output,
                "typedef struct {} {{",
                environment_name(function.id)
            )
            .unwrap();
            for (index, field) in function.environment.iter().enumerate() {
                writeln!(
                    output,
                    "    {} field_{index};",
                    self.types.c_type(&field.ty)
                )
                .unwrap();
            }
            writeln!(output, "}} {};\n", environment_name(function.id)).unwrap();
        }
        output
    }

    fn emit_globals(&self) -> String {
        let mut output = String::new();
        for binding in &self.program.bindings {
            self.emit_top_level_globals(&mut output, &binding.pattern);
        }
        if !output.is_empty() {
            output.push('\n');
        }
        output
    }

    fn emit_function_declarations(&self) -> String {
        let mut output = String::new();
        for function in &self.program.functions {
            writeln!(output, "{};", self.function_signature(function)).unwrap();
        }
        if !output.is_empty() {
            output.push('\n');
        }
        output
    }

    fn emit_function_definitions(&mut self) -> String {
        let mut output = String::new();
        for function in &self.program.functions {
            writeln!(output, "{} {{", self.function_signature(function)).unwrap();
            line(&mut output, 1, "(void)mal_context;");
            match function.environment.is_empty() {
                true => line(&mut output, 1, "(void)mal_environment;"),
                false => {
                    writeln!(
                        output,
                        "    const {} *mal_environment_fields = (const {} *)mal_environment;",
                        environment_name(function.id),
                        environment_name(function.id)
                    )
                    .unwrap();
                }
            }
            if function.parameter.binding.is_none() {
                line(&mut output, 1, "(void)mal_parameter;");
            }
            self.emit_block_bindings(&mut output, &function.body, 1);
            writeln!(
                output,
                "    return {};",
                self.emit_atom(&function.body.result)
            )
            .unwrap();
            output.push_str("}\n\n");
        }
        output
    }

    fn emit_initializer(&mut self) -> String {
        let mut output =
            String::from("static void mal_program_initialize(MalContext *mal_context) {\n");
        line(&mut output, 1, "(void)mal_context;");
        for binding in &self.program.bindings {
            self.emit_block_bindings(&mut output, &binding.value, 1);
            match &binding.pattern {
                TopLevelPattern::Binding { id, .. } => {
                    writeln!(
                        output,
                        "    {} = {};",
                        value_name(*id),
                        self.emit_atom(&binding.value.result)
                    )
                    .unwrap();
                    writeln!(output, "    (void){};", value_name(*id)).unwrap();
                }
                TopLevelPattern::Wildcard { .. } => {
                    writeln!(
                        output,
                        "    (void)({});",
                        self.emit_atom(&binding.value.result)
                    )
                    .unwrap();
                }
                TopLevelPattern::Product { .. } => self.emit_top_level_pattern(
                    &mut output,
                    &binding.pattern,
                    &self.emit_atom(&binding.value.result),
                    1,
                ),
            }
        }
        output.push_str("}\n\n");
        output
    }

    fn emit_main(&self, main: &crate::closure::ast::TopLevelBinding) -> String {
        let TopLevelPattern::Binding { id, .. } = main.pattern else {
            unreachable!()
        };
        let name = value_name(id);
        format!(
            "int main(void) {{\n    MalContext mal_context = {{ NULL }};\n    MalUnit mal_unit = {{ UINT8_C(0) }};\n    mal_program_initialize(&mal_context);\n    int32_t mal_result = {name}.call(&mal_context, {name}.environment, mal_unit);\n    mal_context_destroy(&mal_context);\n    return (int)mal_result;\n}}\n"
        )
    }

    fn function_signature(&self, function: &closure::Function) -> String {
        let result = self.types.c_type(&function.body.result.ty);
        let parameter_type = self.types.c_type(&function.parameter.ty);
        let parameter_name = function
            .parameter
            .binding
            .map_or_else(|| "mal_parameter".into(), value_name);
        format!(
            "static {result} {}(MalContext *mal_context, const void *mal_environment, {parameter_type} {parameter_name})",
            function_name(function.id)
        )
    }

    fn emit_block_bindings(&mut self, output: &mut String, block: &Block, indent: usize) {
        for binding in &block.bindings {
            self.emit_binding(output, binding, indent);
        }
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
                let external = self.external(*id);
                let call = if external.parameter == Type::Unit {
                    format!("mal_ext_{}(mal_context)", external.name)
                } else {
                    format!(
                        "mal_ext_{}(mal_context, {})",
                        external.name,
                        self.emit_atom(argument)
                    )
                };
                line(output, indent, &format!("{call};"));
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
                line(
                    output,
                    indent,
                    &format!("{} {name} = {expression};", self.types.c_type(ty)),
                );
                line(output, indent, &format!("(void){name};"));
            }
            Pattern::Wildcard { .. } => {
                line(output, indent, &format!("(void)({expression});"));
            }
            Pattern::Product { .. } => {
                let target = self.result_target(pattern);
                line(
                    output,
                    indent,
                    &format!("{} {target} = {expression};", self.types.c_type(ty)),
                );
                line(output, indent, &format!("(void){target};"));
                self.emit_pattern_bindings(output, pattern, &target, indent);
            }
        }
    }

    fn emit_unit_result(&self, output: &mut String, pattern: &Pattern, indent: usize) {
        match pattern {
            Pattern::Binding { id, .. } => {
                let name = value_name(*id);
                line(
                    output,
                    indent,
                    &format!("MalUnit {name} = {{ UINT8_C(0) }};"),
                );
                line(output, indent, &format!("(void){name};"));
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
        line(
            output,
            indent,
            &format!("{} {target};", self.types.c_type(ty)),
        );
        let scrutinee_text = self.emit_atom(scrutinee);
        line(output, indent, &format!("switch ({scrutinee_text}.tag) {{"));
        for arm in arms {
            line(
                output,
                indent + 1,
                &format!("case UINT32_C({}): {{", arm.index),
            );
            if let Pattern::Binding { id, ty } = &arm.pattern {
                let name = value_name(*id);
                line(
                    output,
                    indent + 2,
                    &format!(
                        "{} {name} = {scrutinee_text}.payload.variant_{};",
                        self.types.c_type(ty),
                        arm.index
                    ),
                );
                line(output, indent + 2, &format!("(void){name};"));
            }
            self.emit_block_bindings(output, &arm.value, indent + 2);
            line(
                output,
                indent + 2,
                &format!("{target} = {};", self.emit_atom(&arm.value.result)),
            );
            line(output, indent + 2, "break;");
            line(output, indent + 1, "}");
        }
        line(output, indent + 1, "default:");
        line(
            output,
            indent + 2,
            "mal_trap(mal_context, \"invalid sum tag\");",
        );
        line(output, indent, "}");
        line(output, indent, &format!("(void){target};"));
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
            self.needs.allocation = true;
            let allocation = format!("mal_new_environment_{target}");
            line(
                output,
                indent,
                &format!(
                    "{} *{allocation} = ({0} *)mal_allocate(mal_context, sizeof({0}));",
                    environment_name(function)
                ),
            );
            let fields = captures
                .iter()
                .enumerate()
                .map(|(index, atom)| format!(".field_{index} = {}", self.emit_atom(atom)))
                .collect::<Vec<_>>()
                .join(", ");
            line(
                output,
                indent,
                &format!(
                    "*{allocation} = ({}){{ {fields} }};",
                    environment_name(function)
                ),
            );
            allocation
        };
        line(
            output,
            indent,
            &format!(
                "{} {target} = {{ {}, {environment} }};",
                self.types.c_type(ty),
                function_name(function)
            ),
        );
        line(output, indent, &format!("(void){target};"));
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

fn line(output: &mut String, indent: usize, text: &str) {
    writeln!(output, "{}{text}", "    ".repeat(indent)).unwrap();
}
