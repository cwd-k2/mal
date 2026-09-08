use crate::c_emit::syntax::{
    AggregateDefinition, AggregateField, Block as CBlock, Comment, Declaration, Expr as CExpr,
    FunctionDefinition, FunctionSignature, Initializer, Parameter, Statement, TranslationUnit,
    TypeName,
};
use crate::closure::ast as closure;

use super::statement::TailParameterSlot;
use super::{
    BodyEmitter, direct_function_name, environment_destroy_name, environment_name,
    flattened_product_types, flattened_product_values, function_name, has_direct_product_entry,
    has_direct_tail_call, owned_function_name, value_name,
};

impl BodyEmitter<'_> {
    pub(super) fn emit_environments(&self) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for function in &self.program.functions {
            if function.environment.is_empty() {
                continue;
            }
            let name = environment_name(function.id);
            let fields = function
                .environment
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    AggregateField::variable(self.types.c_type(&field.ty), format!("field_{index}"))
                });
            output.push(AggregateDefinition::typedef_structure(
                Some(name.clone()),
                fields,
                name,
            ));
            output.blank_line();
        }
        output
    }

    pub(super) fn emit_globals(&self) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for binding in &self.program.bindings {
            if matches!(binding.pattern, closure::TopLevelPattern::Binding { id, .. }
                if self.closure_uses.is_direct_top_level(id))
            {
                continue;
            }
            self.emit_top_level_globals(&mut output, &binding.pattern);
        }
        if !output.is_empty() {
            output.blank_line();
        }
        output
    }

    pub(super) fn emit_environment_definitions(&self) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for function in &self.program.functions {
            if function.environment.is_empty() {
                continue;
            }
            if self.closure_uses.has_direct_creator(function.id) {
                continue;
            }
            let environment_type = environment_name(function.id);
            let mut body = CBlock::default();
            body.push(Statement::variable(
                TypeName::const_named(environment_type.clone()).pointer(),
                "mal_environment_fields",
                Some(CExpr::cast(
                    TypeName::const_named(environment_type).pointer(),
                    CExpr::identifier("mal_environment"),
                )),
            ));
            body.push(Statement::expression(CExpr::cast(
                "void",
                CExpr::identifier("mal_context"),
            )));
            body.push(Statement::expression(CExpr::cast(
                "void",
                CExpr::identifier("mal_environment_fields"),
            )));
            for (index, field) in function.environment.iter().enumerate().rev() {
                self.types.destroy_value(
                    &mut body,
                    &field.ty,
                    CExpr::identifier("mal_environment_fields")
                        .pointer_field(format!("field_{index}")),
                );
            }
            body.push(Statement::call(
                "mal_deallocate",
                [CExpr::identifier("mal_environment")],
            ));
            output.push(FunctionDefinition::from_signature(
                FunctionSignature::static_function(
                    "void",
                    environment_destroy_name(function.id),
                    [
                        Parameter::named(TypeName::named("MalContext").pointer(), "mal_context"),
                        Parameter::named(
                            TypeName::const_named("void").pointer(),
                            "mal_environment",
                        ),
                    ],
                ),
                body,
            ));
            output.blank_line();
        }
        output
    }

    pub(super) fn emit_function_declarations(&self) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for function in &self.program.functions {
            if let Some(name) = self.top_level_function_name(function.id) {
                output.push(Comment::new(format!("mal source binding: {name}")));
            }
            if has_direct_product_entry(&function.parameter.ty) {
                output.push(Declaration::function(
                    self.direct_function_signature(function),
                ));
            }
            if !has_direct_product_entry(&function.parameter.ty)
                || !self.closure_uses.has_direct_top_level_function(function.id)
            {
                output.push(Declaration::function(self.function_signature(function)));
            }
            if self.owned_calls.contains(function.id) {
                output.push(Declaration::function(
                    self.owned_function_signature(function),
                ));
            }
        }
        if !output.is_empty() {
            output.blank_line();
        }
        output
    }

    pub(super) fn emit_function_definitions(&mut self) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for function in &self.program.functions {
            if let Some(name) = self.top_level_function_name(function.id) {
                output.push(Comment::new(format!("mal source binding: {name}")));
            }
            if has_direct_product_entry(&function.parameter.ty) {
                let tail_slots = direct_tail_parameter_slots(function);
                let mut body = CBlock::default();
                let parameter_name = function
                    .parameter
                    .binding
                    .map_or_else(|| "mal_parameter".into(), value_name);
                if let Some(slots) = &tail_slots {
                    self.emit_leaf_tail_function_body(&mut body, function, slots, false);
                } else {
                    let mut next_parameter = 0;
                    let value =
                        self.direct_parameter_value(&function.parameter.ty, &mut next_parameter);
                    body.push(Statement::variable(
                        self.types.c_type(&function.parameter.ty),
                        parameter_name.clone(),
                        Some(value),
                    ));
                    self.emit_borrowed_direct_function_body(&mut body, function);
                }
                let mut signature = self.direct_function_signature(function);
                if self.closure_uses.has_direct_top_level_function(function.id)
                    && self.owned_calls.contains(function.id)
                {
                    signature = signature.maybe_unused();
                }
                output.push(FunctionDefinition::from_signature(signature, body));
                output.blank_line();

                if !self.closure_uses.has_direct_top_level_function(function.id) {
                    let arguments = flattened_product_values(
                        &function.parameter.ty,
                        CExpr::identifier(&parameter_name),
                    );
                    let mut call_arguments = vec![
                        CExpr::identifier("mal_context"),
                        CExpr::identifier("mal_environment"),
                    ];
                    call_arguments.extend(arguments);
                    let body = CBlock::new([Statement::return_value(CExpr::named_call(
                        direct_function_name(function.id),
                        call_arguments,
                    ))]);
                    output.push(FunctionDefinition::from_signature(
                        self.function_signature(function),
                        body,
                    ));
                    output.blank_line();
                }
                if self.owned_calls.contains(function.id) {
                    let mut body = CBlock::default();
                    if let Some(slots) = &tail_slots {
                        self.emit_leaf_tail_function_body(&mut body, function, slots, true);
                    } else {
                        let mut next_parameter = 0;
                        let value = self
                            .direct_parameter_value(&function.parameter.ty, &mut next_parameter);
                        body.push(Statement::variable(
                            self.types.c_type(&function.parameter.ty),
                            parameter_name,
                            Some(value),
                        ));
                        self.emit_function_body(&mut body, function, true);
                    }
                    output.push(FunctionDefinition::from_signature(
                        self.owned_function_signature(function).maybe_unused(),
                        body,
                    ));
                    output.blank_line();
                }
                continue;
            }
            let mut body = CBlock::default();
            self.emit_function_body(&mut body, function, false);
            output.push(FunctionDefinition::from_signature(
                self.function_signature(function),
                body,
            ));
            output.blank_line();
            if self.owned_calls.contains(function.id) {
                let mut body = CBlock::default();
                self.emit_function_body(&mut body, function, true);
                output.push(FunctionDefinition::from_signature(
                    self.owned_function_signature(function).maybe_unused(),
                    body,
                ));
                output.blank_line();
            }
        }
        output
    }

    fn function_signature(&self, function: &closure::Function) -> FunctionSignature {
        let result = self.types.c_type(&function.body.result.ty);
        let parameter_type = self.types.c_type(&function.parameter.ty);
        let parameter_name = function
            .parameter
            .binding
            .map_or_else(|| "mal_parameter".into(), value_name);
        let signature = FunctionSignature::static_function(
            result,
            function_name(function.id),
            [
                Parameter::named(TypeName::named("MalContext").pointer(), "mal_context"),
                Parameter::named(TypeName::const_named("void").pointer(), "mal_environment"),
                Parameter::named(parameter_type, parameter_name),
            ],
        );
        if self.closure_uses.has_direct_creator(function.id)
            || (self.closure_uses.has_direct_top_level_function(function.id)
                && self.owned_calls.contains(function.id))
        {
            signature.maybe_unused()
        } else {
            signature
        }
    }

    fn direct_function_signature(&self, function: &closure::Function) -> FunctionSignature {
        self.flattened_function_signature(function, direct_function_name(function.id), true)
    }

    fn owned_function_signature(&self, function: &closure::Function) -> FunctionSignature {
        if has_direct_product_entry(&function.parameter.ty) {
            self.flattened_function_signature(function, owned_function_name(function.id), false)
        } else {
            let parameter_name = function
                .parameter
                .binding
                .map_or_else(|| "mal_parameter".into(), value_name);
            FunctionSignature::static_function(
                self.types.c_type(&function.body.result.ty),
                owned_function_name(function.id),
                [
                    Parameter::named(TypeName::named("MalContext").pointer(), "mal_context"),
                    Parameter::named(TypeName::const_named("void").pointer(), "mal_environment"),
                    Parameter::named(self.types.c_type(&function.parameter.ty), parameter_name),
                ],
            )
        }
    }

    fn flattened_function_signature(
        &self,
        function: &closure::Function,
        name: String,
        direct: bool,
    ) -> FunctionSignature {
        let result = self.types.c_type(&function.body.result.ty);
        let crate::check::ast::Type::Product(_) = &function.parameter.ty else {
            unreachable!("only product parameters have direct entry points")
        };
        let mut parameters = vec![
            Parameter::named(TypeName::named("MalContext").pointer(), "mal_context"),
            Parameter::named(TypeName::const_named("void").pointer(), "mal_environment"),
        ];
        parameters.extend(
            flattened_product_types(&function.parameter.ty)
                .into_iter()
                .enumerate()
                .map(|(index, ty)| {
                    Parameter::named(
                        self.types.c_type(ty),
                        format!("mal_direct_parameter_{index}"),
                    )
                }),
        );
        if direct && self.closure_uses.has_direct_top_level_function(function.id) {
            FunctionSignature::static_inline(result, name, parameters)
        } else {
            FunctionSignature::static_function(result, name, parameters)
        }
    }

    fn direct_parameter_value(
        &self,
        ty: &crate::check::ast::Type,
        next_parameter: &mut usize,
    ) -> CExpr {
        if let crate::check::ast::Type::Product(elements) = ty {
            let fields = elements.iter().enumerate().map(|(index, element)| {
                Initializer::designated(
                    format!("field_{index}"),
                    self.direct_parameter_value(element, next_parameter),
                )
            });
            return CExpr::compound_literal(self.types.c_type(ty), fields);
        }
        let parameter = CExpr::identifier(format!("mal_direct_parameter_{next_parameter}"));
        *next_parameter += 1;
        parameter
    }

    fn emit_function_body(
        &mut self,
        output: &mut CBlock,
        function: &closure::Function,
        owns_parameter: bool,
    ) {
        self.emit_function_preamble(output, function);
        if has_direct_tail_call(&function.body, function.id) {
            let parameter_name = function
                .parameter
                .binding
                .map_or_else(|| "mal_parameter".into(), value_name);
            if self.types.contains_managed(&function.parameter.ty) && !owns_parameter {
                output.push(Statement::assignment(
                    CExpr::identifier(&parameter_name),
                    self.types
                        .copy_value(&function.parameter.ty, CExpr::identifier(&parameter_name)),
                ));
            }
            self.parameter_owned = true;
            let mut tail = CBlock::default();
            self.emit_tail_block(
                &mut tail,
                &function.body,
                function.id,
                &parameter_name,
                &function.parameter.ty,
            );
            output.push(Statement::label("mal_tail_entry", tail));
            self.parameter_owned = false;
        } else {
            self.parameter_owned = owns_parameter;
            self.emit_block_bindings(output, &function.body);
            let result_name = "mal_function_result";
            let mut transfers = Vec::new();
            let result = self.materialize_atom(&function.body.result, &mut transfers);
            output.push(Statement::variable(
                self.types.c_type(&function.body.result.ty),
                result_name,
                Some(result),
            ));
            self.clear_transferred_atoms(output, &transfers);
            self.emit_block_cleanup(output, &function.body);
            if owns_parameter && self.types.contains_managed(&function.parameter.ty) {
                let parameter_name = function
                    .parameter
                    .binding
                    .map_or_else(|| "mal_parameter".into(), value_name);
                self.types.destroy_value(
                    output,
                    &function.parameter.ty,
                    CExpr::identifier(parameter_name),
                );
            }
            output.push(Statement::return_value(CExpr::identifier(result_name)));
            self.parameter_owned = false;
        }
    }

    fn emit_borrowed_direct_function_body(
        &mut self,
        output: &mut CBlock,
        function: &closure::Function,
    ) {
        let Some(parameter) = function.parameter.binding else {
            self.emit_function_body(output, function, false);
            return;
        };
        self.emit_function_preamble(output, function);
        self.borrowed_bindings.insert(parameter);
        let mut borrowed = vec![parameter];
        let mut skip_bindings = 0;
        while let Some(binding) = function.body.bindings.get(skip_bindings) {
            let closure::Operation::Atom(closure::Atom {
                kind: closure::AtomKind::Reference(closure::Reference::Binding(source)),
                ..
            }) = &binding.operation
            else {
                break;
            };
            if !self.borrowed_bindings.contains(source)
                || (skip_bindings > 0
                    && !matches!(binding.pattern, closure::Pattern::Product { .. }))
            {
                break;
            }
            borrowed.extend(self.emit_borrowed_pattern_bindings(
                output,
                &binding.pattern,
                CExpr::identifier(value_name(*source)),
            ));
            skip_bindings += 1;
        }
        self.parameter_owned = false;
        self.emit_bindings(output, &function.body.bindings[skip_bindings..]);
        let mut transfers = Vec::new();
        let result = self.materialize_atom(&function.body.result, &mut transfers);
        output.push(Statement::variable(
            self.types.c_type(&function.body.result.ty),
            "mal_function_result",
            Some(result),
        ));
        self.clear_transferred_atoms(output, &transfers);
        self.emit_block_cleanup(output, &function.body);
        output.push(Statement::return_value(CExpr::identifier(
            "mal_function_result",
        )));
        self.parameter_owned = false;
        self.end_borrowed_bindings(borrowed);
    }

    fn emit_leaf_tail_function_body(
        &mut self,
        output: &mut CBlock,
        function: &closure::Function,
        slots: &[TailParameterSlot<'_>],
        owns_parameter: bool,
    ) {
        self.emit_function_preamble(output, function);
        let mut next_parameter = 0;
        for slot in slots {
            let argument = self.direct_parameter_value(slot.ty, &mut next_parameter);
            let value = if !owns_parameter && self.types.contains_managed(slot.ty) {
                self.types.copy_value(slot.ty, argument)
            } else {
                argument
            };
            let name = value_name(slot.id);
            output.push(Statement::variable(
                self.types.c_type(slot.ty),
                &name,
                Some(value),
            ));
            output.push(Statement::expression(CExpr::cast(
                "void",
                CExpr::identifier(name),
            )));
        }
        let stable_slots = slots
            .iter()
            .enumerate()
            .filter_map(|(slot_index, slot)| {
                tail_calls_carry_slot(
                    &function.body,
                    function.id,
                    slots.len(),
                    slot_index,
                    slot.id,
                )
                .then_some(slot.id)
            })
            .collect::<Vec<_>>();
        self.direct_borrow_sources
            .extend(stable_slots.iter().copied());
        let mut borrowed = Vec::new();
        let mut skip_bindings = 1;
        for binding in function.body.bindings.iter().skip(1) {
            let (
                closure::Pattern::Product { .. },
                closure::Operation::Atom(closure::Atom {
                    kind: closure::AtomKind::Reference(closure::Reference::Binding(source)),
                    ..
                }),
            ) = (&binding.pattern, &binding.operation)
            else {
                break;
            };
            if !stable_slots.contains(source) {
                break;
            }
            borrowed.extend(self.emit_borrowed_pattern_bindings(
                output,
                &binding.pattern,
                CExpr::identifier(value_name(*source)),
            ));
            skip_bindings += 1;
        }
        self.parameter_owned = true;
        let mut tail = CBlock::default();
        self.emit_leaf_tail_block(&mut tail, &function.body, function.id, slots, skip_bindings);
        output.push(Statement::label("mal_tail_entry", tail));
        self.parameter_owned = false;
        self.end_borrowed_bindings(borrowed);
        for id in stable_slots {
            self.direct_borrow_sources.remove(&id);
        }
    }

    fn emit_function_preamble(&self, output: &mut CBlock, function: &closure::Function) {
        output.push(Statement::expression(CExpr::cast(
            "void",
            CExpr::identifier("mal_context"),
        )));
        if function.environment.is_empty() {
            output.push(Statement::expression(CExpr::cast(
                "void",
                CExpr::identifier("mal_environment"),
            )));
        } else {
            let environment_type = environment_name(function.id);
            output.push(Statement::variable(
                TypeName::const_named(environment_type.clone()).pointer(),
                "mal_environment_fields",
                Some(CExpr::cast(
                    TypeName::const_named(environment_type).pointer(),
                    CExpr::identifier("mal_environment"),
                )),
            ));
        }
        if function.parameter.binding.is_none() {
            output.push(Statement::expression(CExpr::cast(
                "void",
                CExpr::identifier("mal_parameter"),
            )));
        }
    }
}

fn direct_tail_parameter_slots(function: &closure::Function) -> Option<Vec<TailParameterSlot<'_>>> {
    if !has_direct_tail_call(&function.body, function.id) {
        return None;
    }
    let parameter = function.parameter.binding?;
    let first = function.body.bindings.first()?;
    let closure::Operation::Atom(closure::Atom {
        kind: closure::AtomKind::Reference(closure::Reference::Binding(source)),
        ..
    }) = &first.operation
    else {
        return None;
    };
    if *source != parameter {
        return None;
    }
    let closure::Pattern::Product { elements, .. } = &first.pattern else {
        return None;
    };
    let slots: Option<Vec<_>> = elements
        .iter()
        .map(|pattern| match pattern {
            closure::Pattern::Binding { id, ty } => Some(TailParameterSlot { id: *id, ty }),
            _ => None,
        })
        .collect();
    let slots = slots?;
    tail_calls_have_flat_products(&function.body, function.id, slots.len()).then_some(slots)
}

fn tail_calls_have_flat_products(
    block: &closure::Block,
    function: closure::FunctionId,
    arity: usize,
) -> bool {
    let Some(tail) = block.bindings.last().filter(|binding| {
        matches!(
            (&block.result.kind, &binding.pattern),
            (
                closure::AtomKind::Reference(closure::Reference::Binding(result)),
                closure::Pattern::Binding { id, .. }
            ) if result == id
        )
    }) else {
        return true;
    };
    match &tail.operation {
        closure::Operation::Call { callee, argument }
            if matches!(
                callee.kind,
                closure::AtomKind::Reference(closure::Reference::SelfClosure(id)) if id == function
            ) =>
        {
            let closure::AtomKind::Reference(closure::Reference::Binding(argument_id)) =
                argument.kind
            else {
                return false;
            };
            let Some(product_index) = block.bindings.len().checked_sub(2) else {
                return false;
            };
            matches!(
                block.bindings.get(product_index),
                Some(closure::Binding {
                    pattern: closure::Pattern::Binding { id, .. },
                    operation: closure::Operation::Product(elements),
                    ..
                }) if *id == argument_id && elements.len() == arity
            )
        }
        closure::Operation::Case { arms, .. } => arms
            .iter()
            .all(|arm| tail_calls_have_flat_products(&arm.value, function, arity)),
        closure::Operation::PrimitiveBranch {
            otherwise, then, ..
        } => {
            tail_calls_have_flat_products(otherwise, function, arity)
                && tail_calls_have_flat_products(then, function, arity)
        }
        _ => true,
    }
}

fn tail_calls_carry_slot(
    block: &closure::Block,
    function: closure::FunctionId,
    arity: usize,
    slot_index: usize,
    slot: crate::anf::ast::ValueId,
) -> bool {
    let Some(tail) = block.bindings.last().filter(|binding| {
        matches!(
            (&block.result.kind, &binding.pattern),
            (
                closure::AtomKind::Reference(closure::Reference::Binding(result)),
                closure::Pattern::Binding { id, .. }
            ) if result == id
        )
    }) else {
        return true;
    };
    match &tail.operation {
        closure::Operation::Call { callee, argument }
            if matches!(
                callee.kind,
                closure::AtomKind::Reference(closure::Reference::SelfClosure(id)) if id == function
            ) =>
        {
            let closure::AtomKind::Reference(closure::Reference::Binding(argument_id)) =
                argument.kind
            else {
                return false;
            };
            let Some(product_index) = block.bindings.len().checked_sub(2) else {
                return false;
            };
            matches!(
                block.bindings.get(product_index),
                Some(closure::Binding {
                    pattern: closure::Pattern::Binding { id, .. },
                    operation: closure::Operation::Product(elements),
                    ..
                }) if *id == argument_id
                    && elements.len() == arity
                    && matches!(
                        elements[slot_index].kind,
                        closure::AtomKind::Reference(closure::Reference::Binding(id)) if id == slot
                    )
            )
        }
        closure::Operation::Case { arms, .. } => arms
            .iter()
            .all(|arm| tail_calls_carry_slot(&arm.value, function, arity, slot_index, slot)),
        closure::Operation::PrimitiveBranch {
            otherwise, then, ..
        } => {
            tail_calls_carry_slot(otherwise, function, arity, slot_index, slot)
                && tail_calls_carry_slot(then, function, arity, slot_index, slot)
        }
        _ => true,
    }
}
