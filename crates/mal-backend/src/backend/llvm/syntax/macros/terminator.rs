macro_rules! llvm_switch_cases_item {
    ($cases:ident; ...{{ $($rust:tt)* }}) => {
        $cases.extend({ $($rust)* })
    };
    ($cases:ident; $value:tt => $target:tt) => {
        $cases.push((
            $crate::backend::llvm::syntax::llvm_scalar!($value).to_string(),
            $crate::backend::llvm::syntax::llvm_scalar!($target).to_string(),
        ))
    };
}
macro_rules! llvm_switch_cases {
    ($($case:tt)*) => {{
        #[allow(unused_mut)]
        let mut cases = Vec::from([]);
        $crate::backend::llvm::syntax::llvm_switch_cases_items!(cases; $($case)*);
        cases
    }};
}
macro_rules! llvm_switch_cases_items {
    ($cases:ident;) => {};
    ($cases:ident; ...{{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_switch_cases_item!($cases; ...{{ $($rust)* }});
        $crate::backend::llvm::syntax::llvm_switch_cases_items!($cases; $($($rest)*)?);
    };
    ($cases:ident; $value:tt => $target:tt $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_switch_cases_item!($cases; $value => $target);
        $crate::backend::llvm::syntax::llvm_switch_cases_items!($cases; $($($rest)*)?);
    };
}

macro_rules! llvm_terminator {
    ({{ $($rust:tt)* }}) => {
        Some({ $($rust)* })
    };
    (branch { target: $target:tt $(,)? };) => {
        $crate::backend::llvm::syntax::Terminator::branch(
            $crate::backend::llvm::syntax::llvm_scalar!($target),
        )
    };
    (branch {
        condition: $condition:tt,
        then: $then_target:tt,
        otherwise: $else_target:tt $(,)?
    };) => {
        $crate::backend::llvm::syntax::Terminator::conditional_branch(
            $crate::backend::llvm::syntax::llvm_scalar!($condition),
            $crate::backend::llvm::syntax::llvm_scalar!($then_target),
            $crate::backend::llvm::syntax::llvm_scalar!($else_target),
        )
    };
    (return;) => {
        Some($crate::backend::llvm::syntax::Terminator::return_void())
    };
    (return typed($ty:tt, $value:tt $(,)?);) => {
        $crate::backend::llvm::syntax::Terminator::return_value(
            $crate::backend::llvm::syntax::llvm_instruction_type!($ty),
            $crate::backend::llvm::syntax::llvm_scalar!($value),
        )
    };
    (switch typed($ty:tt, $value:tt $(,)?) {
        cases: [$($case:tt)*],
        default: $default:tt $(,)?
    };) => {
        $crate::backend::llvm::syntax::Terminator::switch(
            $crate::backend::llvm::syntax::llvm_instruction_type!($ty),
            $crate::backend::llvm::syntax::llvm_scalar!($value),
            $crate::backend::llvm::syntax::llvm_scalar!($default),
            $crate::backend::llvm::syntax::llvm_switch_cases!($($case)*),
        )
    };
    (unreachable;) => {
        Some($crate::backend::llvm::syntax::Terminator::unreachable())
    };
}

pub(in crate::backend) use {
    llvm_switch_cases, llvm_switch_cases_item, llvm_switch_cases_items, llvm_terminator,
};
