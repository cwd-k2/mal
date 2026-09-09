use super::*;

pub(super) fn append(output: &mut TranslationUnit) {
    let control = Expr::identifier("control");
    let capacity_field = control.clone().pointer_field("capacity");
    let top = control.clone().pointer_field("top");
    let mut grow = Block::new([
        Statement::variable("size_t", "capacity", Some(capacity_field.clone())),
        Statement::if_else(
            Expr::equal(Expr::identifier("capacity"), Expr::number("0")),
            Block::new([
                Statement::assignment(
                    Expr::identifier("capacity"),
                    Expr::divide(Expr::number("256"), Expr::identifier("size")),
                ),
                Statement::if_then(
                    Expr::equal(Expr::identifier("capacity"), Expr::number("0")),
                    Block::new([Statement::assignment(
                        Expr::identifier("capacity"),
                        Expr::number("1"),
                    )]),
                ),
            ]),
            Block::new([
                Statement::if_then(
                    Expr::greater(
                        Expr::identifier("capacity"),
                        Expr::divide(Expr::identifier("SIZE_MAX"), Expr::number("2")),
                    ),
                    control_failure("control stack size overflow"),
                ),
                Statement::assignment(
                    Expr::identifier("capacity"),
                    Expr::multiply(Expr::identifier("capacity"), Expr::number("2")),
                ),
            ]),
        ),
        Statement::if_then(
            Expr::greater(
                Expr::identifier("capacity"),
                Expr::divide(Expr::identifier("SIZE_MAX"), Expr::identifier("size")),
            ),
            control_failure("control stack size overflow"),
        ),
        Statement::directive(Directive::If(PreprocessorExpr::defined(
            "MAL_TEST_FORCE_CONTROL_ALLOCATION_FAILURE",
        ))),
        Statement::variable(
            TypeName::named("void").pointer(),
            "storage",
            Some(Expr::identifier("NULL")),
        ),
        Statement::directive(Directive::Else),
        Statement::variable(
            TypeName::named("void").pointer(),
            "storage",
            Some(Expr::named_call(
                "realloc",
                [
                    control.clone().pointer_field("storage"),
                    Expr::multiply(Expr::identifier("capacity"), Expr::identifier("size")),
                ],
            )),
        ),
        Statement::directive(Directive::Endif),
        Statement::if_then(
            Expr::equal(Expr::identifier("storage"), Expr::identifier("NULL")),
            control_failure("control stack allocation failed"),
        ),
        Statement::assignment(
            control.clone().pointer_field("storage"),
            Expr::cast(
                TypeName::named("uint8_t").pointer(),
                Expr::identifier("storage"),
            ),
        ),
        Statement::assignment(capacity_field, Expr::identifier("capacity")),
    ]);
    let mut body = Block::new([Statement::if_then(
        Expr::equal(top.clone(), control.clone().pointer_field("capacity")),
        std::mem::take(&mut grow),
    )]);
    body.push(Statement::variable(
        TypeName::named("void").pointer(),
        "frame",
        Some(Expr::add(
            control.pointer_field("storage"),
            Expr::multiply(top.clone(), Expr::identifier("size")),
        )),
    ));
    body.push(Statement::assignment(
        top.clone(),
        Expr::add(top, Expr::number("1")),
    ));
    body.push(Statement::return_value(Expr::identifier("frame")));
    append_function(
        output,
        FunctionSignature::static_inline(
            TypeName::named("void").pointer(),
            "mal_control_push_homogeneous",
            [
                Parameter::named(TypeName::named("MalControlStack").pointer(), "control"),
                Parameter::named("size_t", "size"),
            ],
        )
        .maybe_unused(),
        body,
    );
}
