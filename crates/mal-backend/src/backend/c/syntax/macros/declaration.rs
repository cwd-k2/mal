//! C syntax macros for scalars, comments, types, variables, and declarations.

macro_rules! c_scalar_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    ($literal:literal) => { $literal };
}

macro_rules! c_comment_normalized {
    ($text:tt) => {
        $crate::backend::c::syntax::Comment::new($crate::backend::c::syntax::c_scalar_normalized!(
            $text
        ))
    };
}

macro_rules! c_type_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (named($name:tt)) => {
        $crate::backend::c::syntax::TypeName::named(
            $crate::backend::c::syntax::c_scalar_normalized!($name),
        )
    };
    (struct($name:tt)) => {
        $crate::backend::c::syntax::TypeName::structure(
            $crate::backend::c::syntax::c_scalar_normalized!($name),
        )
    };
    (const(named($name:tt))) => {
        $crate::backend::c::syntax::TypeName::const_named(
            $crate::backend::c::syntax::c_scalar_normalized!($name),
        )
    };
    (ptr($($inner:tt)+)) => {
        ($crate::backend::c::syntax::c_type_normalized!($($inner)+)).pointer()
    };
}

macro_rules! c_variable_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    ($name:tt : $kind:ident($($ty:tt)*)) => {
        $crate::backend::c::syntax::VariableDeclaration::new(
            $crate::backend::c::syntax::c_type_normalized!($kind($($ty)*)),
            $crate::backend::c::syntax::c_scalar_normalized!($name),
        )
    };
    ($name:tt : (@rust $($ty:tt)*)) => {
        $crate::backend::c::syntax::VariableDeclaration::new(
            { $($ty)* },
            $crate::backend::c::syntax::c_scalar_normalized!($name),
        )
    };
    (array $name:tt : $kind:ident($($ty:tt)*); size $size:tt) => {
        $crate::backend::c::syntax::VariableDeclaration::array(
            $crate::backend::c::syntax::c_type_normalized!($kind($($ty)*)),
            $crate::backend::c::syntax::c_scalar_normalized!($name),
            $crate::backend::c::syntax::c_expr_child_normalized!($size),
        )
    };
    (array $name:tt : (@rust $($ty:tt)*); size $size:tt) => {
        $crate::backend::c::syntax::VariableDeclaration::array(
            { $($ty)* },
            $crate::backend::c::syntax::c_scalar_normalized!($name),
            $crate::backend::c::syntax::c_expr_child_normalized!($size),
        )
    };
}

macro_rules! c_declaration_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (fn (@rust $($signature:tt)*);) => {
        $crate::backend::c::syntax::Declaration::function({ $($signature)* })
    };
    (type $alias:tt = $kind:ident($($source:tt)*)) => {
        $crate::backend::c::syntax::Declaration::type_alias(
            $crate::backend::c::syntax::c_type_normalized!($kind($($source)*)),
            $crate::backend::c::syntax::c_scalar_normalized!($alias),
        )
    };
    (type $alias:tt = (@rust $($source:tt)*)) => {
        $crate::backend::c::syntax::Declaration::type_alias(
            { $($source)* },
            $crate::backend::c::syntax::c_scalar_normalized!($alias),
        )
    };
    (static_assert($condition:tt, $message:tt);) => {
        $crate::backend::c::syntax::Declaration::static_assert(
            $crate::backend::c::syntax::c_expr_child_normalized!($condition),
            $crate::backend::c::syntax::c_scalar_normalized!($message),
        )
    };
}

macro_rules! c_comment {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_comment_normalized]; $($syntax)*
        )
    };
}

macro_rules! c_type {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_type_normalized]; $($syntax)*
        )
    };
}

macro_rules! c_variable {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_variable_normalized]; $($syntax)*
        )
    };
}

macro_rules! c_declaration {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_declaration_normalized]; $($syntax)*
        )
    };
}

pub(in crate::backend) use {
    c_comment, c_comment_normalized, c_declaration, c_declaration_normalized, c_scalar_normalized,
    c_type, c_type_normalized, c_variable, c_variable_normalized,
};
