use crate::check::ast::Type;
use crate::closure::ast::{self as closure, TopLevelPattern};

use super::{
    BodyEmitter, direct_function_name, environment_name, flattened_product_types,
    flattened_product_values, function_name, has_direct_product_entry, has_direct_tail_call,
    value_name,
};

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
            if has_direct_product_entry(&function.parameter.ty) {
                c_line!(
                    &mut output,
                    0,
                    "{};",
                    self.direct_function_signature(function)
                );
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
            if has_direct_product_entry(&function.parameter.ty) {
                c_line!(
                    &mut output,
                    0,
                    "{} {{",
                    self.direct_function_signature(function)
                );
                let parameter_name = function
                    .parameter
                    .binding
                    .map_or_else(|| "mal_parameter".into(), value_name);
                let mut next_parameter = 0;
                let value =
                    self.direct_parameter_value(&function.parameter.ty, &mut next_parameter);
                c_line!(
                    &mut output,
                    1,
                    "{} {parameter_name} = {value};",
                    self.types.c_type(&function.parameter.ty),
                );
                self.emit_function_body(&mut output, function);
                output.push_str("}\n\n");

                c_line!(&mut output, 0, "{} {{", self.function_signature(function));
                let arguments = flattened_product_values(&function.parameter.ty, &parameter_name)
                    .into_iter()
                    .map(|value| format!(", {value}"))
                    .collect::<String>();
                c_line!(
                    &mut output,
                    1,
                    "return {}(mal_context, mal_environment{arguments});",
                    direct_function_name(function.id)
                );
                output.push_str("}\n\n");
                continue;
            }
            c_line!(&mut output, 0, "{} {{", self.function_signature(function));
            self.emit_function_body(&mut output, function);
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
        let TopLevelPattern::Binding { id, ref ty, .. } = main.pattern else {
            unreachable!()
        };
        let name = value_name(id);
        let Type::Function { parameter, .. } = ty else {
            unreachable!("entry point validation requires a function")
        };
        if **parameter == Type::Unit {
            return format!(
                "int main(void) {{\n    MalContext mal_context = {{ NULL }};\n    MalType_Unit mal_unit = {{ UINT8_C(0) }};\n    mal_program_initialize(&mal_context);\n    int32_t mal_result = {name}.call(&mal_context, {name}.environment, mal_unit);\n    mal_context_destroy(&mal_context);\n    return (int)mal_result;\n}}\n"
            );
        }

        let parameter_type = self.types.c_type(parameter);
        format!(
            "int main(int mal_argc, char **mal_argv) {{\n    MalContext mal_context = {{ NULL }};\n    mal_program_initialize(&mal_context);\n    size_t mal_argument_count = mal_argc > 1 ? (size_t)(mal_argc - 1) : 0;\n    const size_t mal_argument_stride = sizeof(MalType_Ptr) + sizeof(uint64_t);\n    if (mal_argument_count > SIZE_MAX / mal_argument_stride) {{\n        mal_trap(&mal_context, \"argument descriptor size overflow\");\n    }}\n    uint8_t *mal_argument_storage = (uint8_t *)mal_allocate(&mal_context, mal_argument_count * mal_argument_stride);\n    for (size_t mal_index = 0; mal_index < mal_argument_count; ++mal_index) {{\n        size_t mal_length = strlen(mal_argv[mal_index + 1]);\n        if ((uint64_t)mal_length != mal_length) {{\n            mal_trap(&mal_context, \"argument length overflow\");\n        }}\n        MalType_Ptr mal_data = mal_Ptr_from_address((uint8_t *)mal_argv[mal_index + 1]);\n        uint64_t mal_length_u64 = (uint64_t)mal_length;\n        uint8_t *mal_slot = mal_argument_storage + mal_index * mal_argument_stride;\n        memcpy(mal_slot, &mal_data, sizeof(mal_data));\n        memcpy(mal_slot + sizeof(mal_data), &mal_length_u64, sizeof(mal_length_u64));\n    }}\n    {parameter_type} mal_arguments = {{ .field_0 = (uint64_t)mal_argument_count, .field_1 = mal_Ptr_from_address(mal_argument_storage) }};\n    int32_t mal_result = {name}.call(&mal_context, {name}.environment, mal_arguments);\n    mal_context_destroy(&mal_context);\n    return (int)mal_result;\n}}\n"
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

    fn direct_function_signature(&self, function: &closure::Function) -> String {
        let result = self.types.c_type(&function.body.result.ty);
        let crate::check::ast::Type::Product(_) = &function.parameter.ty else {
            unreachable!("only product parameters have direct entry points")
        };
        let parameters = flattened_product_types(&function.parameter.ty)
            .into_iter()
            .enumerate()
            .map(|(index, ty)| format!("{} mal_direct_parameter_{index}", self.types.c_type(ty)))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "static {result} {}(MalContext *mal_context, const void *mal_environment, {parameters})",
            direct_function_name(function.id)
        )
    }

    fn direct_parameter_value(
        &self,
        ty: &crate::check::ast::Type,
        next_parameter: &mut usize,
    ) -> String {
        if let crate::check::ast::Type::Product(elements) = ty {
            let fields = elements
                .iter()
                .enumerate()
                .map(|(index, element)| {
                    format!(
                        ".field_{index} = {}",
                        self.direct_parameter_value(element, next_parameter)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            return format!("({}){{ {fields} }}", self.types.c_type(ty));
        }
        let parameter = format!("mal_direct_parameter_{next_parameter}");
        *next_parameter += 1;
        parameter
    }

    fn emit_function_body(&mut self, output: &mut String, function: &closure::Function) {
        c_line!(output, 1, "(void)mal_context;");
        if function.environment.is_empty() {
            c_line!(output, 1, "(void)mal_environment;");
        } else {
            c_line!(
                output,
                1,
                "const {} *mal_environment_fields = (const {} *)mal_environment;",
                environment_name(function.id),
                environment_name(function.id)
            );
        }
        if function.parameter.binding.is_none() {
            c_line!(output, 1, "(void)mal_parameter;");
        }
        if has_direct_tail_call(&function.body, function.id) {
            c_line!(output, 1, "mal_tail_entry:");
            c_line!(output, 1, "{{");
            let parameter_name = function
                .parameter
                .binding
                .map_or_else(|| "mal_parameter".into(), value_name);
            self.emit_tail_block(output, &function.body, function.id, &parameter_name, 2);
            c_line!(output, 1, "}}");
        } else {
            self.emit_block_bindings(output, &function.body, 1);
            c_line!(
                output,
                1,
                "return {};",
                self.emit_atom(&function.body.result)
            );
        }
    }
}
