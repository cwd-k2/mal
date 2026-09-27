macro_rules! c_expr_child_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (($($syntax:tt)*)) => {
        $crate::backend::c::syntax::c_expr_normalized!($($syntax)*)
    };
}

macro_rules! c_type_child_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (($($syntax:tt)*)) => {
        $crate::backend::c::syntax::c_type_normalized!($($syntax)*)
    };
}

macro_rules! c_initializer_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (positional($value:tt)) => {
        $crate::backend::c::syntax::Initializer::positional(
            $crate::backend::c::syntax::c_expr_child_normalized!($value),
        )
    };
    (field($name:tt, $value:tt)) => {
        $crate::backend::c::syntax::Initializer::designated(
            $crate::backend::c::syntax::c_scalar_normalized!($name),
            $crate::backend::c::syntax::c_expr_child_normalized!($value),
        )
    };
    (path($path:tt, $value:tt)) => {
        $crate::backend::c::syntax::Initializer::designated_path(
            $crate::backend::c::syntax::c_scalar_normalized!($path),
            $crate::backend::c::syntax::c_expr_child_normalized!($value),
        )
    };
}

macro_rules! c_initializers_normalized {
    ([$($initializer:tt)*]) => {{
        #[allow(unused_mut)]
        let mut initializers = Vec::from([]);
        $crate::backend::c::syntax::c_initializers_items_normalized!(initializers; $($initializer)*);
        initializers
    }};
}

macro_rules! c_initializers_items_normalized {
    ($initializers:ident;) => {};
    ($initializers:ident; ...(@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $initializers.extend({ $($rust)* });
        $crate::backend::c::syntax::c_initializers_items_normalized!($initializers; $($($rest)*)?);
    };
    ($initializers:ident; (@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $initializers.push({ $($rust)* });
        $crate::backend::c::syntax::c_initializers_items_normalized!($initializers; $($($rest)*)?);
    };
    ($initializers:ident; $kind:ident($($argument:tt)*) $(, $($rest:tt)*)?) => {
        $initializers.push($crate::backend::c::syntax::c_initializer_normalized!($kind($($argument)*)));
        $crate::backend::c::syntax::c_initializers_items_normalized!($initializers; $($($rest)*)?);
    };
}

macro_rules! c_exprs_normalized {
    ([$($expression:tt)*]) => {{
        #[allow(unused_mut)]
        let mut expressions = Vec::from([]);
        $crate::backend::c::syntax::c_exprs_items_normalized!(expressions; $($expression)*);
        expressions
    }};
}

macro_rules! c_exprs_items_normalized {
    ($expressions:ident;) => {};
    ($expressions:ident; ...(@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $expressions.extend({ $($rust)* });
        $crate::backend::c::syntax::c_exprs_items_normalized!($expressions; $($($rest)*)?);
    };
    ($expressions:ident; (@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $expressions.push({ $($rust)* });
        $crate::backend::c::syntax::c_exprs_items_normalized!($expressions; $($($rest)*)?);
    };
    ($expressions:ident; $kind:ident($($argument:tt)*) $(, $($rest:tt)*)?) => {
        $expressions.push($crate::backend::c::syntax::c_expr_normalized!($kind($($argument)*)));
        $crate::backend::c::syntax::c_exprs_items_normalized!($expressions; $($($rest)*)?);
    };
}

macro_rules! c_expr_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (id($name:tt)) => {
        $crate::backend::c::syntax::Expr::identifier($crate::backend::c::syntax::c_scalar_normalized!($name))
    };
    (number($value:tt)) => {
        $crate::backend::c::syntax::Expr::number(
            $crate::backend::c::syntax::c_scalar_normalized!($value).to_string(),
        )
    };
    (string($value:tt)) => {
        $crate::backend::c::syntax::Expr::string($crate::backend::c::syntax::c_scalar_normalized!($value))
    };
    (address($value:tt)) => {
        $crate::backend::c::syntax::Expr::address_of(
            $crate::backend::c::syntax::c_expr_child_normalized!($value),
        )
    };
    (dereference($value:tt)) => {
        $crate::backend::c::syntax::Expr::dereference(
            $crate::backend::c::syntax::c_expr_child_normalized!($value),
        )
    };
    (field($value:tt, $name:tt)) => {
        ($crate::backend::c::syntax::c_expr_child_normalized!($value))
            .field($crate::backend::c::syntax::c_scalar_normalized!($name))
    };
    (pointer_field($value:tt, $name:tt)) => {
        ($crate::backend::c::syntax::c_expr_child_normalized!($value))
            .pointer_field($crate::backend::c::syntax::c_scalar_normalized!($name))
    };
    (sizeof($value:tt)) => {
        $crate::backend::c::syntax::Expr::sizeof_value(
            $crate::backend::c::syntax::c_expr_child_normalized!($value),
        )
    };
    (cast($ty:tt, $value:tt)) => {
        $crate::backend::c::syntax::Expr::cast(
            $crate::backend::c::syntax::c_type_child_normalized!($ty),
            $crate::backend::c::syntax::c_expr_child_normalized!($value),
        )
    };
    (call($name:tt, [$($argument:tt)*] $(,)?)) => {
        $crate::backend::c::syntax::Expr::named_call(
            $crate::backend::c::syntax::c_scalar_normalized!($name),
            $crate::backend::c::syntax::c_exprs_normalized!([$($argument)*]),
        )
    };
    (invoke($callee:tt, [$($argument:tt)*] $(,)?)) => {
        $crate::backend::c::syntax::Expr::call(
            $crate::backend::c::syntax::c_expr_child_normalized!($callee),
            $crate::backend::c::syntax::c_exprs_normalized!([$($argument)*]),
        )
    };
    (add($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::add(
            $crate::backend::c::syntax::c_expr_child_normalized!($left),
            $crate::backend::c::syntax::c_expr_child_normalized!($right),
        )
    };
    (subtract($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::subtract(
            $crate::backend::c::syntax::c_expr_child_normalized!($left),
            $crate::backend::c::syntax::c_expr_child_normalized!($right),
        )
    };
    (multiply($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::multiply(
            $crate::backend::c::syntax::c_expr_child_normalized!($left),
            $crate::backend::c::syntax::c_expr_child_normalized!($right),
        )
    };
    (assign($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::assign(
            $crate::backend::c::syntax::c_expr_child_normalized!($left),
            $crate::backend::c::syntax::c_expr_child_normalized!($right),
        )
    };
    (equal($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::equal(
            $crate::backend::c::syntax::c_expr_child_normalized!($left),
            $crate::backend::c::syntax::c_expr_child_normalized!($right),
        )
    };
    (not_equal($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::not_equal(
            $crate::backend::c::syntax::c_expr_child_normalized!($left),
            $crate::backend::c::syntax::c_expr_child_normalized!($right),
        )
    };
    (greater($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::greater(
            $crate::backend::c::syntax::c_expr_child_normalized!($left),
            $crate::backend::c::syntax::c_expr_child_normalized!($right),
        )
    };
    (logical_and($left:tt, $right:tt)) => {
        $crate::backend::c::syntax::Expr::logical_and(
            $crate::backend::c::syntax::c_expr_child_normalized!($left),
            $crate::backend::c::syntax::c_expr_child_normalized!($right),
        )
    };
    (conditional($condition:tt, $then:tt, $otherwise:tt $(,)?)) => {
        $crate::backend::c::syntax::Expr::conditional(
            $crate::backend::c::syntax::c_expr_child_normalized!($condition),
            $crate::backend::c::syntax::c_expr_child_normalized!($then),
            $crate::backend::c::syntax::c_expr_child_normalized!($otherwise),
        )
    };
    (initializer([$($element:tt)*] $(,)?)) => {
        $crate::backend::c::syntax::Expr::initializer_list(
            $crate::backend::c::syntax::c_exprs_normalized!([$($element)*]),
        )
    };
    (compound($ty:tt, [$($initializer:tt)*] $(,)?)) => {
        $crate::backend::c::syntax::Expr::compound_literal(
            $crate::backend::c::syntax::c_type_child_normalized!($ty),
            $crate::backend::c::syntax::c_initializers_normalized!([$($initializer)*]),
        )
    };
}

macro_rules! c_initializer {
    (@normalized $($syntax:tt)*) => {
        $crate::backend::c::syntax::c_initializer_normalized!($($syntax)*)
    };
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_initializer]; $($syntax)*
        )
    };
}

macro_rules! c_expr {
    (@normalized $($syntax:tt)*) => {
        $crate::backend::c::syntax::c_expr_normalized!($($syntax)*)
    };
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_expr]; $($syntax)*
        )
    };
}

pub(in crate::backend) use {
    c_expr_child_normalized, c_expr_normalized, c_exprs_items_normalized, c_exprs_normalized,
    c_initializer_normalized, c_initializers_items_normalized, c_initializers_normalized,
    c_type_child_normalized,
};

pub(in crate::backend) use {c_expr, c_initializer};
