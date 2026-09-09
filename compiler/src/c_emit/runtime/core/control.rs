use super::*;

mod homogeneous;

pub(super) fn append_control_stack(
    output: &mut TranslationUnit,
    homogeneous: bool,
    heterogeneous: bool,
) {
    if heterogeneous {
        append_heterogeneous_control_push(output);
    }
    if homogeneous {
        if heterogeneous {
            output.blank_line();
        }
        homogeneous::append(output);
    }
}

fn append_heterogeneous_control_push(output: &mut TranslationUnit) {
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

pub(super) fn control_arena_name(arena: usize) -> String {
    format!("control_arena_{arena}")
}

fn control_failure(message: &str) -> Block {
    resource_failure(message)
}
