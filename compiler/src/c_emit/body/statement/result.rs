use crate::check::ast::Type;
use crate::closure::ast::{Atom, FunctionId, Pattern};

use super::super::{BodyEmitter, environment_name, function_name, value_name};

impl BodyEmitter<'_> {
    pub(super) fn emit_simple_result(
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
            Pattern::Wildcard { .. } => c_line!(output, indent, "(void)({expression});"),
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

    pub(super) fn emit_unit_result(&self, output: &mut String, pattern: &Pattern, indent: usize) {
        match pattern {
            Pattern::Binding { id, .. } => {
                let name = value_name(*id);
                c_line!(output, indent, "MalType_Unit {name} = {{ UINT8_C(0) }};");
                c_line!(output, indent, "(void){name};");
            }
            Pattern::Wildcard { .. } => {}
            Pattern::Product { .. } => {
                unreachable!("a product pattern cannot match a Unit operation")
            }
        }
    }

    pub(super) fn emit_make_closure(
        &mut self,
        output: &mut String,
        pattern: &Pattern,
        ty: &Type,
        function: FunctionId,
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
}
