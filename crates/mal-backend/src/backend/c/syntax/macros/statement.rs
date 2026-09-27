macro_rules! c_switch_case_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (_ => { $($body:tt)* }) => {
        $crate::backend::c::syntax::SwitchCase::default(
            $crate::backend::c::syntax::c_block_normalized!( $($body)* ),
        )
    };
    ($label:tt => { $($body:tt)* }) => {
        $crate::backend::c::syntax::SwitchCase::case(
            $crate::backend::c::syntax::c_expr_child_normalized!($label),
            $crate::backend::c::syntax::c_block_normalized!( $($body)* ),
        )
    };
}

macro_rules! c_switch_cases_normalized {
    ({ $($case:tt)* }) => {{
        #[allow(unused_mut)]
        let mut cases = Vec::from([]);
        $crate::backend::c::syntax::c_switch_cases_items_normalized!(cases; $($case)*);
        cases
    }};
}

macro_rules! c_switch_cases_items_normalized {
    ($cases:ident;) => {};
    ($cases:ident; ...(@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $cases.extend({ $($rust)* });
        $crate::backend::c::syntax::c_switch_cases_items_normalized!($cases; $($($rest)*)?);
    };
    ($cases:ident; (@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $cases.push({ $($rust)* });
        $crate::backend::c::syntax::c_switch_cases_items_normalized!($cases; $($($rest)*)?);
    };
    ($cases:ident; _ => { $($body:tt)* } $(, $($rest:tt)*)?) => {
        $cases.push($crate::backend::c::syntax::c_switch_case_normalized!(_ => { $($body)* }));
        $crate::backend::c::syntax::c_switch_cases_items_normalized!($cases; $($($rest)*)?);
    };
    ($cases:ident; $label:tt => { $($body:tt)* } $(, $($rest:tt)*)?) => {
        $cases.push($crate::backend::c::syntax::c_switch_case_normalized!($label => { $($body)* }));
        $crate::backend::c::syntax::c_switch_cases_items_normalized!($cases; $($($rest)*)?);
    };
}

macro_rules! c_statement_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (let $name:tt : $kind:ident($($ty:tt)*);) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            $crate::backend::c::syntax::c_variable_normalized!($name : $kind($($ty)*)),
            None,
        )
    };
    (let $name:tt : (@rust $($ty:tt)*);) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            $crate::backend::c::syntax::c_variable_normalized!($name : (@rust $($ty)*)),
            None,
        )
    };
    (let $name:tt : $kind:ident($($ty:tt)*) = $value:tt;) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            $crate::backend::c::syntax::c_variable_normalized!($name : $kind($($ty)*)),
            Some($crate::backend::c::syntax::c_expr_child_normalized!($value)),
        )
    };
    (let $name:tt : (@rust $($ty:tt)*) = $value:tt;) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            $crate::backend::c::syntax::c_variable_normalized!($name : (@rust $($ty)*)),
            Some($crate::backend::c::syntax::c_expr_child_normalized!($value)),
        )
    };
    (let (@rust $($declaration:tt)*);) => {
        $crate::backend::c::syntax::Statement::variable_declaration({ $($declaration)* }, None)
    };
    (let (@rust $($declaration:tt)*) = $value:tt;) => {
        $crate::backend::c::syntax::Statement::variable_declaration(
            { $($declaration)* },
            Some($crate::backend::c::syntax::c_expr_child_normalized!($value)),
        )
    };
    (return $value:tt;) => {
        $crate::backend::c::syntax::Statement::return_value(
            $crate::backend::c::syntax::c_expr_child_normalized!($value),
        )
    };
    (return;) => {
        $crate::backend::c::syntax::Statement::return_void()
    };
    (if $condition:tt { $($body:tt)* }) => {
        $crate::backend::c::syntax::Statement::if_then(
            $crate::backend::c::syntax::c_expr_child_normalized!($condition),
            $crate::backend::c::syntax::c_block_normalized!( $($body)* ),
        )
    };
    (switch $value:tt { $($case:tt)* }) => {
        $crate::backend::c::syntax::Statement::switch(
            $crate::backend::c::syntax::c_expr_child_normalized!($value),
            $crate::backend::c::syntax::c_switch_cases_normalized!({ $($case)* }),
        )
    };
    ($kind:ident($($argument:tt)*);) => {
        $crate::backend::c::syntax::Statement::expression(
            $crate::backend::c::syntax::c_expr_normalized!($kind($($argument)*)),
        )
    };
    ((@rust $($value:tt)*);) => {
        $crate::backend::c::syntax::Statement::expression({ $($value)* })
    };
}

macro_rules! c_block_normalized {
    ($($statement:tt)*) => {{
        let mut block = $crate::backend::c::syntax::Block::default();
        $crate::backend::c::syntax::c_block_items_normalized!(block; $($statement)*);
        block
    }};
}

macro_rules! c_block_items_normalized {
    ($block:ident;) => {};
    ($block:ident; ...(@rust $($rust:tt)*) $($rest:tt)*) => {
        $block.extend({ $($rust)* });
        $crate::backend::c::syntax::c_block_items_normalized!($block; $($rest)*);
    };
    ($block:ident; (@rust $($rust:tt)*) $($rest:tt)*) => {
        $block.push({ $($rust)* });
        $crate::backend::c::syntax::c_block_items_normalized!($block; $($rest)*);
    };
    ($block:ident; let $name:tt : $kind:ident($($ty:tt)*); $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement_normalized!(let $name : $kind($($ty)*);));
        $crate::backend::c::syntax::c_block_items_normalized!($block; $($rest)*);
    };
    ($block:ident; let $name:tt : (@rust $($ty:tt)*); $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement_normalized!(let $name : (@rust $($ty)*);));
        $crate::backend::c::syntax::c_block_items_normalized!($block; $($rest)*);
    };
    ($block:ident; let $name:tt : $kind:ident($($ty:tt)*) = $value:tt; $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement_normalized! {
            let $name : $kind($($ty)*) = $value;
        });
        $crate::backend::c::syntax::c_block_items_normalized!($block; $($rest)*);
    };
    ($block:ident; let $name:tt : (@rust $($ty:tt)*) = $value:tt; $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement_normalized! {
            let $name : (@rust $($ty)*) = $value;
        });
        $crate::backend::c::syntax::c_block_items_normalized!($block; $($rest)*);
    };
    ($block:ident; let (@rust $($declaration:tt)*); $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement_normalized!(let (@rust $($declaration)*);));
        $crate::backend::c::syntax::c_block_items_normalized!($block; $($rest)*);
    };
    ($block:ident; let (@rust $($declaration:tt)*) = $value:tt; $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement_normalized! {
            let (@rust $($declaration)*) = $value;
        });
        $crate::backend::c::syntax::c_block_items_normalized!($block; $($rest)*);
    };
    ($block:ident; return; $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement_normalized!(return;));
        $crate::backend::c::syntax::c_block_items_normalized!($block; $($rest)*);
    };
    ($block:ident; return $value:tt; $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement_normalized!(return $value;));
        $crate::backend::c::syntax::c_block_items_normalized!($block; $($rest)*);
    };
    ($block:ident; if $condition:tt { $($body:tt)* } $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement_normalized!(if $condition { $($body)* }));
        $crate::backend::c::syntax::c_block_items_normalized!($block; $($rest)*);
    };
    ($block:ident; switch $value:tt { $($case:tt)* } $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement_normalized!(switch $value { $($case)* }));
        $crate::backend::c::syntax::c_block_items_normalized!($block; $($rest)*);
    };
    ($block:ident; $kind:ident($($argument:tt)*); $($rest:tt)*) => {
        $block.push($crate::backend::c::syntax::c_statement_normalized!($kind($($argument)*);));
        $crate::backend::c::syntax::c_block_items_normalized!($block; $($rest)*);
    };
}

macro_rules! c_switch_case {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_switch_case_normalized]; $($syntax)*
        )
    };
}

macro_rules! c_statement {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_statement_normalized]; $($syntax)*
        )
    };
}

macro_rules! c_block {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_block_normalized]; $($syntax)*
        )
    };
}

pub(in crate::backend) use {
    c_block_items_normalized, c_block_normalized, c_statement_normalized, c_switch_case_normalized,
    c_switch_cases_items_normalized, c_switch_cases_normalized,
};

pub(in crate::backend) use {c_block, c_statement, c_switch_case};
