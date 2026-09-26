macro_rules! c_switch_case {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (case $label:tt; $body:tt) => {
        $crate::backend::c::syntax::SwitchCase::case(
            $crate::backend::c::syntax::c_expr_child!($label),
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
    ($cases:ident; {{ $($rust:tt)* }}) => {
        $cases.extend({ $($rust)* })
    };
    ($cases:ident; { $($rust:tt)* }) => {
        $cases.push({ $($rust)* })
    };
    ($cases:ident; $case:tt) => {
        $cases.push($crate::backend::c::syntax::c_switch_case! $case)
    };
}

macro_rules! c_statement {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (expr $value:tt) => {
        $crate::backend::c::syntax::Statement::expression(
            $crate::backend::c::syntax::c_expr_child!($value),
        )
    };
    (call $name:tt; $($argument:tt),* $(,)?) => {
        $crate::backend::c::syntax::Statement::expression($crate::backend::c::syntax::c_expr!(
            call $name; $($argument),*
        ))
    };
    (invoke $callee:tt; $($argument:tt),* $(,)?) => {
        $crate::backend::c::syntax::Statement::expression($crate::backend::c::syntax::c_expr!(
            invoke $callee; $($argument),*
        ))
    };
    (var $name:tt : { $($ty:tt)* }) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            $crate::backend::c::syntax::c_variable!($name : { $($ty)* }),
            None,
        )
    };
    (var $name:tt : $kind:ident($($ty:tt)*)) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            $crate::backend::c::syntax::c_variable!($name : $kind($($ty)*)),
            None,
        )
    };
    (var $name:tt : { $($ty:tt)* } = $value:tt) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            $crate::backend::c::syntax::c_variable!($name : { $($ty)* }),
            Some($crate::backend::c::syntax::c_expr_child!($value)),
        )
    };
    (var $name:tt : $kind:ident($($ty:tt)*) = $value:tt) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            $crate::backend::c::syntax::c_variable!($name : $kind($($ty)*)),
            Some($crate::backend::c::syntax::c_expr_child!($value)),
        )
    };
    (declaration { $($declaration:tt)* }) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            { $($declaration)* },
            None,
        )
    };
    (declaration { $($declaration:tt)* } = $value:tt) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            { $($declaration)* },
            Some($crate::backend::c::syntax::c_expr_child!($value)),
        )
    };
    (return $value:tt) => {
        $crate::backend::c::syntax::Statement::return_value(
            $crate::backend::c::syntax::c_expr_child!($value),
        )
    };
    (return_void) => {
        $crate::backend::c::syntax::Statement::return_void()
    };
    (if $condition:tt; $body:tt) => {
        $crate::backend::c::syntax::Statement::if_then(
            $crate::backend::c::syntax::c_expr_child!($condition),
            $crate::backend::c::syntax::c_block! $body,
        )
    };
    (switch $value:tt; $cases:tt) => {
        $crate::backend::c::syntax::Statement::switch(
            $crate::backend::c::syntax::c_expr_child!($value),
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
    ($block:ident; {{ $($rust:tt)* }}) => {
        $block.extend({ $($rust)* })
    };
    ($block:ident; { $($rust:tt)* }) => {
        $block.push({ $($rust)* })
    };
    ($block:ident; $statement:tt) => {
        $block.push($crate::backend::c::syntax::c_statement! $statement)
    };
}

pub(in crate::backend) use c_block;
pub(in crate::backend) use c_statement;
pub(in crate::backend) use {c_block_item, c_switch_case, c_switch_cases, c_switch_cases_item};
