macro_rules! c_preprocessor_expr_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (defined($name:tt)) => {
        $crate::backend::c::syntax::PreprocessorExpr::defined(
            $crate::backend::c::syntax::c_scalar_normalized!($name),
        )
    };
}

macro_rules! c_directive_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (include(system $path:tt)) => {
        $crate::backend::c::syntax::Directive::include_system(
            $crate::backend::c::syntax::c_scalar_normalized!($path),
        )
    };
    (include(quoted $path:tt)) => {
        $crate::backend::c::syntax::Directive::include_quoted(
            $crate::backend::c::syntax::c_scalar_normalized!($path),
        )
    };
    (define $name:tt = unused;) => {
        $crate::backend::c::syntax::Directive::define_attribute(
            $crate::backend::c::syntax::c_scalar_normalized!($name),
            $crate::backend::c::syntax::Attribute::Unused,
        )
    };
    (define $name:tt = $value:tt;) => {
        $crate::backend::c::syntax::Directive::define_expr(
            $crate::backend::c::syntax::c_scalar_normalized!($name),
            $crate::backend::c::syntax::c_expr_child_normalized!($value),
        )
    };
    (define $name:tt;) => {
        $crate::backend::c::syntax::Directive::define_empty(
            $crate::backend::c::syntax::c_scalar_normalized!($name),
        )
    };
    (if ($kind:ident($($condition:tt)*))) => {
        $crate::backend::c::syntax::Directive::If(
            $crate::backend::c::syntax::c_preprocessor_expr_normalized!($kind($($condition)*)),
        )
    };
    (ifndef $name:tt) => {
        $crate::backend::c::syntax::Directive::Ifndef(
            $crate::backend::c::syntax::c_scalar_normalized!($name).into(),
        )
    };
    (else) => {
        $crate::backend::c::syntax::Directive::Else
    };
    (endif) => {
        $crate::backend::c::syntax::Directive::Endif
    };
    (define_items $name:tt {
        parameters: (@rust $($parameters:tt)*),
        declarations: (@rust $($declarations:tt)*),
        definitions: (@rust $($definitions:tt)*),
        trailing: (@rust $($trailing:tt)*),
    }) => {
        $crate::backend::c::syntax::Directive::function_items_define(
            $crate::backend::c::syntax::c_scalar_normalized!($name),
            { $($parameters)* },
            { $($declarations)* },
            { $($definitions)* },
            { $($trailing)* },
        )
    };
    (define_functions $name:tt {
        parameters: (@rust $($parameters:tt)*),
        definitions: (@rust $($definitions:tt)*),
    }) => {
        $crate::backend::c::syntax::Directive::function_definitions_define(
            $crate::backend::c::syntax::c_scalar_normalized!($name),
            { $($parameters)* },
            { $($definitions)* },
        )
    };
}

macro_rules! c_macro_invocation_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    ($name:tt([$($argument:tt)*])) => {
        $crate::backend::c::syntax::MacroInvocation::new(
            $crate::backend::c::syntax::c_scalar_normalized!($name),
            $crate::backend::c::syntax::c_exprs_normalized!([$($argument)*]),
        )
    };
}

macro_rules! c_directive {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_directive_normalized]; $($syntax)*
        )
    };
}

macro_rules! c_macro_invocation {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_macro_invocation_normalized]; $($syntax)*
        )
    };
}

pub(in crate::backend) use {
    c_directive_normalized, c_macro_invocation_normalized, c_preprocessor_expr_normalized,
};

pub(in crate::backend) use {c_directive, c_macro_invocation};
