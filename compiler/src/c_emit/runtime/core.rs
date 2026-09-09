use crate::c_emit::syntax::{
    AggregateDefinition, AggregateField, Block, Declaration, Directive, Expr, FunctionDefinition,
    FunctionSignature, Initializer, Parameter, PreprocessorExpr, Statement, TranslationUnit,
    TypeName, VariableDeclaration,
};

pub(super) fn emit(needs_symbol_copy: bool, control_arenas: usize) -> TranslationUnit {
    let mut output = TranslationUnit::default();
    output.push(AggregateDefinition::typedef_structure(
        None,
        [
            AggregateField::variable("uint64_t", "references"),
            AggregateField::variable("size_t", "capacity"),
        ],
        "MalAllocation",
    ));
    output.blank_line();
    output.push(AggregateDefinition::typedef_structure(
        Some("MalSymbolRope".into()),
        [
            AggregateField::variable("MalType_Symbol", "left"),
            AggregateField::variable("MalType_Symbol", "right"),
            AggregateField::variable(TypeName::named("uint8_t").pointer(), "flattened"),
            AggregateField::variable("uint8_t", "height"),
        ],
        "MalSymbolRope",
    ));
    output.blank_line();
    if control_arenas != 0 {
        output.push(AggregateDefinition::typedef_structure(
            None,
            [
                AggregateField::variable(TypeName::named("uint8_t").pointer(), "storage"),
                AggregateField::variable("size_t", "capacity"),
            ],
            "MalControlArena",
        ));
        output.blank_line();
        output.push(AggregateDefinition::typedef_structure(
            None,
            [
                AggregateField::variable(TypeName::named("uint8_t").pointer(), "storage"),
                AggregateField::variable("size_t", "capacity"),
                AggregateField::variable("size_t", "top"),
                AggregateField::variable("size_t", "frame"),
            ],
            "MalControlStack",
        ));
        output.blank_line();
    }
    let mut context_fields = vec![AggregateField::variable("uint8_t", "unused")];
    for arena in 0..control_arenas {
        context_fields.push(AggregateField::variable(
            "MalControlArena",
            control_arena_name(arena),
        ));
    }
    output.push(AggregateDefinition::structure("MalContext", context_fields));
    output.push(Directive::If(live_allocation_tracking()));
    output.push(Declaration::variable(VariableDeclaration::static_variable(
        "size_t",
        "mal_live_allocations",
    )));
    output.push(Directive::Endif);
    output.push(Directive::If(PreprocessorExpr::defined(
        "MAL_TEST_TOTAL_ALLOCATION_LIMIT",
    )));
    output.push(Declaration::variable(VariableDeclaration::static_variable(
        "size_t",
        "mal_total_allocations",
    )));
    output.push(Directive::Endif);
    for (counter, limit) in [
        ("mal_test_retain_count", "MAL_TEST_RETAIN_LIMIT"),
        ("mal_test_release_count", "MAL_TEST_RELEASE_LIMIT"),
        (
            "mal_test_materialization_count",
            "MAL_TEST_MATERIALIZATION_LIMIT",
        ),
    ] {
        output.push(Directive::If(PreprocessorExpr::defined(limit)));
        output.push(Declaration::variable(VariableDeclaration::static_variable(
            "size_t", counter,
        )));
        output.push(Directive::Endif);
    }
    output.blank_line();

    append_trap(&mut output);
    append_resource_failure(&mut output);
    if control_arenas != 0 {
        append_control_stack(&mut output);
    }
    let mut context_destroy = Block::new([
        Statement::expression(Expr::cast("void", Expr::identifier("context"))),
        Statement::directive(Directive::If(PreprocessorExpr::defined(
            "MAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS",
        ))),
        Statement::if_then(
            Expr::not_equal(Expr::identifier("mal_live_allocations"), Expr::number("0")),
            trap("live allocations at context destruction"),
        ),
        Statement::directive(Directive::Endif),
        Statement::directive(Directive::If(PreprocessorExpr::defined(
            "MAL_TEST_TOTAL_ALLOCATION_LIMIT",
        ))),
        Statement::if_then(
            Expr::greater(
                Expr::identifier("mal_total_allocations"),
                Expr::cast(
                    "size_t",
                    Expr::identifier("MAL_TEST_TOTAL_ALLOCATION_LIMIT"),
                ),
            ),
            trap("total allocation limit exceeded"),
        ),
        Statement::directive(Directive::Endif),
    ]);
    push_test_counter_limit_check(
        &mut context_destroy,
        "mal_test_retain_count",
        "MAL_TEST_RETAIN_LIMIT",
        "retain limit exceeded",
    );
    for arena in 0..control_arenas {
        context_destroy.push(Statement::call(
            "free",
            [Expr::identifier("context")
                .pointer_field(control_arena_name(arena))
                .field("storage")],
        ));
    }
    push_test_counter_limit_check(
        &mut context_destroy,
        "mal_test_release_count",
        "MAL_TEST_RELEASE_LIMIT",
        "release limit exceeded",
    );
    push_test_counter_limit_check(
        &mut context_destroy,
        "mal_test_materialization_count",
        "MAL_TEST_MATERIALIZATION_LIMIT",
        "materialization limit exceeded",
    );
    append_function(
        &mut output,
        FunctionSignature::static_function("void", "mal_context_destroy", [context_parameter()]),
        context_destroy,
    );
    append_allocation(&mut output);
    append_reference_counting(&mut output);
    append_symbol_lifetime(&mut output);
    append_host_symbol_lifetime(&mut output);
    append_symbol_admission(&mut output);
    if needs_symbol_copy {
        append_symbol_copy(&mut output);
    }
    append_symbol_materialization(&mut output);
    output
}

fn append_control_stack(output: &mut TranslationUnit) {
    output.push(AggregateDefinition::typedef_structure(
        None,
        [
            AggregateField::variable("size_t", "previous_frame"),
            AggregateField::variable("uint32_t", "resume"),
        ],
        "MalControlFrameHeader",
    ));
    output.blank_line();
    let alignment = Expr::sizeof_type("max_align_t");
    let padded_input = Expr::add(
        Expr::identifier("size"),
        Expr::subtract(alignment.clone(), Expr::number("1")),
    );
    let padded = Expr::multiply(
        Expr::divide(padded_input, alignment.clone()),
        alignment.clone(),
    );
    let mut body = Block::new([
        Statement::if_then(
            Expr::greater(
                Expr::identifier("size"),
                Expr::subtract(
                    Expr::identifier("SIZE_MAX"),
                    Expr::subtract(alignment, Expr::number("1")),
                ),
            ),
            control_failure("control frame size overflow"),
        ),
        Statement::variable("size_t", "padded", Some(padded)),
        Statement::variable("size_t", "required", None),
    ]);
    let mut grow = Block::new([
        Statement::variable(
            "size_t",
            "capacity",
            Some(Expr::identifier("control").pointer_field("capacity")),
        ),
        Statement::if_then(
            Expr::equal(Expr::identifier("capacity"), Expr::number("0")),
            Block::new([Statement::assignment(
                Expr::identifier("capacity"),
                Expr::number("256"),
            )]),
        ),
    ]);
    grow.push(Statement::if_then(
        Expr::less(Expr::identifier("capacity"), Expr::identifier("required")),
        Block::new([Statement::if_else(
            Expr::logical_or(
                Expr::greater(
                    Expr::identifier("capacity"),
                    Expr::divide(Expr::identifier("SIZE_MAX"), Expr::number("2")),
                ),
                Expr::less(
                    Expr::multiply(Expr::identifier("capacity"), Expr::number("2")),
                    Expr::identifier("required"),
                ),
            ),
            Block::new([Statement::assignment(
                Expr::identifier("capacity"),
                Expr::identifier("required"),
            )]),
            Block::new([Statement::assignment(
                Expr::identifier("capacity"),
                Expr::multiply(Expr::identifier("capacity"), Expr::number("2")),
            )]),
        )]),
    ));
    grow.push(Statement::directive(Directive::If(
        PreprocessorExpr::defined("MAL_TEST_FORCE_CONTROL_ALLOCATION_FAILURE"),
    )));
    grow.push(Statement::variable(
        TypeName::named("void").pointer(),
        "storage",
        Some(Expr::identifier("NULL")),
    ));
    grow.push(Statement::directive(Directive::Else));
    grow.push(Statement::variable(
        TypeName::named("void").pointer(),
        "storage",
        Some(Expr::named_call(
            "realloc",
            [
                Expr::identifier("control").pointer_field("storage"),
                Expr::identifier("capacity"),
            ],
        )),
    ));
    grow.push(Statement::directive(Directive::Endif));
    grow.push(Statement::if_then(
        Expr::equal(Expr::identifier("storage"), Expr::identifier("NULL")),
        control_failure("control stack allocation failed"),
    ));
    grow.push(Statement::assignment(
        Expr::identifier("control").pointer_field("storage"),
        Expr::cast(
            TypeName::named("uint8_t").pointer(),
            Expr::identifier("storage"),
        ),
    ));
    grow.push(Statement::assignment(
        Expr::identifier("control").pointer_field("capacity"),
        Expr::identifier("capacity"),
    ));
    let mut slow = Block::new([
        Statement::if_then(
            Expr::greater(
                Expr::identifier("control").pointer_field("top"),
                Expr::subtract(Expr::identifier("SIZE_MAX"), Expr::identifier("padded")),
            ),
            control_failure("control stack size overflow"),
        ),
        Statement::assignment(
            Expr::identifier("required"),
            Expr::add(
                Expr::identifier("control").pointer_field("top"),
                Expr::identifier("padded"),
            ),
        ),
    ]);
    slow.append(grow);
    body.push(Statement::if_else(
        Expr::greater(
            Expr::identifier("padded"),
            Expr::subtract(
                Expr::identifier("control").pointer_field("capacity"),
                Expr::identifier("control").pointer_field("top"),
            ),
        ),
        slow,
        Block::new([Statement::assignment(
            Expr::identifier("required"),
            Expr::add(
                Expr::identifier("control").pointer_field("top"),
                Expr::identifier("padded"),
            ),
        )]),
    ));
    body.push(Statement::variable(
        "size_t",
        "start",
        Some(Expr::identifier("control").pointer_field("top")),
    ));
    body.push(Statement::assignment(
        Expr::identifier("control").pointer_field("top"),
        Expr::identifier("required"),
    ));
    body.push(Statement::assignment(
        Expr::identifier("control").pointer_field("frame"),
        Expr::identifier("start"),
    ));
    body.push(Statement::return_value(Expr::add(
        Expr::identifier("control").pointer_field("storage"),
        Expr::identifier("start"),
    )));
    append_function(
        output,
        FunctionSignature::static_function(
            TypeName::named("void").pointer(),
            "mal_control_push",
            [
                Parameter::named(TypeName::named("MalControlStack").pointer(), "control"),
                Parameter::named("size_t", "size"),
            ],
        )
        .maybe_unused(),
        body,
    );
}

fn control_arena_name(arena: usize) -> String {
    format!("control_arena_{arena}")
}

fn control_failure(message: &str) -> Block {
    resource_failure(message)
}

fn append_trap(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::no_return(
            "void",
            "mal_trap",
            [
                context_parameter(),
                Parameter::named(TypeName::const_named("char").pointer(), "message"),
            ],
        ),
        Block::new([
            Statement::expression(Expr::cast("void", Expr::identifier("context"))),
            Statement::call(
                "fputs",
                [Expr::string("mal trap: "), Expr::identifier("stderr")],
            ),
            Statement::call(
                "fputs",
                [Expr::identifier("message"), Expr::identifier("stderr")],
            ),
            Statement::call("fputc", [Expr::character('\n'), Expr::identifier("stderr")]),
            Statement::call("abort", []),
        ]),
    );
}

fn append_resource_failure(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::no_return(
            "void",
            "mal_resource_failure",
            [Parameter::named(
                TypeName::const_named("char").pointer(),
                "message",
            )],
        )
        .maybe_unused(),
        Block::new([
            Statement::call(
                "fputs",
                [
                    Expr::string("mal implementation resource failure: "),
                    Expr::identifier("stderr"),
                ],
            ),
            Statement::call(
                "fputs",
                [Expr::identifier("message"), Expr::identifier("stderr")],
            ),
            Statement::call("fputc", [Expr::character('\n'), Expr::identifier("stderr")]),
            Statement::call("abort", []),
        ]),
    );
}

fn append_allocation(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::static_function(
            TypeName::named("void").pointer(),
            "mal_allocate",
            [context_parameter(), Parameter::named("size_t", "size")],
        ),
        Block::new([
            Statement::expression(Expr::cast("void", Expr::identifier("context"))),
            Statement::if_then(
                Expr::greater(
                    Expr::identifier("size"),
                    Expr::subtract(
                        Expr::identifier("SIZE_MAX"),
                        Expr::sizeof_type("MalAllocation"),
                    ),
                ),
                trap("allocation size overflow"),
            ),
            Statement::directive(Directive::If(PreprocessorExpr::defined(
                "MAL_TEST_LIVE_ALLOCATION_LIMIT",
            ))),
            Statement::if_then(
                Expr::greater_equal(
                    Expr::identifier("mal_live_allocations"),
                    Expr::cast("size_t", Expr::identifier("MAL_TEST_LIVE_ALLOCATION_LIMIT")),
                ),
                trap("allocation failed"),
            ),
            Statement::directive(Directive::Endif),
            Statement::directive(Directive::If(PreprocessorExpr::defined(
                "MAL_TEST_FORCE_ALLOCATION_FAILURE",
            ))),
            Statement::variable(
                TypeName::named("MalAllocation").pointer(),
                "allocation",
                Some(Expr::identifier("NULL")),
            ),
            Statement::directive(Directive::Else),
            Statement::variable(
                TypeName::named("MalAllocation").pointer(),
                "allocation",
                Some(Expr::named_call(
                    "malloc",
                    [Expr::add(
                        Expr::sizeof_type("MalAllocation"),
                        Expr::identifier("size"),
                    )],
                )),
            ),
            Statement::directive(Directive::Endif),
            Statement::if_then(
                Expr::equal(Expr::identifier("allocation"), Expr::identifier("NULL")),
                trap("allocation failed"),
            ),
            Statement::assignment(
                Expr::identifier("allocation").pointer_field("references"),
                uint64(1),
            ),
            Statement::assignment(
                Expr::identifier("allocation").pointer_field("capacity"),
                Expr::identifier("size"),
            ),
            Statement::directive(Directive::If(live_allocation_tracking())),
            Statement::expression(Expr::pre_increment(Expr::identifier(
                "mal_live_allocations",
            ))),
            Statement::directive(Directive::Endif),
            Statement::directive(Directive::If(PreprocessorExpr::defined(
                "MAL_TEST_TOTAL_ALLOCATION_LIMIT",
            ))),
            Statement::expression(Expr::pre_increment(Expr::identifier(
                "mal_total_allocations",
            ))),
            Statement::directive(Directive::Endif),
            Statement::return_value(Expr::add(Expr::identifier("allocation"), Expr::number("1"))),
        ]),
    );
}

fn append_reference_counting(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::static_function(
            TypeName::const_named("void").pointer(),
            "mal_retain",
            [
                context_parameter(),
                Parameter::named(TypeName::const_named("void").pointer(), "value"),
            ],
        ),
        Block::new([
            Statement::expression(Expr::cast("void", Expr::identifier("context"))),
            Statement::directive(Directive::If(PreprocessorExpr::defined(
                "MAL_TEST_RETAIN_LIMIT",
            ))),
            Statement::expression(Expr::pre_increment(Expr::identifier(
                "mal_test_retain_count",
            ))),
            Statement::directive(Directive::Endif),
            Statement::if_then(
                Expr::equal(Expr::identifier("value"), Expr::identifier("NULL")),
                Block::new([Statement::return_value(Expr::identifier("NULL"))]),
            ),
            Statement::variable(
                TypeName::named("MalAllocation").pointer(),
                "allocation",
                Some(allocation_for(Expr::identifier("value"))),
            ),
            Statement::if_then(
                Expr::equal(
                    Expr::identifier("allocation").pointer_field("references"),
                    Expr::identifier("UINT64_MAX"),
                ),
                resource_failure("reference count overflow"),
            ),
            Statement::expression(Expr::pre_increment(
                Expr::identifier("allocation").pointer_field("references"),
            )),
            Statement::return_value(Expr::identifier("value")),
        ]),
    );
    append_function(
        output,
        FunctionSignature::static_function(
            "uint8_t",
            "mal_release",
            [Parameter::named(
                TypeName::const_named("void").pointer(),
                "value",
            )],
        ),
        Block::new([
            Statement::directive(Directive::If(PreprocessorExpr::defined(
                "MAL_TEST_RELEASE_LIMIT",
            ))),
            Statement::expression(Expr::pre_increment(Expr::identifier(
                "mal_test_release_count",
            ))),
            Statement::directive(Directive::Endif),
            Statement::if_then(
                Expr::equal(Expr::identifier("value"), Expr::identifier("NULL")),
                Block::new([Statement::return_value(uint8(0))]),
            ),
            Statement::variable(
                TypeName::named("MalAllocation").pointer(),
                "allocation",
                Some(allocation_for(Expr::identifier("value"))),
            ),
            Statement::assignment(
                Expr::identifier("allocation").pointer_field("references"),
                Expr::subtract(
                    Expr::identifier("allocation").pointer_field("references"),
                    uint64(1),
                ),
            ),
            Statement::return_value(Expr::cast(
                "uint8_t",
                Expr::equal(
                    Expr::identifier("allocation").pointer_field("references"),
                    uint64(0),
                ),
            )),
        ]),
    );
    append_function(
        output,
        FunctionSignature::static_function(
            "void",
            "mal_deallocate",
            [Parameter::named(
                TypeName::const_named("void").pointer(),
                "value",
            )],
        ),
        Block::new([
            Statement::directive(Directive::If(live_allocation_tracking())),
            Statement::assignment(
                Expr::identifier("mal_live_allocations"),
                Expr::subtract(Expr::identifier("mal_live_allocations"), Expr::number("1")),
            ),
            Statement::directive(Directive::Endif),
            Statement::call("free", [allocation_for(Expr::identifier("value"))]),
        ]),
    );
}

fn append_symbol_lifetime(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_symbol_retain",
            [
                context_parameter(),
                Parameter::named("MalType_Symbol", "value"),
            ],
        ),
        Block::new([
            Statement::expression(Expr::named_call(
                "mal_retain",
                [
                    Expr::identifier("context"),
                    Expr::identifier("value").field("ownership"),
                ],
            )),
            Statement::return_value(Expr::identifier("value")),
        ]),
    );
    append_function(
        output,
        FunctionSignature::static_inline(
            "void",
            "mal_symbol_release",
            [Parameter::named("MalType_Symbol", "value")],
        ),
        Block::new([Statement::if_then(
            Expr::not_equal(
                Expr::identifier("value").field("ownership"),
                Expr::identifier("NULL"),
            ),
            Block::new([Statement::if_then(
                Expr::not_equal(
                    Expr::named_call(
                        "mal_release",
                        [Expr::identifier("value").field("ownership")],
                    ),
                    uint8(0),
                ),
                Block::new([
                    Statement::if_then(
                        Expr::equal(
                            allocation_for(Expr::identifier("value").field("ownership"))
                                .pointer_field("capacity"),
                            Expr::identifier("SIZE_MAX"),
                        ),
                        Block::new([
                            Statement::variable(
                                TypeName::named("MalSymbolRope").pointer(),
                                "rope",
                                Some(Expr::cast(
                                    TypeName::named("MalSymbolRope").pointer(),
                                    Expr::identifier("value").field("ownership"),
                                )),
                            ),
                            Statement::call(
                                "mal_symbol_release",
                                [Expr::identifier("rope").pointer_field("right")],
                            ),
                            Statement::call(
                                "mal_symbol_release",
                                [Expr::identifier("rope").pointer_field("left")],
                            ),
                            Statement::if_then(
                                Expr::not_equal(
                                    Expr::identifier("rope").pointer_field("flattened"),
                                    Expr::identifier("NULL"),
                                ),
                                Block::new([Statement::call(
                                    "mal_deallocate",
                                    [Expr::identifier("rope").pointer_field("flattened")],
                                )]),
                            ),
                        ]),
                    ),
                    Statement::call(
                        "mal_deallocate",
                        [Expr::identifier("value").field("ownership")],
                    ),
                ]),
            )]),
        )]),
    );
}

fn append_host_symbol_lifetime(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_Symbol_clone",
            [
                context_parameter(),
                Parameter::named("MalType_Symbol", "value"),
            ],
        ),
        Block::new([Statement::return_value(Expr::named_call(
            "mal_symbol_retain",
            [Expr::identifier("context"), Expr::identifier("value")],
        ))]),
    );
    append_function(
        output,
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_Symbol_take",
            [Parameter::named(
                TypeName::named("MalType_Symbol").pointer(),
                "value",
            )],
        ),
        Block::new([
            Statement::variable(
                "MalType_Symbol",
                "result",
                Some(Expr::dereference(Expr::identifier("value"))),
            ),
            Statement::assignment(
                Expr::dereference(Expr::identifier("value")),
                symbol([
                    Expr::identifier("NULL"),
                    uint64(0),
                    Expr::identifier("NULL"),
                ]),
            ),
            Statement::return_value(Expr::identifier("result")),
        ]),
    );
    append_function(
        output,
        FunctionSignature::new(
            "void",
            "mal_Symbol_drop",
            [
                context_parameter(),
                Parameter::named(TypeName::named("MalType_Symbol").pointer(), "value"),
            ],
        ),
        Block::new([
            Statement::expression(Expr::cast("void", Expr::identifier("context"))),
            Statement::call(
                "mal_symbol_release",
                [Expr::dereference(Expr::identifier("value"))],
            ),
            Statement::assignment(
                Expr::dereference(Expr::identifier("value")),
                symbol([
                    Expr::identifier("NULL"),
                    uint64(0),
                    Expr::identifier("NULL"),
                ]),
            ),
        ]),
    );
}

fn append_symbol_admission(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::new(
            "MalSymbolAdmission",
            "mal_SymbolAdmission_begin",
            [
                context_parameter(),
                Parameter::named("uint64_t", "minimum_capacity"),
            ],
        ),
        Block::new([
            Statement::variable(
                TypeName::named("void").pointer(),
                "state",
                Some(Expr::identifier("NULL")),
            ),
            Statement::if_then(
                Expr::not_equal(Expr::identifier("minimum_capacity"), uint64(0)),
                Block::new([
                    Statement::variable(
                        "size_t",
                        "size",
                        Some(Expr::cast("size_t", Expr::identifier("minimum_capacity"))),
                    ),
                    Statement::if_then(
                        Expr::not_equal(
                            Expr::cast("uint64_t", Expr::identifier("size")),
                            Expr::identifier("minimum_capacity"),
                        ),
                        trap("allocation size overflow"),
                    ),
                    Statement::assignment(
                        Expr::identifier("state"),
                        Expr::named_call(
                            "mal_allocate",
                            [Expr::identifier("context"), Expr::identifier("size")],
                        ),
                    ),
                ]),
            ),
            Statement::return_value(Expr::compound_literal(
                "MalSymbolAdmission",
                [Initializer::designated("state", Expr::identifier("state"))],
            )),
        ]),
    );
    append_function(
        output,
        FunctionSignature::new(
            "uint64_t",
            "mal_SymbolAdmission_capacity",
            [Parameter::named(
                TypeName::const_named("MalSymbolAdmission").pointer(),
                "admission",
            )],
        ),
        Block::new([
            Statement::if_then(
                Expr::equal(
                    Expr::identifier("admission").pointer_field("state"),
                    Expr::identifier("NULL"),
                ),
                Block::new([Statement::return_value(uint64(0))]),
            ),
            Statement::return_value(Expr::cast(
                "uint64_t",
                allocation_for(Expr::identifier("admission").pointer_field("state"))
                    .pointer_field("capacity"),
            )),
        ]),
    );
    append_function(
        output,
        FunctionSignature::new(
            TypeName::named("uint8_t").pointer(),
            "mal_SymbolAdmission_data",
            [Parameter::named(
                TypeName::named("MalSymbolAdmission").pointer(),
                "admission",
            )],
        ),
        Block::new([Statement::return_value(Expr::cast(
            TypeName::named("uint8_t").pointer(),
            Expr::identifier("admission").pointer_field("state"),
        ))]),
    );

    let resize = Block::new([
        Statement::variable(
            TypeName::named("MalAllocation").pointer(),
            "allocation",
            Some(allocation_for(
                Expr::identifier("admission").pointer_field("state"),
            )),
        ),
        Statement::variable(
            "size_t",
            "capacity",
            Some(Expr::identifier("allocation").pointer_field("capacity")),
        ),
        Statement::if_then(
            Expr::greater_equal(Expr::identifier("capacity"), Expr::identifier("size")),
            Block::new([Statement::goto("mal_admission_reserve_done")]),
        ),
        Statement::variable(
            "size_t",
            "maximum_capacity",
            Some(Expr::subtract(
                Expr::identifier("SIZE_MAX"),
                Expr::sizeof_type("MalAllocation"),
            )),
        ),
        Statement::if_else(
            Expr::greater(
                Expr::identifier("capacity"),
                Expr::subtract(
                    Expr::identifier("maximum_capacity"),
                    Expr::identifier("capacity"),
                ),
            ),
            Block::new([Statement::assignment(
                Expr::identifier("capacity"),
                Expr::identifier("maximum_capacity"),
            )]),
            Block::new([Statement::assignment(
                Expr::identifier("capacity"),
                Expr::add(Expr::identifier("capacity"), Expr::identifier("capacity")),
            )]),
        ),
        Statement::if_then(
            Expr::less(Expr::identifier("capacity"), Expr::identifier("size")),
            Block::new([Statement::assignment(
                Expr::identifier("capacity"),
                Expr::identifier("size"),
            )]),
        ),
        Statement::directive(Directive::If(PreprocessorExpr::defined(
            "MAL_TEST_TOTAL_ALLOCATION_LIMIT",
        ))),
        Statement::expression(Expr::pre_increment(Expr::identifier(
            "mal_total_allocations",
        ))),
        Statement::directive(Directive::Endif),
        Statement::directive(Directive::If(PreprocessorExpr::defined(
            "MAL_TEST_FORCE_REALLOCATION_FAILURE",
        ))),
        Statement::variable(
            TypeName::named("MalAllocation").pointer(),
            "resized",
            Some(Expr::identifier("NULL")),
        ),
        Statement::directive(Directive::Else),
        Statement::variable(
            TypeName::named("MalAllocation").pointer(),
            "resized",
            Some(Expr::named_call(
                "realloc",
                [
                    Expr::identifier("allocation"),
                    Expr::add(
                        Expr::sizeof_type("MalAllocation"),
                        Expr::identifier("capacity"),
                    ),
                ],
            )),
        ),
        Statement::directive(Directive::Endif),
        Statement::if_then(
            Expr::equal(Expr::identifier("resized"), Expr::identifier("NULL")),
            trap("allocation failed"),
        ),
        Statement::assignment(
            Expr::identifier("resized").pointer_field("capacity"),
            Expr::identifier("capacity"),
        ),
        Statement::assignment(
            Expr::identifier("admission").pointer_field("state"),
            Expr::cast(
                TypeName::named("void").pointer(),
                Expr::add(Expr::identifier("resized"), Expr::number("1")),
            ),
        ),
    ]);
    append_function(
        output,
        FunctionSignature::new(
            "void",
            "mal_SymbolAdmission_reserve",
            [
                context_parameter(),
                Parameter::named(TypeName::named("MalSymbolAdmission").pointer(), "admission"),
                Parameter::named("uint64_t", "minimum_capacity"),
            ],
        ),
        Block::new([
            Statement::variable(
                "size_t",
                "size",
                Some(Expr::cast("size_t", Expr::identifier("minimum_capacity"))),
            ),
            Statement::if_then(
                Expr::not_equal(
                    Expr::cast("uint64_t", Expr::identifier("size")),
                    Expr::identifier("minimum_capacity"),
                ),
                trap("allocation size overflow"),
            ),
            Statement::if_else(
                Expr::equal(
                    Expr::identifier("admission").pointer_field("state"),
                    Expr::identifier("NULL"),
                ),
                Block::new([Statement::if_then(
                    Expr::not_equal(Expr::identifier("size"), Expr::number("0")),
                    Block::new([Statement::assignment(
                        Expr::identifier("admission").pointer_field("state"),
                        Expr::named_call(
                            "mal_allocate",
                            [Expr::identifier("context"), Expr::identifier("size")],
                        ),
                    )]),
                )]),
                resize,
            ),
            Statement::label(
                "mal_admission_reserve_done",
                Block::new([Statement::expression(Expr::cast("void", Expr::number("0")))]),
            ),
        ]),
    );
    append_function(
        output,
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_SymbolAdmission_finish",
            [
                context_parameter(),
                Parameter::named(TypeName::named("MalSymbolAdmission").pointer(), "admission"),
                Parameter::named("uint64_t", "length"),
            ],
        ),
        Block::new([
            Statement::if_then(
                Expr::greater(
                    Expr::identifier("length"),
                    Expr::named_call(
                        "mal_SymbolAdmission_capacity",
                        [Expr::identifier("admission")],
                    ),
                ),
                trap("Symbol admission length exceeds capacity"),
            ),
            Statement::variable(
                TypeName::named("uint8_t").pointer(),
                "data",
                Some(Expr::cast(
                    TypeName::named("uint8_t").pointer(),
                    Expr::identifier("admission").pointer_field("state"),
                )),
            ),
            Statement::assignment(
                Expr::identifier("admission").pointer_field("state"),
                Expr::identifier("NULL"),
            ),
            Statement::if_then(
                Expr::equal(Expr::identifier("length"), uint64(0)),
                Block::new([
                    Statement::if_then(
                        Expr::not_equal(Expr::identifier("data"), Expr::identifier("NULL")),
                        Block::new([Statement::call(
                            "mal_deallocate",
                            [Expr::identifier("data")],
                        )]),
                    ),
                    Statement::return_value(symbol([
                        Expr::identifier("NULL"),
                        uint64(0),
                        Expr::identifier("NULL"),
                    ])),
                ]),
            ),
            Statement::return_value(symbol([
                Expr::identifier("data"),
                Expr::identifier("length"),
                Expr::identifier("data"),
            ])),
        ]),
    );
    append_function(
        output,
        FunctionSignature::new(
            "void",
            "mal_SymbolAdmission_drop",
            [
                context_parameter(),
                Parameter::named(TypeName::named("MalSymbolAdmission").pointer(), "admission"),
            ],
        ),
        Block::new([
            Statement::expression(Expr::cast("void", Expr::identifier("context"))),
            Statement::if_then(
                Expr::not_equal(
                    Expr::identifier("admission").pointer_field("state"),
                    Expr::identifier("NULL"),
                ),
                Block::new([Statement::call(
                    "mal_deallocate",
                    [Expr::identifier("admission").pointer_field("state")],
                )]),
            ),
            Statement::assignment(
                Expr::identifier("admission").pointer_field("state"),
                Expr::identifier("NULL"),
            ),
        ]),
    );
}

fn append_symbol_copy(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::static_function(
            "MalType_Symbol",
            "mal_symbol_copy_from_bytes",
            [
                context_parameter(),
                Parameter::named(TypeName::const_named("uint8_t").pointer(), "data"),
                Parameter::named("uint64_t", "length"),
            ],
        ),
        Block::new([
            Statement::if_then(
                Expr::equal(Expr::identifier("length"), uint64(0)),
                Block::new([Statement::return_value(symbol([
                    Expr::identifier("NULL"),
                    uint64(0),
                    Expr::identifier("NULL"),
                ]))]),
            ),
            Statement::if_then(
                Expr::equal(Expr::identifier("data"), Expr::identifier("NULL")),
                trap("null Symbol data"),
            ),
            Statement::variable(
                "size_t",
                "size",
                Some(Expr::cast("size_t", Expr::identifier("length"))),
            ),
            Statement::if_then(
                Expr::not_equal(
                    Expr::cast("uint64_t", Expr::identifier("size")),
                    Expr::identifier("length"),
                ),
                trap("allocation size overflow"),
            ),
            Statement::variable(
                TypeName::named("uint8_t").pointer(),
                "copy",
                Some(Expr::cast(
                    TypeName::named("uint8_t").pointer(),
                    Expr::named_call(
                        "mal_allocate",
                        [Expr::identifier("context"), Expr::identifier("size")],
                    ),
                )),
            ),
            Statement::call(
                "memcpy",
                [
                    Expr::identifier("copy"),
                    Expr::identifier("data"),
                    Expr::identifier("size"),
                ],
            ),
            Statement::return_value(symbol([
                Expr::identifier("copy"),
                Expr::identifier("length"),
                Expr::identifier("copy"),
            ])),
        ]),
    );
}

fn append_symbol_materialization(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::static_function(
            "uint8_t",
            "mal_symbol_copy_into",
            [
                Parameter::named("MalType_Symbol", "value"),
                Parameter::named(TypeName::named("uint8_t").pointer(), "bytes"),
            ],
        ),
        Block::new([
            Statement::if_then(
                Expr::logical_and(
                    Expr::not_equal(
                        Expr::identifier("value").field("ownership"),
                        Expr::identifier("NULL"),
                    ),
                    Expr::equal(
                        allocation_for(Expr::identifier("value").field("ownership"))
                            .pointer_field("capacity"),
                        Expr::identifier("SIZE_MAX"),
                    ),
                ),
                Block::new([
                    Statement::variable(
                        TypeName::const_named("MalSymbolRope").pointer(),
                        "rope",
                        Some(Expr::cast(
                            TypeName::const_named("MalSymbolRope").pointer(),
                            Expr::identifier("value").field("ownership"),
                        )),
                    ),
                    Statement::call(
                        "mal_symbol_copy_into",
                        [
                            Expr::identifier("rope").pointer_field("left"),
                            Expr::identifier("bytes"),
                        ],
                    ),
                    Statement::call(
                        "mal_symbol_copy_into",
                        [
                            Expr::identifier("rope").pointer_field("right"),
                            Expr::add(
                                Expr::identifier("bytes"),
                                Expr::cast(
                                    "size_t",
                                    Expr::identifier("rope")
                                        .pointer_field("left")
                                        .field("length"),
                                ),
                            ),
                        ],
                    ),
                    Statement::return_value(uint8(0)),
                ]),
            ),
            Statement::if_then(
                Expr::not_equal(Expr::identifier("value").field("length"), uint64(0)),
                Block::new([Statement::call(
                    "memcpy",
                    [
                        Expr::identifier("bytes"),
                        Expr::identifier("value").field("data"),
                        Expr::cast("size_t", Expr::identifier("value").field("length")),
                    ],
                )]),
            ),
            Statement::return_value(uint8(0)),
        ]),
    );
    append_function(
        output,
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_symbol_materialize",
            [
                context_parameter(),
                Parameter::named("MalType_Symbol", "value"),
            ],
        ),
        Block::new([
            Statement::directive(Directive::If(PreprocessorExpr::defined(
                "MAL_TEST_MATERIALIZATION_LIMIT",
            ))),
            Statement::expression(Expr::pre_increment(Expr::identifier(
                "mal_test_materialization_count",
            ))),
            Statement::directive(Directive::Endif),
            Statement::if_then(
                Expr::logical_or(
                    Expr::not_equal(
                        Expr::identifier("value").field("data"),
                        Expr::identifier("NULL"),
                    ),
                    Expr::equal(Expr::identifier("value").field("length"), uint64(0)),
                ),
                Block::new([Statement::return_value(Expr::identifier("value"))]),
            ),
            Statement::variable(
                TypeName::named("MalSymbolRope").pointer(),
                "rope",
                Some(Expr::cast(
                    TypeName::named("MalSymbolRope").pointer(),
                    Expr::identifier("value").field("ownership"),
                )),
            ),
            Statement::if_then(
                Expr::equal(
                    Expr::identifier("rope").pointer_field("flattened"),
                    Expr::identifier("NULL"),
                ),
                Block::new([
                    Statement::variable(
                        "size_t",
                        "size",
                        Some(Expr::cast(
                            "size_t",
                            Expr::identifier("value").field("length"),
                        )),
                    ),
                    Statement::if_then(
                        Expr::not_equal(
                            Expr::cast("uint64_t", Expr::identifier("size")),
                            Expr::identifier("value").field("length"),
                        ),
                        trap("allocation size overflow"),
                    ),
                    Statement::assignment(
                        Expr::identifier("rope").pointer_field("flattened"),
                        Expr::cast(
                            TypeName::named("uint8_t").pointer(),
                            Expr::named_call(
                                "mal_allocate",
                                [Expr::identifier("context"), Expr::identifier("size")],
                            ),
                        ),
                    ),
                    Statement::call(
                        "mal_symbol_copy_into",
                        [
                            Expr::identifier("value"),
                            Expr::identifier("rope").pointer_field("flattened"),
                        ],
                    ),
                ]),
            ),
            Statement::return_value(symbol([
                Expr::identifier("rope").pointer_field("flattened"),
                Expr::identifier("value").field("length"),
                Expr::identifier("value").field("ownership"),
            ])),
        ]),
    );
}

fn allocation_for(value: Expr) -> Expr {
    Expr::subtract(
        Expr::cast(TypeName::named("MalAllocation").pointer(), value),
        Expr::number("1"),
    )
}

fn context_parameter() -> Parameter {
    Parameter::named(TypeName::named("MalContext").pointer(), "context")
}

fn trap(message: &str) -> Block {
    Block::new([Statement::call(
        "mal_trap",
        [Expr::identifier("context"), Expr::string(message)],
    )])
}

fn resource_failure(message: &str) -> Block {
    Block::new([Statement::call(
        "mal_resource_failure",
        [Expr::string(message)],
    )])
}

fn symbol(fields: impl IntoIterator<Item = Expr>) -> Expr {
    Expr::compound_literal(
        "MalType_Symbol",
        fields.into_iter().map(Initializer::positional),
    )
}

fn uint8(value: u8) -> Expr {
    Expr::named_call("UINT8_C", [Expr::number(value.to_string())])
}

fn uint64(value: u64) -> Expr {
    Expr::named_call("UINT64_C", [Expr::number(value.to_string())])
}

fn live_allocation_tracking() -> PreprocessorExpr {
    PreprocessorExpr::logical_or(
        PreprocessorExpr::defined("MAL_TEST_LIVE_ALLOCATION_LIMIT"),
        PreprocessorExpr::defined("MAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS"),
    )
}

fn push_test_counter_limit_check(block: &mut Block, counter: &str, limit: &str, message: &str) {
    block.push(Statement::directive(Directive::If(
        PreprocessorExpr::defined(limit),
    )));
    block.push(Statement::if_then(
        Expr::greater(
            Expr::identifier(counter),
            Expr::cast("size_t", Expr::identifier(limit)),
        ),
        trap(message),
    ));
    block.push(Statement::directive(Directive::Endif));
}

fn append_function(output: &mut TranslationUnit, signature: FunctionSignature, body: Block) {
    output.push(FunctionDefinition::from_signature(signature, body));
    output.blank_line();
}
