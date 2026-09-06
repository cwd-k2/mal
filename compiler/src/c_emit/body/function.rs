use crate::c_emit::syntax::{
    AggregateDefinition, AggregateField, Block as CBlock, Comment, Declaration, Expr as CExpr,
    FunctionDefinition, FunctionSignature, Initializer, Parameter, Statement, TranslationUnit,
    TypeName,
};
use crate::closure::ast as closure;

use super::{
    BodyEmitter, direct_function_name, environment_name, flattened_product_types,
    flattened_product_values, function_name, has_direct_product_entry, has_direct_tail_call,
    value_name,
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
            self.emit_top_level_globals(&mut output, &binding.pattern);
        }
        if !output.is_empty() {
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
            output.push(Declaration::function(self.function_signature(function)));
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
                let mut body = CBlock::default();
                let parameter_name = function
                    .parameter
                    .binding
                    .map_or_else(|| "mal_parameter".into(), value_name);
                let mut next_parameter = 0;
                let value =
                    self.direct_parameter_value(&function.parameter.ty, &mut next_parameter);
                body.push(Statement::variable(
                    self.types.c_type(&function.parameter.ty),
                    parameter_name.clone(),
                    Some(value),
                ));
                self.emit_function_body(&mut body, function);
                output.push(FunctionDefinition::from_signature(
                    self.direct_function_signature(function),
                    body,
                ));
                output.blank_line();

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
                continue;
            }
            let mut body = CBlock::default();
            self.emit_function_body(&mut body, function);
            output.push(FunctionDefinition::from_signature(
                self.function_signature(function),
                body,
            ));
            output.blank_line();
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
        FunctionSignature::static_function(
            result,
            function_name(function.id),
            [
                Parameter::named(TypeName::named("MalContext").pointer(), "mal_context"),
                Parameter::named(TypeName::const_named("void").pointer(), "mal_environment"),
                Parameter::named(parameter_type, parameter_name),
            ],
        )
    }

    fn direct_function_signature(&self, function: &closure::Function) -> FunctionSignature {
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
        FunctionSignature::static_function(result, direct_function_name(function.id), parameters)
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

    fn emit_function_body(&mut self, output: &mut CBlock, function: &closure::Function) {
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
        if has_direct_tail_call(&function.body, function.id) {
            let parameter_name = function
                .parameter
                .binding
                .map_or_else(|| "mal_parameter".into(), value_name);
            let mut tail = CBlock::default();
            self.emit_tail_block(&mut tail, &function.body, function.id, &parameter_name);
            output.push(Statement::label("mal_tail_entry", tail));
        } else {
            self.emit_block_bindings(output, &function.body);
            output.push(Statement::return_value(
                self.emit_atom(&function.body.result),
            ));
        }
    }
}
