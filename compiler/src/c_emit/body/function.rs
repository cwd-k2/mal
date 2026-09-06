use crate::c_emit::syntax::{
    AggregateDefinition, AggregateField, Block as CBlock, Comment, Declaration, Expr as CExpr,
    ForInitializer, FunctionDefinition, Initializer, Statement,
};
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
            let name = environment_name(function.id);
            let fields = function
                .environment
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    AggregateField::declaration(format!(
                        "{} field_{index}",
                        self.types.c_type(&field.ty)
                    ))
                });
            output.push_str(
                &AggregateDefinition::new(format!("typedef struct {name}"), fields, Some(name))
                    .render(),
            );
            output.push('\n');
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
                output.push_str(&Comment::new(format!("mal source binding: {name}")).render());
            }
            if has_direct_product_entry(&function.parameter.ty) {
                output
                    .push_str(&Declaration::new(self.direct_function_signature(function)).render());
            }
            output.push_str(&Declaration::new(self.function_signature(function)).render());
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
                output.push_str(&Comment::new(format!("mal source binding: {name}")).render());
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
                body.push(Statement::declaration(
                    format!(
                        "{} {parameter_name}",
                        self.types.c_type(&function.parameter.ty)
                    ),
                    Some(value),
                ));
                self.emit_function_body(&mut body, function);
                output.push_str(
                    &FunctionDefinition::new(self.direct_function_signature(function), body)
                        .render(),
                );
                output.push('\n');

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
                output.push_str(
                    &FunctionDefinition::new(self.function_signature(function), body).render(),
                );
                output.push('\n');
                continue;
            }
            let mut body = CBlock::default();
            self.emit_function_body(&mut body, function);
            output.push_str(
                &FunctionDefinition::new(self.function_signature(function), body).render(),
            );
            output.push('\n');
        }
        output
    }

    pub(super) fn emit_initializer(&mut self) -> String {
        let mut body = CBlock::default();
        body.push(Statement::expression(CExpr::cast(
            "void",
            CExpr::identifier("mal_context"),
        )));
        for binding in &self.program.bindings {
            self.emit_block_bindings(&mut body, &binding.value);
            match &binding.pattern {
                TopLevelPattern::Binding { id, .. } => {
                    body.push(Statement::assignment(
                        CExpr::identifier(value_name(*id)),
                        self.emit_atom(&binding.value.result),
                    ));
                    body.push(Statement::expression(CExpr::cast(
                        "void",
                        CExpr::identifier(value_name(*id)),
                    )));
                }
                TopLevelPattern::Wildcard { .. } => {
                    body.push(Statement::expression(CExpr::cast(
                        "void",
                        self.emit_atom(&binding.value.result),
                    )));
                }
                TopLevelPattern::Product { .. } => self.emit_top_level_pattern(
                    &mut body,
                    &binding.pattern,
                    self.emit_atom(&binding.value.result),
                ),
            }
        }
        let mut output = FunctionDefinition::new(
            "static void mal_program_initialize(MalContext *mal_context)",
            body,
        )
        .render();
        output.push('\n');
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
            return FunctionDefinition::new(
                "int main(void)",
                CBlock::new([
                    Statement::declaration(
                        "MalContext mal_context",
                        Some(CExpr::initializer_list([Initializer::positional(
                            CExpr::identifier("NULL"),
                        )])),
                    ),
                    Statement::declaration(
                        "MalType_Unit mal_unit",
                        Some(CExpr::initializer_list([Initializer::positional(
                            CExpr::named_call("UINT8_C", [CExpr::literal("0")]),
                        )])),
                    ),
                    Statement::expression(CExpr::named_call(
                        "mal_program_initialize",
                        [CExpr::unary("&", CExpr::identifier("mal_context"))],
                    )),
                    Statement::declaration(
                        "int32_t mal_result",
                        Some(CExpr::call(
                            CExpr::identifier(name.clone()).field("call"),
                            [
                                CExpr::unary("&", CExpr::identifier("mal_context")),
                                CExpr::identifier(name.clone()).field("environment"),
                                CExpr::identifier("mal_unit"),
                            ],
                        )),
                    ),
                    Statement::expression(CExpr::named_call(
                        "mal_context_destroy",
                        [CExpr::unary("&", CExpr::identifier("mal_context"))],
                    )),
                    Statement::return_value(CExpr::cast("int", CExpr::identifier("mal_result"))),
                ]),
            )
            .render();
        }

        let parameter_type = self.types.c_type(parameter);
        let argv = || {
            CExpr::identifier("mal_argv").index(CExpr::binary(
                "+",
                CExpr::identifier("mal_index"),
                CExpr::literal("1"),
            ))
        };
        let trap = |message: &'static str| {
            Statement::expression(CExpr::named_call(
                "mal_trap",
                [
                    CExpr::unary("&", CExpr::identifier("mal_context")),
                    CExpr::literal(format!("\"{message}\"")),
                ],
            ))
        };
        FunctionDefinition::new(
            "int main(int mal_argc, char **mal_argv)",
            CBlock::new([
                Statement::declaration(
                    "MalContext mal_context",
                    Some(CExpr::initializer_list([Initializer::positional(
                        CExpr::identifier("NULL"),
                    )])),
                ),
                Statement::expression(CExpr::named_call(
                    "mal_program_initialize",
                    [CExpr::unary("&", CExpr::identifier("mal_context"))],
                )),
                Statement::declaration(
                    "size_t mal_argument_count",
                    Some(CExpr::conditional(
                        CExpr::binary(">", CExpr::identifier("mal_argc"), CExpr::literal("1")),
                        CExpr::cast(
                            "size_t",
                            CExpr::binary("-", CExpr::identifier("mal_argc"), CExpr::literal("1")),
                        ),
                        CExpr::literal("0"),
                    )),
                ),
                Statement::declaration(
                    "const size_t mal_argument_stride",
                    Some(CExpr::binary(
                        "+",
                        CExpr::sizeof_type("MalType_Ptr"),
                        CExpr::sizeof_type("uint64_t"),
                    )),
                ),
                Statement::if_then(
                    CExpr::binary(
                        ">",
                        CExpr::identifier("mal_argument_count"),
                        CExpr::binary(
                            "/",
                            CExpr::identifier("SIZE_MAX"),
                            CExpr::identifier("mal_argument_stride"),
                        ),
                    ),
                    CBlock::new([trap("argument descriptor size overflow")]),
                ),
                Statement::declaration(
                    "uint8_t *mal_argument_storage",
                    Some(CExpr::cast(
                        "uint8_t *",
                        CExpr::named_call(
                            "mal_allocate",
                            [
                                CExpr::unary("&", CExpr::identifier("mal_context")),
                                CExpr::binary(
                                    "*",
                                    CExpr::identifier("mal_argument_count"),
                                    CExpr::identifier("mal_argument_stride"),
                                ),
                            ],
                        ),
                    )),
                ),
                Statement::for_loop(
                    ForInitializer::declaration("size_t mal_index", CExpr::literal("0")),
                    CExpr::binary(
                        "<",
                        CExpr::identifier("mal_index"),
                        CExpr::identifier("mal_argument_count"),
                    ),
                    CExpr::unary("++", CExpr::identifier("mal_index")),
                    CBlock::new([
                        Statement::declaration(
                            "size_t mal_length",
                            Some(CExpr::named_call("strlen", [argv()])),
                        ),
                        Statement::if_then(
                            CExpr::binary(
                                "!=",
                                CExpr::cast("uint64_t", CExpr::identifier("mal_length")),
                                CExpr::identifier("mal_length"),
                            ),
                            CBlock::new([trap("argument length overflow")]),
                        ),
                        Statement::declaration(
                            "MalType_Ptr mal_data",
                            Some(CExpr::named_call(
                                "mal_Ptr_from_address",
                                [CExpr::cast("uint8_t *", argv())],
                            )),
                        ),
                        Statement::declaration(
                            "uint64_t mal_length_u64",
                            Some(CExpr::cast("uint64_t", CExpr::identifier("mal_length"))),
                        ),
                        Statement::declaration(
                            "uint8_t *mal_slot",
                            Some(CExpr::binary(
                                "+",
                                CExpr::identifier("mal_argument_storage"),
                                CExpr::binary(
                                    "*",
                                    CExpr::identifier("mal_index"),
                                    CExpr::identifier("mal_argument_stride"),
                                ),
                            )),
                        ),
                        Statement::expression(CExpr::named_call(
                            "memcpy",
                            [
                                CExpr::identifier("mal_slot"),
                                CExpr::unary("&", CExpr::identifier("mal_data")),
                                CExpr::sizeof_type("mal_data"),
                            ],
                        )),
                        Statement::expression(CExpr::named_call(
                            "memcpy",
                            [
                                CExpr::binary(
                                    "+",
                                    CExpr::identifier("mal_slot"),
                                    CExpr::sizeof_type("mal_data"),
                                ),
                                CExpr::unary("&", CExpr::identifier("mal_length_u64")),
                                CExpr::sizeof_type("mal_length_u64"),
                            ],
                        )),
                    ]),
                ),
                Statement::declaration(
                    format!("{parameter_type} mal_arguments"),
                    Some(CExpr::compound_literal(
                        parameter_type,
                        [
                            Initializer::designated(
                                "field_0",
                                CExpr::cast("uint64_t", CExpr::identifier("mal_argument_count")),
                            ),
                            Initializer::designated(
                                "field_1",
                                CExpr::named_call(
                                    "mal_Ptr_from_address",
                                    [CExpr::identifier("mal_argument_storage")],
                                ),
                            ),
                        ],
                    )),
                ),
                Statement::declaration(
                    "int32_t mal_result",
                    Some(CExpr::call(
                        CExpr::identifier(name.clone()).field("call"),
                        [
                            CExpr::unary("&", CExpr::identifier("mal_context")),
                            CExpr::identifier(name).field("environment"),
                            CExpr::identifier("mal_arguments"),
                        ],
                    )),
                ),
                Statement::expression(CExpr::named_call(
                    "mal_context_destroy",
                    [CExpr::unary("&", CExpr::identifier("mal_context"))],
                )),
                Statement::return_value(CExpr::cast("int", CExpr::identifier("mal_result"))),
            ]),
        )
        .render()
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
            output.push(Statement::declaration(
                format!("const {environment_type} *mal_environment_fields"),
                Some(CExpr::cast(
                    format!("const {environment_type} *"),
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
