use crate::c_emit::syntax::{
    AggregateDefinition, AggregateField, Block, Declaration, Directive, Expr, FunctionDefinition,
    FunctionSignature, Initializer, Parameter, PreprocessorExpr, Statement, TranslationUnit,
    TypeName, VariableDeclaration,
};

mod allocation;
mod control;
mod symbol;

use allocation::{append_allocation, append_reference_counting, append_resource_failure};
use control::{append_control_stack, control_arena_name};
use symbol::{append_symbol_copy, append_symbol_lifetime, append_symbol_materialization};

pub(super) fn emit(
    control_arenas: usize,
    homogeneous_control: bool,
    heterogeneous_control: bool,
) -> TranslationUnit {
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
        let mut stack_fields = vec![
            AggregateField::variable(TypeName::named("uint8_t").pointer(), "storage"),
            AggregateField::variable("size_t", "capacity"),
            AggregateField::variable("size_t", "top"),
        ];
        if heterogeneous_control {
            stack_fields.push(AggregateField::variable("size_t", "frame"));
        }
        output.push(AggregateDefinition::typedef_structure(
            None,
            stack_fields,
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
        append_control_stack(&mut output, homogeneous_control, heterogeneous_control);
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
    append_symbol_copy(&mut output);
    append_symbol_materialization(&mut output);
    output
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
