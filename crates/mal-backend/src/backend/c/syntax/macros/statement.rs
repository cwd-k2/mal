macro_rules! c_switch_case {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (_ => { $($body:tt)* }) => {
        $crate::backend::c::syntax::SwitchCase::default(
            $crate::backend::c::syntax::c_block!({ $($body)* }),
        )
    };
    ($label:tt => { $($body:tt)* }) => {
        $crate::backend::c::syntax::SwitchCase::case(
            $crate::backend::c::syntax::c_expr_child!($label),
            $crate::backend::c::syntax::c_block!({ $($body)* }),
        )
    };
}

macro_rules! c_switch_cases {
    ({ $($case:tt)* }) => {{
        #[allow(unused_mut)]
        let mut cases = Vec::from([]);
        $crate::backend::c::syntax::c_switch_cases_items!(cases; $($case)*);
        cases
    }};
}

macro_rules! c_switch_cases_items {
    ($cases:ident;) => {};
    ($cases:ident; ...{{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $cases.extend({ $($rust)* });
        $crate::backend::c::syntax::c_switch_cases_items!($cases; $($($rest)*)?);
    };
    ($cases:ident; {{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $cases.push({ $($rust)* });
        $crate::backend::c::syntax::c_switch_cases_items!($cases; $($($rest)*)?);
    };
    ($cases:ident; _ => { $($body:tt)* } $(, $($rest:tt)*)?) => {
        $cases.push($crate::backend::c::syntax::c_switch_case!(_ => { $($body)* }));
        $crate::backend::c::syntax::c_switch_cases_items!($cases; $($($rest)*)?);
    };
    ($cases:ident; $label:tt => { $($body:tt)* } $(, $($rest:tt)*)?) => {
        $cases.push($crate::backend::c::syntax::c_switch_case!($label => { $($body)* }));
        $crate::backend::c::syntax::c_switch_cases_items!($cases; $($($rest)*)?);
    };
}

macro_rules! c_statement {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (let $name:tt : $kind:ident($($ty:tt)*);) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            $crate::backend::c::syntax::c_variable!($name : $kind($($ty)*)),
            None,
        )
    };
    (let $name:tt : {{ $($ty:tt)* }};) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            $crate::backend::c::syntax::c_variable!($name : {{ $($ty)* }}),
            None,
        )
    };
    (let $name:tt : $kind:ident($($ty:tt)*) = $value:tt;) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            $crate::backend::c::syntax::c_variable!($name : $kind($($ty)*)),
            Some($crate::backend::c::syntax::c_expr_child!($value)),
        )
    };
    (let $name:tt : {{ $($ty:tt)* }} = $value:tt;) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            $crate::backend::c::syntax::c_variable!($name : {{ $($ty)* }}),
            Some($crate::backend::c::syntax::c_expr_child!($value)),
        )
    };
    (let {{ $($declaration:tt)* }};) => {
        $crate::backend::c::syntax::Statement::variable_declaration({ $($declaration)* }, None)
    };
    (let {{ $($declaration:tt)* }} = $value:tt;) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            { $($declaration)* },
            Some($crate::backend::c::syntax::c_expr_child!($value)),
        )
    };
    (return $value:tt;) => {
        $crate::backend::c::syntax::Statement::return_value(
            $crate::backend::c::syntax::c_expr_child!($value),
        )
    };
    (return;) => {
        $crate::backend::c::syntax::Statement::return_void()
    };
    (if $condition:tt { $($body:tt)* }) => {
        $crate::backend::c::syntax::Statement::if_then(
            $crate::backend::c::syntax::c_expr_child!($condition),
            $crate::backend::c::syntax::c_block!({ $($body)* }),
        )
    };
    (switch $value:tt { $($case:tt)* }) => {
        $crate::backend::c::syntax::Statement::switch(
            $crate::backend::c::syntax::c_expr_child!($value),
            $crate::backend::c::syntax::c_switch_cases!({ $($case)* }),
        )
    };
    ($kind:ident($($argument:tt)*);) => {
        $crate::backend::c::syntax::Statement::expression(
            $crate::backend::c::syntax::c_expr!($kind($($argument)*)),
        )
    };
    ({{ $($value:tt)* }};) => {
        $crate::backend::c::syntax::Statement::expression({ $($value)* })
    };
}

macro_rules! c_block {
    ({ $($statement:tt)* }) => {{
        let mut block = $crate::backend::c::syntax::Block::default();
        $crate::backend::c::syntax::c_block_items!(block; $($statement)*);
        block
    }};
}

macro_rules! c_block_items {
    ($block:ident;) => {};
    ($block:ident; ...{{ $($rust:tt)* }} $($rest:tt)*) => {
        $block.extend({ $($rust)* });
        $crate::backend::c::syntax::c_block_items!($block; $($rest)*);
    };
    ($block:ident; {{ $($rust:tt)* }} $($rest:tt)*) => {
        $block.push({ $($rust)* });
        $crate::backend::c::syntax::c_block_items!($block; $($rest)*);
    };
    ($block:ident; let $name:tt : $kind:ident($($ty:tt)*); $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement!(let $name : $kind($($ty)*);));
        $crate::backend::c::syntax::c_block_items!($block; $($rest)*);
    };
    ($block:ident; let $name:tt : {{ $($ty:tt)* }}; $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement!(let $name : {{ $($ty)* }};));
        $crate::backend::c::syntax::c_block_items!($block; $($rest)*);
    };
    ($block:ident; let $name:tt : $kind:ident($($ty:tt)*) = $value:tt; $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement!(
            let $name : $kind($($ty)*) = $value;
        ));
        $crate::backend::c::syntax::c_block_items!($block; $($rest)*);
    };
    ($block:ident; let $name:tt : {{ $($ty:tt)* }} = $value:tt; $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement!(
            let $name : {{ $($ty)* }} = $value;
        ));
        $crate::backend::c::syntax::c_block_items!($block; $($rest)*);
    };
    ($block:ident; let {{ $($declaration:tt)* }}; $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement!(let {{ $($declaration)* }};));
        $crate::backend::c::syntax::c_block_items!($block; $($rest)*);
    };
    ($block:ident; let {{ $($declaration:tt)* }} = $value:tt; $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement!(
            let {{ $($declaration)* }} = $value;
        ));
        $crate::backend::c::syntax::c_block_items!($block; $($rest)*);
    };
    ($block:ident; return; $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement!(return;));
        $crate::backend::c::syntax::c_block_items!($block; $($rest)*);
    };
    ($block:ident; return $value:tt; $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement!(return $value;));
        $crate::backend::c::syntax::c_block_items!($block; $($rest)*);
    };
    ($block:ident; if $condition:tt { $($body:tt)* } $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement!(if $condition { $($body)* }));
        $crate::backend::c::syntax::c_block_items!($block; $($rest)*);
    };
    ($block:ident; switch $value:tt { $($case:tt)* } $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement!(switch $value { $($case)* }));
        $crate::backend::c::syntax::c_block_items!($block; $($rest)*);
    };
    ($block:ident; $kind:ident($($argument:tt)*); $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement!($kind($($argument)*);));
        $crate::backend::c::syntax::c_block_items!($block; $($rest)*);
    };
}

pub(in crate::backend) use {
    c_block, c_block_items, c_statement, c_switch_case, c_switch_cases, c_switch_cases_items,
};
