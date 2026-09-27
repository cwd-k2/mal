macro_rules! c_preprocessor_expr {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (defined($name:tt)) => {
        $crate::backend::c::syntax::PreprocessorExpr::defined(
            $crate::backend::c::syntax::c_scalar!($name),
        )
    };
}

macro_rules! c_directive {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (include(system $path:tt)) => {
        $crate::backend::c::syntax::Directive::include_system(
            $crate::backend::c::syntax::c_scalar!($path),
        )
    };
    (include(quoted $path:tt)) => {
        $crate::backend::c::syntax::Directive::include_quoted(
            $crate::backend::c::syntax::c_scalar!($path),
        )
    };
    (define $name:tt = unused;) => {
        $crate::backend::c::syntax::Directive::define_attribute(
            $crate::backend::c::syntax::c_scalar!($name),
            $crate::backend::c::syntax::Attribute::Unused,
        )
    };
    (define $name:tt = $value:tt;) => {
        $crate::backend::c::syntax::Directive::define_expr(
            $crate::backend::c::syntax::c_scalar!($name),
            $crate::backend::c::syntax::c_expr_child!($value),
        )
    };
    (define $name:tt;) => {
        $crate::backend::c::syntax::Directive::define_empty(
            $crate::backend::c::syntax::c_scalar!($name),
        )
    };
    (if ($kind:ident($($condition:tt)*))) => {
        $crate::backend::c::syntax::Directive::If(
            $crate::backend::c::syntax::c_preprocessor_expr!($kind($($condition)*)),
        )
    };
    (ifndef $name:tt) => {
        $crate::backend::c::syntax::Directive::Ifndef(
            $crate::backend::c::syntax::c_scalar!($name).into(),
        )
    };
    (else) => {
        $crate::backend::c::syntax::Directive::Else
    };
    (endif) => {
        $crate::backend::c::syntax::Directive::Endif
    };
    (define_items $name:tt {
        parameters: {{ $($parameters:tt)* }},
        declarations: {{ $($declarations:tt)* }},
        definitions: {{ $($definitions:tt)* }},
        trailing: {{ $($trailing:tt)* }},
    }) => {
        $crate::backend::c::syntax::Directive::function_items_define(
            $crate::backend::c::syntax::c_scalar!($name),
            { $($parameters)* },
            { $($declarations)* },
            { $($definitions)* },
            { $($trailing)* },
        )
    };
}

macro_rules! c_macro_invocation {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    ($name:tt([$($argument:tt)*])) => {
        $crate::backend::c::syntax::MacroInvocation::new(
            $crate::backend::c::syntax::c_scalar!($name),
            $crate::backend::c::syntax::c_exprs!([$($argument)*]),
        )
    };
}

pub(in crate::backend) use {c_directive, c_macro_invocation, c_preprocessor_expr};
