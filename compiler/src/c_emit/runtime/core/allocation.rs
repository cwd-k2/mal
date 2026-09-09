use super::*;

pub(super) fn append_resource_failure(output: &mut TranslationUnit) {
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

pub(super) fn append_allocation(output: &mut TranslationUnit) {
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

pub(super) fn append_reference_counting(output: &mut TranslationUnit) {
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
            Statement::directive(Directive::If(PreprocessorExpr::defined(
                "MAL_TEST_FORCE_REFERENCE_COUNT_OVERFLOW",
            ))),
            Statement::assignment(
                Expr::identifier("allocation").pointer_field("references"),
                Expr::identifier("UINT64_MAX"),
            ),
            Statement::directive(Directive::Endif),
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
