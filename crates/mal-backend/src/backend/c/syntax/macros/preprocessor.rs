macro_rules! c_preprocessor_expr {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (defined($name:expr)) => {
        $crate::backend::c::syntax::PreprocessorExpr::defined($name)
    };
}

macro_rules! c_directive {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (include(system $path:expr)) => {
        $crate::backend::c::syntax::Directive::include_system($path)
    };
    (include(quoted $path:expr)) => {
        $crate::backend::c::syntax::Directive::include_quoted($path)
    };
    (define $name:tt = unused) => {
        $crate::backend::c::syntax::Directive::define_attribute(
            $name,
            $crate::backend::c::syntax::Attribute::Unused,
        )
    };
    (define $name:tt = $value:tt) => {
        $crate::backend::c::syntax::Directive::define_expr(
            $name,
            $crate::backend::c::syntax::c_expr_child!($value),
        )
    };
    (define $name:expr) => {
        $crate::backend::c::syntax::Directive::define_empty($name)
    };
    (if $condition:tt) => {
        $crate::backend::c::syntax::Directive::If(
            $crate::backend::c::syntax::c_preprocessor_expr! $condition,
        )
    };
    (ifndef $name:expr) => {
        $crate::backend::c::syntax::Directive::Ifndef($name.into())
    };
    (else) => {
        $crate::backend::c::syntax::Directive::Else
    };
    (endif) => {
        $crate::backend::c::syntax::Directive::Endif
    };
    (define_items $name:tt($parameters:expr);
        declarations { $declarations:expr };
        definitions { $definitions:expr };
        trailing { $trailing:expr }
    ) => {
        $crate::backend::c::syntax::Directive::function_items_define(
            $name,
            $parameters,
            $declarations,
            $definitions,
            $trailing,
        )
    };
}

macro_rules! c_macro_invocation {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    ($name:expr; [$($argument:tt),* $(,)?]) => {
        $crate::backend::c::syntax::MacroInvocation::new(
            $name,
            $crate::backend::c::syntax::c_exprs!($($argument),*),
        )
    };
}

pub(in crate::backend) use {c_directive, c_macro_invocation, c_preprocessor_expr};
