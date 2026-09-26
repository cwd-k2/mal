macro_rules! c_initializer {
    (rust $initializer:expr) => {
        $initializer
    };
    (positional $value:tt) => {
        $crate::backend::c::syntax::Initializer::positional(
            $crate::backend::c::syntax::c_expr! $value,
        )
    };
    (field $name:expr; $value:tt) => {
        $crate::backend::c::syntax::Initializer::designated(
            $name,
            $crate::backend::c::syntax::c_expr! $value,
        )
    };
    (path $path:expr; $value:tt) => {
        $crate::backend::c::syntax::Initializer::designated_path(
            $path,
            $crate::backend::c::syntax::c_expr! $value,
        )
    };
}

macro_rules! c_initializers {
    ($($initializer:tt),* $(,)?) => {{
        let mut initializers = Vec::new();
        $(
            $crate::backend::c::syntax::c_initializers_item!(initializers; $initializer);
        )*
        initializers
    }};
}

macro_rules! c_initializers_item {
    ($initializers:ident; (extend $more:expr)) => {
        $initializers.extend($more)
    };
    ($initializers:ident; $initializer:tt) => {
        $initializers.extend([$crate::backend::c::syntax::c_initializer! $initializer])
    };
}

macro_rules! c_expr {
    (rust $expression:expr) => {
        $expression
    };
    (id $name:expr) => {
        $crate::backend::c::syntax::Expr::identifier($name)
    };
    (number $value:expr) => {
        $crate::backend::c::syntax::Expr::number($value.to_string())
    };
    (string $value:expr) => {
        $crate::backend::c::syntax::Expr::string($value)
    };
    (address $value:tt) => {
        $crate::backend::c::syntax::Expr::address_of($crate::backend::c::syntax::c_expr! $value)
    };
    (dereference $value:tt) => {
        $crate::backend::c::syntax::Expr::dereference($crate::backend::c::syntax::c_expr! $value)
    };
    (field $value:tt; $name:expr) => {
        ($crate::backend::c::syntax::c_expr! $value).field($name)
    };
    (pointer_field $value:tt; $name:expr) => {
        ($crate::backend::c::syntax::c_expr! $value).pointer_field($name)
    };
    (sizeof $value:tt) => {
        $crate::backend::c::syntax::Expr::sizeof_value($crate::backend::c::syntax::c_expr! $value)
    };
    (cast $ty:expr; $value:tt) => {
        $crate::backend::c::syntax::Expr::cast($ty, $crate::backend::c::syntax::c_expr! $value)
    };
    (call $name:expr; $($argument:tt),* $(,)?) => {
        $crate::backend::c::syntax::Expr::named_call(
            $name,
            [$($crate::backend::c::syntax::c_expr! $argument),*],
        )
    };
    (invoke $callee:tt; $($argument:tt),* $(,)?) => {
        $crate::backend::c::syntax::Expr::call(
            $crate::backend::c::syntax::c_expr! $callee,
            [$($crate::backend::c::syntax::c_expr! $argument),*],
        )
    };
    (add $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::add(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (subtract $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::subtract(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (multiply $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::multiply(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (assign $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::assign(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (equal $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::equal(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (not_equal $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::not_equal(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (greater $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::greater(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (logical_and $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::logical_and(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (conditional $condition:tt; $then:tt; $otherwise:tt) => {
        $crate::backend::c::syntax::Expr::conditional(
            $crate::backend::c::syntax::c_expr! $condition,
            $crate::backend::c::syntax::c_expr! $then,
            $crate::backend::c::syntax::c_expr! $otherwise,
        )
    };
    (initializer $($element:tt),* $(,)?) => {
        $crate::backend::c::syntax::Expr::initializer_list([
            $($crate::backend::c::syntax::c_expr! $element),*
        ])
    };
    (compound $ty:expr; $($initializer:tt),* $(,)?) => {
        $crate::backend::c::syntax::Expr::compound_literal(
            $ty,
            $crate::backend::c::syntax::c_initializers!($($initializer),*),
        )
    };
}

macro_rules! c_switch_case {
    (rust $case:expr) => {
        $case
    };
    (case $label:tt; $body:tt) => {
        $crate::backend::c::syntax::SwitchCase::case(
            $crate::backend::c::syntax::c_expr! $label,
            $crate::backend::c::syntax::c_block! $body,
        )
    };
    (default; $body:tt) => {
        $crate::backend::c::syntax::SwitchCase::default(
            $crate::backend::c::syntax::c_block! $body,
        )
    };
}

macro_rules! c_switch_cases {
    ($($case:tt),* $(,)?) => {{
        let mut cases = Vec::new();
        $(
            $crate::backend::c::syntax::c_switch_cases_item!(cases; $case);
        )*
        cases
    }};
}

macro_rules! c_switch_cases_item {
    ($cases:ident; (extend $more:expr)) => {
        $cases.extend($more)
    };
    ($cases:ident; $case:tt) => {
        $cases.push($crate::backend::c::syntax::c_switch_case! $case)
    };
}

macro_rules! c_statement {
    (rust $statement:expr) => {
        $statement
    };
    (expr $value:tt) => {
        $crate::backend::c::syntax::Statement::expression(
            $crate::backend::c::syntax::c_expr! $value,
        )
    };
    (call $name:expr; $($argument:tt),* $(,)?) => {
        $crate::backend::c::syntax::Statement::expression($crate::backend::c::syntax::c_expr!(
            call $name; $($argument),*
        ))
    };
    (invoke $callee:tt; $($argument:tt),* $(,)?) => {
        $crate::backend::c::syntax::Statement::expression($crate::backend::c::syntax::c_expr!(
            invoke $callee; $($argument),*
        ))
    };
    (var ($ty:expr) ($name:expr)) => {
        $crate::backend::c::syntax::Statement::variable($ty, $name, None)
    };
    (var ($ty:expr) ($name:expr) = $value:tt) => {
        $crate::backend::c::syntax::Statement::variable(
            $ty,
            $name,
            Some($crate::backend::c::syntax::c_expr! $value),
        )
    };
    (declaration ($declaration:expr)) => {
        $crate::backend::c::syntax::Statement::variable_declaration($declaration, None)
    };
    (declaration ($declaration:expr) = $value:tt) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            $declaration,
            Some($crate::backend::c::syntax::c_expr! $value),
        )
    };
    (return $value:tt) => {
        $crate::backend::c::syntax::Statement::return_value(
            $crate::backend::c::syntax::c_expr! $value,
        )
    };
    (return_void) => {
        $crate::backend::c::syntax::Statement::return_void()
    };
    (if $condition:tt; $body:tt) => {
        $crate::backend::c::syntax::Statement::if_then(
            $crate::backend::c::syntax::c_expr! $condition,
            $crate::backend::c::syntax::c_block! $body,
        )
    };
    (switch $value:tt; $cases:tt) => {
        $crate::backend::c::syntax::Statement::switch(
            $crate::backend::c::syntax::c_expr! $value,
            $crate::backend::c::syntax::c_switch_cases! $cases,
        )
    };
}

macro_rules! c_block {
    ($($statement:tt),* $(,)?) => {{
        let mut block = $crate::backend::c::syntax::Block::default();
        $(
            $crate::backend::c::syntax::c_block_item!(block; $statement);
        )*
        block
    }};
}

macro_rules! c_block_item {
    ($block:ident; (extend $statements:expr)) => {
        $block.extend($statements)
    };
    ($block:ident; $statement:tt) => {
        $block.push($crate::backend::c::syntax::c_statement! $statement)
    };
}

pub(in crate::backend) use c_block;
pub(in crate::backend) use c_expr;
pub(in crate::backend) use c_statement;
pub(in crate::backend) use {
    c_block_item, c_initializer, c_initializers, c_initializers_item, c_switch_case,
    c_switch_cases, c_switch_cases_item,
};

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::backend::c::syntax::{
        Expr, FunctionDefinition, FunctionSignature, Initializer, SwitchCase,
    };

    #[test]
    fn embeds_rust_expressions_once_inside_structured_expressions() {
        let evaluations = Cell::new(0);
        let dynamic = || {
            evaluations.set(evaluations.get() + 1);
            Expr::identifier("dynamic")
        };

        let expression = super::c_expr!(conditional
            (greater (id "count"); (number 0));
            (add (rust dynamic()); (number 1));
            (cast "size_t"; (number 0))
        );

        assert_eq!(evaluations.get(), 1);
        assert_eq!(
            expression.to_string(),
            "(count > 0) ? (dynamic + 1) : (size_t)0"
        );
    }

    #[test]
    fn constructs_compound_literals_from_static_and_rust_initializers() {
        let dynamic = Initializer::designated("second", Expr::number("2"));
        let trailing = [Initializer::positional(Expr::number("3"))];
        let expression = super::c_expr!(compound "Pair";
            (field "first"; (number 1)),
            (rust dynamic),
            (extend trailing),
        );

        assert_eq!(
            expression.to_string(),
            "(Pair){ .first = 1, .second = 2, 3 }"
        );
    }

    #[test]
    fn builds_nested_blocks_and_splices_runtime_node_sequences_in_order() {
        let statements = [super::c_statement!(call "observe"; (id "value"))];
        let cases = [SwitchCase::case(
            Expr::number("1"),
            super::c_block!((return (number 2))),
        )];
        let body = super::c_block!(
            (var ("int") ("value") = (number 0)),
            (extend statements),
            (if (equal (id "value"); (number 0)); [
                (switch (id "value"); [
                    (extend cases),
                    (default; [(return (number 3))]),
                ]),
            ]),
        );
        let function =
            FunctionDefinition::from_signature(FunctionSignature::new("int", "example", []), body);

        assert_eq!(
            function.render(),
            concat!(
                "int example(void) {\n",
                "    int value = 0;\n",
                "    observe(value);\n",
                "    if (value == 0) {\n",
                "        switch (value) {\n",
                "            case 1: {\n",
                "                return 2;\n",
                "            }\n",
                "            default: {\n",
                "                return 3;\n",
                "            }\n",
                "        }\n",
                "    }\n",
                "}\n",
            )
        );
    }
}
