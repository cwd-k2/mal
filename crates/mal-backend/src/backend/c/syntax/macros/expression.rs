macro_rules! c_expr_child {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (($($syntax:tt)*)) => {
        $crate::backend::c::syntax::c_expr!($($syntax)*)
    };
}

macro_rules! c_type_child {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (($($syntax:tt)*)) => {
        $crate::backend::c::syntax::c_type!($($syntax)*)
    };
}

macro_rules! c_initializer {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (positional($value:tt)) => {
        $crate::backend::c::syntax::Initializer::positional(
            $crate::backend::c::syntax::c_expr_child!($value),
        )
    };
    (field($name:tt, $value:tt)) => {
        $crate::backend::c::syntax::Initializer::designated(
            $crate::backend::c::syntax::c_scalar!($name),
            $crate::backend::c::syntax::c_expr_child!($value),
        )
    };
    (path($path:tt, $value:tt)) => {
        $crate::backend::c::syntax::Initializer::designated_path(
            $crate::backend::c::syntax::c_scalar!($path),
            $crate::backend::c::syntax::c_expr_child!($value),
        )
    };
}

macro_rules! c_initializers {
    ([$($initializer:tt)*]) => {{
        #[allow(unused_mut)]
        let mut initializers = Vec::from([]);
        $crate::backend::c::syntax::c_initializers_items!(initializers; $($initializer)*);
        initializers
    }};
}

macro_rules! c_initializers_items {
    ($initializers:ident;) => {};
    ($initializers:ident; ...{{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $initializers.extend({ $($rust)* });
        $crate::backend::c::syntax::c_initializers_items!($initializers; $($($rest)*)?);
    };
    ($initializers:ident; {{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $initializers.push({ $($rust)* });
        $crate::backend::c::syntax::c_initializers_items!($initializers; $($($rest)*)?);
    };
    ($initializers:ident; $kind:ident($($argument:tt)*) $(, $($rest:tt)*)?) => {
        $initializers.push($crate::backend::c::syntax::c_initializer!($kind($($argument)*)));
        $crate::backend::c::syntax::c_initializers_items!($initializers; $($($rest)*)?);
    };
}

macro_rules! c_exprs {
    ([$($expression:tt)*]) => {{
        #[allow(unused_mut)]
        let mut expressions = Vec::from([]);
        $crate::backend::c::syntax::c_exprs_items!(expressions; $($expression)*);
        expressions
    }};
}

macro_rules! c_exprs_items {
    ($expressions:ident;) => {};
    ($expressions:ident; ...{{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $expressions.extend({ $($rust)* });
        $crate::backend::c::syntax::c_exprs_items!($expressions; $($($rest)*)?);
    };
    ($expressions:ident; {{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $expressions.push({ $($rust)* });
        $crate::backend::c::syntax::c_exprs_items!($expressions; $($($rest)*)?);
    };
    ($expressions:ident; $kind:ident($($argument:tt)*) $(, $($rest:tt)*)?) => {
        $expressions.push($crate::backend::c::syntax::c_expr!($kind($($argument)*)));
        $crate::backend::c::syntax::c_exprs_items!($expressions; $($($rest)*)?);
    };
}

macro_rules! c_expr {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (id($name:tt)) => {
        $crate::backend::c::syntax::Expr::identifier($crate::backend::c::syntax::c_scalar!($name))
    };
    (number($value:tt)) => {
        $crate::backend::c::syntax::Expr::number(
            $crate::backend::c::syntax::c_scalar!($value).to_string(),
        )
    };
    (string($value:tt)) => {
        $crate::backend::c::syntax::Expr::string($crate::backend::c::syntax::c_scalar!($value))
    };
    (address($value:tt)) => {
        $crate::backend::c::syntax::Expr::address_of(
            $crate::backend::c::syntax::c_expr_child!($value),
        )
    };
    (dereference($value:tt)) => {
        $crate::backend::c::syntax::Expr::dereference(
            $crate::backend::c::syntax::c_expr_child!($value),
        )
    };
    (field($value:tt, $name:tt)) => {
        ($crate::backend::c::syntax::c_expr_child!($value))
            .field($crate::backend::c::syntax::c_scalar!($name))
    };
    (pointer_field($value:tt, $name:tt)) => {
        ($crate::backend::c::syntax::c_expr_child!($value))
            .pointer_field($crate::backend::c::syntax::c_scalar!($name))
    };
    (sizeof($value:tt)) => {
        $crate::backend::c::syntax::Expr::sizeof_value(
            $crate::backend::c::syntax::c_expr_child!($value),
        )
    };
    (cast($ty:tt, $value:tt)) => {
        $crate::backend::c::syntax::Expr::cast(
            $crate::backend::c::syntax::c_type_child!($ty),
            $crate::backend::c::syntax::c_expr_child!($value),
        )
    };
    (call($name:tt, [$($argument:tt)*] $(,)?)) => {
        $crate::backend::c::syntax::Expr::named_call(
            $crate::backend::c::syntax::c_scalar!($name),
            $crate::backend::c::syntax::c_exprs!([$($argument)*]),
        )
    };
    (invoke($callee:tt, [$($argument:tt)*] $(,)?)) => {
        $crate::backend::c::syntax::Expr::call(
            $crate::backend::c::syntax::c_expr_child!($callee),
            $crate::backend::c::syntax::c_exprs!([$($argument)*]),
        )
    };
    (add($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::add(
            $crate::backend::c::syntax::c_expr_child!($left),
            $crate::backend::c::syntax::c_expr_child!($right),
        )
    };
    (subtract($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::subtract(
            $crate::backend::c::syntax::c_expr_child!($left),
            $crate::backend::c::syntax::c_expr_child!($right),
        )
    };
    (multiply($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::multiply(
            $crate::backend::c::syntax::c_expr_child!($left),
            $crate::backend::c::syntax::c_expr_child!($right),
        )
    };
    (assign($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::assign(
            $crate::backend::c::syntax::c_expr_child!($left),
            $crate::backend::c::syntax::c_expr_child!($right),
        )
    };
    (equal($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::equal(
            $crate::backend::c::syntax::c_expr_child!($left),
            $crate::backend::c::syntax::c_expr_child!($right),
        )
    };
    (not_equal($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::not_equal(
            $crate::backend::c::syntax::c_expr_child!($left),
            $crate::backend::c::syntax::c_expr_child!($right),
        )
    };
    (greater($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::greater(
            $crate::backend::c::syntax::c_expr_child!($left),
            $crate::backend::c::syntax::c_expr_child!($right),
        )
    };
    (logical_and($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::logical_and(
            $crate::backend::c::syntax::c_expr_child!($left),
            $crate::backend::c::syntax::c_expr_child!($right),
        )
    };
    (conditional($condition:tt, $then:tt, $otherwise:tt $(,)?)) => {
        $crate::backend::c::syntax::Expr::conditional(
            $crate::backend::c::syntax::c_expr_child!($condition),
            $crate::backend::c::syntax::c_expr_child!($then),
            $crate::backend::c::syntax::c_expr_child!($otherwise),
        )
    };
    (initializer([$($element:tt)*] $(,)?)) => {
        $crate::backend::c::syntax::Expr::initializer_list(
            $crate::backend::c::syntax::c_exprs!([$($element)*]),
        )
    };
    (compound($ty:tt, [$($initializer:tt)*] $(,)?)) => {
        $crate::backend::c::syntax::Expr::compound_literal(
            $crate::backend::c::syntax::c_type_child!($ty),
            $crate::backend::c::syntax::c_initializers!([$($initializer)*]),
        )
    };
}

pub(in crate::backend) use {
    c_expr, c_expr_child, c_exprs, c_exprs_items, c_initializer, c_initializers,
    c_initializers_items, c_type_child,
};
