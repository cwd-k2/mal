use crate::check::ast::Type;
use crate::closure::ast::FunctionId;
use crate::closure::ast::{self as closure, Atom, Binding, Block, Operation, Pattern};

use super::{BodyEmitter, environment_name, function_name, pattern_type, value_name};
use crate::c_emit::types::is_bool;

impl BodyEmitter<'_> {
    pub(super) fn emit_block_bindings(
        &mut self,
        output: &mut String,
        block: &Block,
        indent: usize,
    ) {
        for binding in &block.bindings {
            self.emit_binding(output, binding, indent);
        }
    }

    pub(super) fn emit_tail_block(
        &mut self,
        output: &mut String,
        block: &Block,
        function: FunctionId,
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
            Some(Operation::PrimitiveBranch {
                operator,
                left,
                right,
                otherwise,
                then,
            }) => self.emit_tail_primitive_branch(
                output,
                *operator,
                left,
                right,
                otherwise,
                then,
                function,
                parameter_name,
                indent,
            ),
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
        function: FunctionId,
        parameter_name: &str,
        indent: usize,
    ) {
        let scrutinee_text = self.emit_atom(scrutinee);
        let tag = if is_bool(&scrutinee.ty) {
            scrutinee_text.clone()
        } else {
            format!("{scrutinee_text}.tag")
        };
        c_line!(output, indent, "switch ({tag}) {{");
        for arm in arms {
            c_line!(output, indent + 1, "case UINT32_C({}): {{", arm.index);
            let payload = if is_bool(&scrutinee.ty) {
                "(MalType_Unit){ UINT8_C(0) }".into()
            } else {
                format!("{scrutinee_text}.payload.variant_{}", arm.index)
            };
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

    #[allow(clippy::too_many_arguments)]
    fn emit_tail_primitive_branch(
        &mut self,
        output: &mut String,
        operator: crate::core::ast::BinaryPrimitive,
        left: &Atom,
        right: &Atom,
        otherwise: &Block,
        then: &Block,
        function: FunctionId,
        parameter_name: &str,
        indent: usize,
    ) {
        let condition = self.emit_primitive_condition(operator, left, right);
        c_line!(output, indent, "if ({condition}) {{");
        self.emit_tail_block(output, then, function, parameter_name, indent + 1);
        c_line!(output, indent, "}} else {{");
        self.emit_tail_block(output, otherwise, function, parameter_name, indent + 1);
        c_line!(output, indent, "}}");
    }

    fn emit_binding(&mut self, output: &mut String, binding: &Binding, indent: usize) {
        let ty = pattern_type(&binding.pattern);
        match &binding.operation {
            Operation::Atom(atom) if matches!(binding.pattern, Pattern::Product { .. }) => {
                let value = self.emit_atom(atom);
                self.emit_pattern_bindings(output, &binding.pattern, &value, indent);
            }
            Operation::Case { scrutinee, arms } => {
                self.emit_case(output, &binding.pattern, ty, scrutinee, arms, indent);
            }
            Operation::PrimitiveBranch {
                operator,
                left,
                right,
                otherwise,
                then,
            } => self.emit_primitive_branch(
                output,
                &binding.pattern,
                ty,
                *operator,
                left,
                right,
                otherwise,
                then,
                indent,
            ),
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
                c_line!(output, indent, "MalType_Unit {name} = {{ UINT8_C(0) }};");
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
        let bool_scrutinee = is_bool(&scrutinee.ty);
        let tag = if bool_scrutinee {
            scrutinee_text.clone()
        } else {
            format!("{scrutinee_text}.tag")
        };
        c_line!(output, indent, "switch ({tag}) {{");
        for arm in arms {
            c_line!(output, indent + 1, "case UINT32_C({}): {{", arm.index);
            if let Pattern::Binding { id, ty } = &arm.pattern {
                let name = value_name(*id);
                let payload = if bool_scrutinee {
                    "(MalType_Unit){ UINT8_C(0) }".into()
                } else {
                    format!("{scrutinee_text}.payload.variant_{}", arm.index)
                };
                c_line!(
                    output,
                    indent + 2,
                    "{} {name} = {payload};",
                    self.types.c_type(ty)
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

    #[allow(clippy::too_many_arguments)]
    fn emit_primitive_branch(
        &mut self,
        output: &mut String,
        pattern: &Pattern,
        ty: &Type,
        operator: crate::core::ast::BinaryPrimitive,
        left: &Atom,
        right: &Atom,
        otherwise: &Block,
        then: &Block,
        indent: usize,
    ) {
        let target = self.result_target(pattern);
        c_line!(output, indent, "{} {target};", self.types.c_type(ty));
        let condition = self.emit_primitive_condition(operator, left, right);
        c_line!(output, indent, "if ({condition}) {{");
        self.emit_block_bindings(output, then, indent + 1);
        c_line!(
            output,
            indent + 1,
            "{target} = {};",
            self.emit_atom(&then.result)
        );
        c_line!(output, indent, "}} else {{");
        self.emit_block_bindings(output, otherwise, indent + 1);
        c_line!(
            output,
            indent + 1,
            "{target} = {};",
            self.emit_atom(&otherwise.result)
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
