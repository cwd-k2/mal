macro_rules! llvm_switch_cases_item {
    ($cases:ident; (extend $more:expr)) => {
        $cases.extend($more)
    };
    ($cases:ident; (rust $case:expr)) => {
        $cases.push($case)
    };
    ($cases:ident; (case $value:expr => $target:expr)) => {
        $cases.push(($value.to_string(), $target.to_string()))
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
    (rust $terminator:expr) => {
        Some($terminator)
    };
    (branch $target:expr) => {
        $crate::backend::llvm::syntax::Terminator::branch($target)
    };
    (conditional $condition:expr => $then_target:expr, $else_target:expr) => {
        $crate::backend::llvm::syntax::Terminator::conditional_branch(
            $condition,
            $then_target,
            $else_target,
        )
    };
    (return_void) => {
        Some($crate::backend::llvm::syntax::Terminator::return_void())
    };
    (return $ty:expr => $value:expr) => {
        $crate::backend::llvm::syntax::Terminator::return_value($ty, $value)
    };
    (switch $ty:expr => $value:expr; default $default:expr; [$($case:tt),* $(,)?]) => {
        $crate::backend::llvm::syntax::Terminator::switch(
            $ty,
            $value,
            $default,
            $crate::backend::llvm::syntax::llvm_switch_cases!($($case),*),
        )
    };
    (unreachable) => {
        Some($crate::backend::llvm::syntax::Terminator::unreachable())
    };
}

pub(in crate::backend::llvm) use {llvm_switch_cases, llvm_switch_cases_item, llvm_terminator};
