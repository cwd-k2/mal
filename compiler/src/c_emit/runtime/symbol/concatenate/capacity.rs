use crate::c_emit::syntax::{Block, Directive, Expr, PreprocessorExpr, Statement, TypeName};

use super::{append_right_bytes, symbol_result, symbol_result_with, trap};

pub(super) fn unique_concatenation(allocation: Expr) -> Block {
    let resize = Block::new([
        Statement::variable(
            "size_t",
            "maximum_capacity",
            Some(Expr::subtract(
                Expr::identifier("SIZE_MAX"),
                Expr::sizeof_type("MalAllocation"),
            )),
        ),
        Statement::variable(
            "size_t",
            "capacity",
            Some(Expr::identifier("allocation").pointer_field("capacity")),
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
        Statement::assignment(Expr::identifier("allocation"), Expr::identifier("resized")),
        Statement::assignment(
            Expr::identifier("allocation").pointer_field("capacity"),
            Expr::identifier("capacity"),
        ),
    ]);
    Block::new([
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
        Statement::variable(
            TypeName::named("MalAllocation").pointer(),
            "allocation",
            Some(allocation),
        ),
        Statement::variable(
            "size_t",
            "offset",
            Some(Expr::cast(
                "size_t",
                Expr::subtract(
                    Expr::identifier("left").field("data"),
                    Expr::cast(
                        TypeName::named("uint8_t").pointer(),
                        Expr::add(Expr::identifier("allocation"), Expr::number("1")),
                    ),
                ),
            )),
        ),
        Statement::if_then(
            Expr::less(
                Expr::identifier("allocation").pointer_field("capacity"),
                Expr::identifier("size"),
            ),
            resize,
        ),
        Statement::variable(
            TypeName::named("uint8_t").pointer(),
            "bytes",
            Some(Expr::cast(
                TypeName::named("uint8_t").pointer(),
                Expr::add(Expr::identifier("allocation"), Expr::number("1")),
            )),
        ),
        Statement::if_then(
            Expr::not_equal(Expr::identifier("offset"), Expr::number("0")),
            Block::new([Statement::call(
                "memmove",
                [
                    Expr::identifier("bytes"),
                    Expr::add(Expr::identifier("bytes"), Expr::identifier("offset")),
                    Expr::cast("size_t", Expr::identifier("left").field("length")),
                ],
            )]),
        ),
        append_right_bytes(),
        Statement::return_value(symbol_result()),
    ])
}

pub(super) fn unique_prepend(allocation: Expr) -> Block {
    let grow = Block::new([
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
        Statement::assignment(Expr::identifier("allocation"), Expr::identifier("resized")),
        Statement::assignment(
            Expr::identifier("allocation").pointer_field("capacity"),
            Expr::identifier("capacity"),
        ),
    ]);
    let reposition = Block::new([
        Statement::variable(
            "size_t",
            "capacity",
            Some(Expr::identifier("allocation").pointer_field("capacity")),
        ),
        Statement::if_then(
            Expr::less(Expr::identifier("capacity"), Expr::identifier("size")),
            grow,
        ),
        Statement::variable(
            TypeName::named("uint8_t").pointer(),
            "allocation_bytes",
            Some(Expr::cast(
                TypeName::named("uint8_t").pointer(),
                Expr::add(Expr::identifier("allocation"), Expr::number("1")),
            )),
        ),
        Statement::variable(
            "size_t",
            "new_offset",
            Some(Expr::subtract(
                Expr::identifier("capacity"),
                Expr::identifier("size"),
            )),
        ),
        Statement::call(
            "memmove",
            [
                Expr::add(
                    Expr::identifier("allocation_bytes"),
                    Expr::add(
                        Expr::identifier("new_offset"),
                        Expr::cast("size_t", Expr::identifier("left").field("length")),
                    ),
                ),
                Expr::add(
                    Expr::identifier("allocation_bytes"),
                    Expr::identifier("offset"),
                ),
                Expr::cast("size_t", Expr::identifier("right").field("length")),
            ],
        ),
        Statement::assignment(
            Expr::identifier("bytes"),
            Expr::add(
                Expr::identifier("allocation_bytes"),
                Expr::identifier("new_offset"),
            ),
        ),
    ]);
    Block::new([
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
        Statement::variable(
            TypeName::named("MalAllocation").pointer(),
            "allocation",
            Some(allocation),
        ),
        Statement::variable(
            TypeName::named("uint8_t").pointer(),
            "allocation_bytes",
            Some(Expr::cast(
                TypeName::named("uint8_t").pointer(),
                Expr::add(Expr::identifier("allocation"), Expr::number("1")),
            )),
        ),
        Statement::variable(
            "size_t",
            "offset",
            Some(Expr::cast(
                "size_t",
                Expr::subtract(
                    Expr::identifier("right").field("data"),
                    Expr::identifier("allocation_bytes"),
                ),
            )),
        ),
        Statement::variable(
            TypeName::named("uint8_t").pointer(),
            "bytes",
            Some(Expr::cast(
                TypeName::named("uint8_t").pointer(),
                Expr::identifier("right").field("data"),
            )),
        ),
        Statement::if_else(
            Expr::less(
                Expr::identifier("offset"),
                Expr::cast("size_t", Expr::identifier("left").field("length")),
            ),
            reposition,
            Block::new([Statement::assignment(
                Expr::identifier("bytes"),
                Expr::subtract(
                    Expr::identifier("bytes"),
                    Expr::cast("size_t", Expr::identifier("left").field("length")),
                ),
            )]),
        ),
        Statement::call(
            "memcpy",
            [
                Expr::identifier("bytes"),
                Expr::identifier("left").field("data"),
                Expr::cast("size_t", Expr::identifier("left").field("length")),
            ],
        ),
        Statement::return_value(symbol_result_with(
            Expr::identifier("bytes"),
            Expr::cast(
                TypeName::named("uint8_t").pointer(),
                Expr::add(Expr::identifier("allocation"), Expr::number("1")),
            ),
        )),
    ])
}

pub(super) fn allocation_for(value: Expr) -> Expr {
    Expr::subtract(
        Expr::cast(TypeName::named("MalAllocation").pointer(), value),
        Expr::number("1"),
    )
}
