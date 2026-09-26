macro_rules! llvm_switch_cases_item {
    ($cases:ident; {{ $($rust:tt)* }}) => {
        $cases.extend({ $($rust)* })
    };
    ($cases:ident; { $($rust:tt)* }) => {
        $cases.push({ $($rust)* })
    };
    ($cases:ident; (case $value:tt => $target:tt)) => {
        $cases.push((
            $crate::backend::llvm::syntax::llvm_scalar!($value).to_string(),
            $crate::backend::llvm::syntax::llvm_scalar!($target).to_string(),
        ))
    };
}

macro_rules! llvm_switch_cases {
    ($($case:tt),* $(,)?) => {{
        let mut cases = Vec::new();
        $(
            $crate::backend::llvm::syntax::llvm_switch_cases_item!(cases; $case);
        )*
        cases
    }};
}

macro_rules! llvm_terminator {
    ({ $($rust:tt)* }) => {
        Some({ $($rust)* })
    };
    (branch $target:tt) => {
        $crate::backend::llvm::syntax::Terminator::branch(
            $crate::backend::llvm::syntax::llvm_scalar!($target),
        )
    };
    (conditional $condition:tt => $then_target:tt, $else_target:tt) => {
        $crate::backend::llvm::syntax::Terminator::conditional_branch(
            $crate::backend::llvm::syntax::llvm_scalar!($condition),
            $crate::backend::llvm::syntax::llvm_scalar!($then_target),
            $crate::backend::llvm::syntax::llvm_scalar!($else_target),
        )
    };
    (return_void) => {
        Some($crate::backend::llvm::syntax::Terminator::return_void())
    };
    (return $ty:tt => $value:tt) => {
        $crate::backend::llvm::syntax::Terminator::return_value(
            $crate::backend::llvm::syntax::llvm_instruction_type!($ty),
            $crate::backend::llvm::syntax::llvm_scalar!($value),
        )
    };
    (switch $ty:tt => $value:tt; default $default:tt; [$($case:tt),* $(,)?]) => {
        $crate::backend::llvm::syntax::Terminator::switch(
            $crate::backend::llvm::syntax::llvm_instruction_type!($ty),
            $crate::backend::llvm::syntax::llvm_scalar!($value),
            $crate::backend::llvm::syntax::llvm_scalar!($default),
            $crate::backend::llvm::syntax::llvm_switch_cases!($($case),*),
        )
    };
    (unreachable) => {
        Some($crate::backend::llvm::syntax::Terminator::unreachable())
    };
}

pub(in crate::backend) use {llvm_switch_cases, llvm_switch_cases_item, llvm_terminator};
