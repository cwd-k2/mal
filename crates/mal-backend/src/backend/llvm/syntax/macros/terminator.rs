macro_rules! llvm_switch_cases_item_normalized {
    ($cases:ident; ...(@rust $($rust:tt)*)) => {
        $cases.extend({ $($rust)* })
    };
    ($cases:ident; $value:tt => $target:tt) => {
        $cases.push((
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($value).to_string(),
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($target).to_string(),
        ))
    };
}
macro_rules! llvm_switch_cases_normalized {
    ($($case:tt)*) => {{
        #[allow(unused_mut)]
        let mut cases = Vec::from([]);
        $crate::backend::llvm::syntax::llvm_switch_cases_items_normalized!(cases; $($case)*);
        cases
    }};
}
macro_rules! llvm_switch_cases_items_normalized {
    ($cases:ident;) => {};
    ($cases:ident; ...(@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_switch_cases_item_normalized!($cases; ...(@rust $($rust)*));
        $crate::backend::llvm::syntax::llvm_switch_cases_items_normalized!($cases; $($($rest)*)?);
    };
    ($cases:ident; $value:tt => $target:tt $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_switch_cases_item_normalized!($cases; $value => $target);
        $crate::backend::llvm::syntax::llvm_switch_cases_items_normalized!($cases; $($($rest)*)?);
    };
}

macro_rules! llvm_terminator_normalized {
    ((@rust $($rust:tt)*)) => {
        Some({ $($rust)* })
    };
    (branch { target: $target:tt $(,)? };) => {
        $crate::backend::llvm::syntax::Terminator::branch(
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($target),
        )
    };
    (branch {
        condition: $condition:tt,
        then: $then_target:tt,
        otherwise: $else_target:tt $(,)?
    };) => {
        $crate::backend::llvm::syntax::Terminator::conditional_branch(
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($condition),
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($then_target),
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($else_target),
        )
    };
    (return;) => {
        Some($crate::backend::llvm::syntax::Terminator::return_void())
    };
    (return typed($ty:tt, $value:tt $(,)?);) => {
        $crate::backend::llvm::syntax::Terminator::return_value(
            $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($ty),
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($value),
        )
    };
    (switch typed($ty:tt, $value:tt $(,)?) {
        cases: [$($case:tt)*],
        default: $default:tt $(,)?
    };) => {
        $crate::backend::llvm::syntax::Terminator::switch(
            $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($ty),
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($value),
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($default),
            $crate::backend::llvm::syntax::llvm_switch_cases_normalized!($($case)*),
        )
    };
    (unreachable;) => {
        Some($crate::backend::llvm::syntax::Terminator::unreachable())
    };
}

macro_rules! llvm_terminator {
    (@normalized $($syntax:tt)*) => {
        $crate::backend::llvm::syntax::llvm_terminator_normalized!($($syntax)*)
    };
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::llvm::syntax::llvm_terminator]; $($syntax)*
        )
    };
}

pub(in crate::backend) use {
    llvm_switch_cases_item_normalized, llvm_switch_cases_items_normalized,
    llvm_switch_cases_normalized, llvm_terminator_normalized,
};

pub(in crate::backend) use llvm_terminator;
