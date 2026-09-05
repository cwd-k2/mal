use crate::closure::ast::{self as closure, TopLevelPattern};

use super::{BodyEmitter, environment_name, function_name, has_direct_tail_call, value_name};

impl BodyEmitter<'_> {
    pub(super) fn emit_environments(&self) -> String {
        let mut output = String::new();
        for function in &self.program.functions {
            if function.environment.is_empty() {
                continue;
            }
            c_line!(
                &mut output,
                0,
                "typedef struct {} {{",
                environment_name(function.id)
            );
            for (index, field) in function.environment.iter().enumerate() {
                c_line!(
                    &mut output,
                    1,
                    "{} field_{index};",
                    self.types.c_type(&field.ty)
                );
            }
            c_line!(&mut output, 0, "}} {};\n", environment_name(function.id));
        }
        output
    }

    pub(super) fn emit_globals(&self) -> String {
        let mut output = String::new();
        for binding in &self.program.bindings {
            self.emit_top_level_globals(&mut output, &binding.pattern);
        }
        if !output.is_empty() {
            output.push('\n');
        }
        output
    }

    pub(super) fn emit_function_declarations(&self) -> String {
        let mut output = String::new();
        for function in &self.program.functions {
            if let Some(name) = self.top_level_function_name(function.id) {
                c_line!(&mut output, 0, "/* mal source binding: {name} */");
            }
            c_line!(&mut output, 0, "{};", self.function_signature(function));
        }
        if !output.is_empty() {
            output.push('\n');
        }
        output
    }

    pub(super) fn emit_function_definitions(&mut self) -> String {
        let mut output = String::new();
        for function in &self.program.functions {
            if let Some(name) = self.top_level_function_name(function.id) {
                c_line!(&mut output, 0, "/* mal source binding: {name} */");
            }
            c_line!(&mut output, 0, "{} {{", self.function_signature(function));
            c_line!(&mut output, 1, "(void)mal_context;");
            if function.environment.is_empty() {
                c_line!(&mut output, 1, "(void)mal_environment;");
            } else {
                c_line!(
                    &mut output,
                    1,
                    "const {} *mal_environment_fields = (const {} *)mal_environment;",
                    environment_name(function.id),
                    environment_name(function.id)
                );
            }
            if function.parameter.binding.is_none() {
                c_line!(&mut output, 1, "(void)mal_parameter;");
            }
            if has_direct_tail_call(&function.body, function.id) {
                c_line!(&mut output, 1, "mal_tail_entry:");
                c_line!(&mut output, 1, "{{");
                let parameter_name = function
                    .parameter
                    .binding
                    .map_or_else(|| "mal_parameter".into(), value_name);
                self.emit_tail_block(&mut output, &function.body, function.id, &parameter_name, 2);
                c_line!(&mut output, 1, "}}");
            } else {
                self.emit_block_bindings(&mut output, &function.body, 1);
                c_line!(
                    &mut output,
                    1,
                    "return {};",
                    self.emit_atom(&function.body.result)
                );
            }
            output.push_str("}\n\n");
        }
        output
    }

    pub(super) fn emit_initializer(&mut self) -> String {
        let mut output =
            String::from("static void mal_program_initialize(MalContext *mal_context) {\n");
        c_line!(&mut output, 1, "(void)mal_context;");
        for binding in &self.program.bindings {
            self.emit_block_bindings(&mut output, &binding.value, 1);
            match &binding.pattern {
                TopLevelPattern::Binding { id, .. } => {
                    c_line!(
                        &mut output,
                        1,
                        "{} = {};",
                        value_name(*id),
                        self.emit_atom(&binding.value.result)
                    );
                    c_line!(&mut output, 1, "(void){};", value_name(*id));
                }
                TopLevelPattern::Wildcard { .. } => {
                    c_line!(
                        &mut output,
                        1,
                        "(void)({});",
                        self.emit_atom(&binding.value.result)
                    );
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

    pub(super) fn emit_main(&self, main: &closure::TopLevelBinding) -> String {
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
}
