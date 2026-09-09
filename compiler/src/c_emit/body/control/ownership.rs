use crate::c_emit::syntax::{Block, Expr, FunctionDefinition, Initializer, Statement, TypeName};
use crate::check::ast::Type;
use crate::closure::ast::{self as closure, Pattern};
use crate::control::ast as control;

use super::super::{
    BodyEmitter, ResultOwnership, environment_destroy_name, environment_name, function_name,
    has_direct_product_entry, pattern_type, value_name,
};
use super::frame_field_name;
use super::support::{closure_operation, uint8};

impl BodyEmitter<'_> {
    pub(super) fn emit_control_frame_field_moves(
        &self,
        output: &mut Block,
        site: control::StateId,
        frame_variable: &str,
    ) {
        let frame = self
            .control_frames
            .frame(site)
            .expect("dispatching non-tail calls have frames");
        for (index, field) in frame.fields.iter().enumerate() {
            let source = Expr::identifier(value_name(field.value.id));
            output.push(Statement::assignment(
                Expr::identifier(frame_variable).pointer_field(frame_field_name(index)),
                source.clone(),
            ));
            if field.managed {
                output.push(Statement::assignment(
                    source,
                    zero_value(self, &field.value.ty),
                ));
            }
        }
    }

    pub(super) fn emit_control_frame_field_restores(
        &self,
        output: &mut Block,
        site: control::StateId,
        frame_variable: &str,
    ) {
        let frame = self
            .control_frames
            .frame(site)
            .expect("dispatching non-tail calls have frames");
        for (index, field) in frame.fields.iter().enumerate() {
            let source = Expr::identifier(frame_variable).pointer_field(frame_field_name(index));
            output.push(Statement::assignment(
                Expr::identifier(value_name(field.value.id)),
                source.clone(),
            ));
            output.push(Statement::assignment(
                source,
                zero_value(self, &field.value.ty),
            ));
        }
    }

    pub(super) fn emit_control_binding(&mut self, output: &mut Block, binding: &control::Binding) {
        if let control::Operation::MakeClosure { function, captures } = &binding.operation {
            self.emit_control_make_closure(output, &binding.pattern, *function, captures);
            return;
        }
        let operation = closure_operation(&binding.operation);
        if let closure::Operation::ExternalCall { id, argument } = &operation
            && pattern_type(&binding.pattern) == &Type::Unit
        {
            output.push(Statement::expression(
                self.emit_external_call(*id, argument),
            ));
            self.emit_control_owned_pattern_assignment(
                output,
                &binding.pattern,
                Expr::compound_literal("MalType_Unit", [Initializer::positional(uint8(0))]),
            );
            return;
        }
        let emitted = self.emit_operation_expression(&operation, pattern_type(&binding.pattern));
        if emitted.ownership == ResultOwnership::Owned {
            self.emit_control_owned_pattern_assignment(
                output,
                &binding.pattern,
                emitted.expression,
            );
        } else {
            self.emit_control_borrowed_pattern_assignment(
                output,
                &binding.pattern,
                emitted.expression,
            );
        }
    }

    fn emit_control_make_closure(
        &mut self,
        output: &mut Block,
        pattern: &Pattern,
        function: closure::FunctionId,
        captures: &[closure::Atom],
    ) {
        let environment = if captures.is_empty() {
            Expr::identifier("NULL")
        } else {
            let allocation = format!("mal_control_environment_{}", self.next_discard);
            self.next_discard += 1;
            let environment_type = environment_name(function);
            output.push(Statement::variable(
                TypeName::named(environment_type.clone()).pointer(),
                &allocation,
                Some(Expr::cast(
                    TypeName::named(environment_type.clone()).pointer(),
                    Expr::named_call(
                        "mal_allocate",
                        [
                            Expr::identifier("mal_context"),
                            Expr::sizeof_type(environment_type.clone()),
                        ],
                    ),
                )),
            ));
            output.push(Statement::assignment(
                Expr::dereference(Expr::identifier(&allocation)),
                Expr::compound_literal(
                    environment_type,
                    captures.iter().enumerate().map(|(index, capture)| {
                        Initializer::designated(
                            format!("field_{index}"),
                            self.types.copy_value(&capture.ty, self.emit_atom(capture)),
                        )
                    }),
                ),
            ));
            Expr::identifier(allocation)
        };
        let value = Expr::compound_literal(
            self.types.c_type(pattern_type(pattern)),
            [
                Initializer::positional(Expr::identifier(function_name(function))),
                Initializer::positional(environment),
                Initializer::positional(if captures.is_empty() {
                    Expr::identifier("NULL")
                } else {
                    Expr::identifier(environment_destroy_name(function))
                }),
            ],
        );
        self.emit_control_owned_pattern_assignment(output, pattern, value);
    }

    pub(super) fn emit_control_owned_pattern_assignment(
        &mut self,
        output: &mut Block,
        pattern: &Pattern,
        value: Expr,
    ) {
        if matches!(pattern, Pattern::Product { .. }) {
            let temporary = format!("mal_control_owned_{}", self.next_discard);
            self.next_discard += 1;
            output.push(Statement::variable(
                self.types.c_type(pattern_type(pattern)),
                &temporary,
                Some(value),
            ));
            self.emit_control_owned_pattern_fields(output, pattern, Expr::identifier(temporary));
        } else {
            self.emit_control_owned_pattern_fields(output, pattern, value);
        }
    }

    fn emit_control_owned_pattern_fields(
        &self,
        output: &mut Block,
        pattern: &Pattern,
        value: Expr,
    ) {
        match pattern {
            Pattern::Binding { id, .. } => output.push(Statement::assignment(
                Expr::identifier(value_name(*id)),
                value,
            )),
            Pattern::Wildcard { ty, .. } if self.types.contains_managed(ty) => {
                self.types.destroy_value(output, ty, value);
            }
            Pattern::Wildcard { .. } => {
                output.push(Statement::expression(Expr::cast("void", value)))
            }
            Pattern::Product { elements, .. } => {
                for (index, element) in elements.iter().enumerate() {
                    self.emit_control_owned_pattern_fields(
                        output,
                        element,
                        value.clone().field(format!("field_{index}")),
                    );
                }
            }
        }
    }

    pub(super) fn emit_control_borrowed_pattern_assignment(
        &mut self,
        output: &mut Block,
        pattern: &Pattern,
        value: Expr,
    ) {
        match pattern {
            Pattern::Product { elements, .. } => {
                for (index, element) in elements.iter().enumerate() {
                    self.emit_control_borrowed_pattern_assignment(
                        output,
                        element,
                        value.clone().field(format!("field_{index}")),
                    );
                }
            }
            Pattern::Binding { .. } => {
                let value = self.types.copy_value(pattern_type(pattern), value);
                self.emit_control_owned_pattern_assignment(output, pattern, value);
            }
            Pattern::Wildcard { .. } => {
                output.push(Statement::expression(Expr::cast("void", value)))
            }
        }
    }

    pub(super) fn emit_control_activation_cleanup(
        &self,
        output: &mut Block,
        function: &closure::Function,
        local_slots: &[(crate::anf::ast::ValueId, Type)],
    ) {
        for (id, ty) in local_slots.iter().rev() {
            if self.types.contains_managed(ty) {
                let target = Expr::identifier(value_name(*id));
                self.types.destroy_value(output, ty, target.clone());
                output.push(Statement::assignment(target, zero_value(self, ty)));
            }
        }
        if let Some(parameter) = function.parameter.binding
            && self.types.contains_managed(&function.parameter.ty)
        {
            let target = Expr::identifier(value_name(parameter));
            self.types
                .destroy_value(output, &function.parameter.ty, target.clone());
            output.push(Statement::assignment(
                target,
                zero_value(self, &function.parameter.ty),
            ));
        }
    }

    pub(super) fn emit_owned_control_wrapper(
        &self,
        function: &closure::Function,
    ) -> FunctionDefinition {
        let mut body = Block::default();
        let parameter = if has_direct_product_entry(&function.parameter.ty) {
            let mut next_parameter = 0;
            self.direct_parameter_value(&function.parameter.ty, &mut next_parameter)
        } else {
            Expr::identifier(
                function
                    .parameter
                    .binding
                    .map_or_else(|| "mal_parameter".into(), value_name),
            )
        };
        let result = "mal_owned_control_result";
        body.push(Statement::variable(
            self.types.c_type(&function.body.result.ty),
            result,
            Some(Expr::named_call(
                function_name(function.id),
                [
                    Expr::identifier("mal_context"),
                    Expr::identifier("mal_environment"),
                    parameter.clone(),
                ],
            )),
        ));
        self.types
            .destroy_value(&mut body, &function.parameter.ty, parameter);
        body.push(Statement::return_value(Expr::identifier(result)));
        FunctionDefinition::from_signature(
            self.owned_function_signature(function).maybe_unused(),
            body,
        )
    }
}

pub(super) fn supports_local_control_type(ty: &Type) -> bool {
    match ty {
        Type::Product(elements) | Type::Sum(elements) => {
            elements.iter().all(supports_local_control_type)
        }
        _ => true,
    }
}

pub(super) fn zero_value(emitter: &BodyEmitter<'_>, ty: &Type) -> Expr {
    Expr::compound_literal(
        emitter.types.c_type(ty),
        [Initializer::positional(Expr::number("0"))],
    )
}
