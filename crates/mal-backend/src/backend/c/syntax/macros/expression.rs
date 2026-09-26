macro_rules! c_initializer {
    (rust $initializer:expr) => {
        $initializer
    };
    (positional $value:tt) => {
        $crate::backend::c::syntax::Initializer::positional(
            $crate::backend::c::syntax::c_expr! $value,
        )
    };
    (field $name:expr; $value:tt) => {
        $crate::backend::c::syntax::Initializer::designated(
            $name,
            $crate::backend::c::syntax::c_expr! $value,
        )
    };
    (path $path:expr; $value:tt) => {
        $crate::backend::c::syntax::Initializer::designated_path(
            $path,
            $crate::backend::c::syntax::c_expr! $value,
        )
    };
}

macro_rules! c_initializers {
    ($($initializer:tt),* $(,)?) => {{
        let mut initializers = Vec::new();
        $(
            $crate::backend::c::syntax::c_initializers_item!(initializers; $initializer);
        )*
        initializers
    }};
}

macro_rules! c_initializers_item {
    ($initializers:ident; (extend $more:expr)) => {
        $initializers.extend($more)
    };
    ($initializers:ident; $initializer:tt) => {
        $initializers.extend([$crate::backend::c::syntax::c_initializer! $initializer])
    };
}

macro_rules! c_exprs {
    ($($expression:tt),* $(,)?) => {{
        let mut expressions = Vec::new();
        $(
            $crate::backend::c::syntax::c_exprs_item!(expressions; $expression);
        )*
        expressions
    }};
}

macro_rules! c_exprs_item {
    ($expressions:ident; (extend $more:expr)) => {
        $expressions.extend($more)
    };
    ($expressions:ident; $expression:tt) => {
        $expressions.extend([$crate::backend::c::syntax::c_expr! $expression])
    };
}

macro_rules! c_expr {
    (rust $expression:expr) => {
        $expression
    };
    (id $name:expr) => {
        $crate::backend::c::syntax::Expr::identifier($name)
    };
    (number $value:expr) => {
        $crate::backend::c::syntax::Expr::number($value.to_string())
    };
    (string $value:expr) => {
        $crate::backend::c::syntax::Expr::string($value)
    };
    (address $value:tt) => {
        $crate::backend::c::syntax::Expr::address_of($crate::backend::c::syntax::c_expr! $value)
    };
    (dereference $value:tt) => {
        $crate::backend::c::syntax::Expr::dereference($crate::backend::c::syntax::c_expr! $value)
    };
    (field $value:tt; $name:expr) => {
        ($crate::backend::c::syntax::c_expr! $value).field($name)
    };
    (pointer_field $value:tt; $name:expr) => {
        ($crate::backend::c::syntax::c_expr! $value).pointer_field($name)
    };
    (sizeof $value:tt) => {
        $crate::backend::c::syntax::Expr::sizeof_value($crate::backend::c::syntax::c_expr! $value)
    };
    (cast $ty:expr; $value:tt) => {
        $crate::backend::c::syntax::Expr::cast($ty, $crate::backend::c::syntax::c_expr! $value)
    };
    (call $name:expr; $($argument:tt),* $(,)?) => {
        $crate::backend::c::syntax::Expr::named_call(
            $name,
            $crate::backend::c::syntax::c_exprs!($($argument),*),
        )
    };
    (invoke $callee:tt; $($argument:tt),* $(,)?) => {
        $crate::backend::c::syntax::Expr::call(
            $crate::backend::c::syntax::c_expr! $callee,
            $crate::backend::c::syntax::c_exprs!($($argument),*),
        )
    };
    (add $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::add(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (subtract $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::subtract(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (multiply $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::multiply(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (assign $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::assign(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (equal $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::equal(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (not_equal $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::not_equal(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (greater $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::greater(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (logical_and $left:tt; $right:tt) => {
        $crate::backend::c::syntax::Expr::logical_and(
            $crate::backend::c::syntax::c_expr! $left,
            $crate::backend::c::syntax::c_expr! $right,
        )
    };
    (conditional $condition:tt; $then:tt; $otherwise:tt) => {
        $crate::backend::c::syntax::Expr::conditional(
            $crate::backend::c::syntax::c_expr! $condition,
            $crate::backend::c::syntax::c_expr! $then,
            $crate::backend::c::syntax::c_expr! $otherwise,
        )
    };
    (initializer $($element:tt),* $(,)?) => {
        $crate::backend::c::syntax::Expr::initializer_list(
            $crate::backend::c::syntax::c_exprs!($($element),*)
        )
    };
    (compound $ty:expr; $($initializer:tt),* $(,)?) => {
        $crate::backend::c::syntax::Expr::compound_literal(
            $ty,
            $crate::backend::c::syntax::c_initializers!($($initializer),*),
        )
    };
}

pub(in crate::backend) use {
    c_expr, c_exprs, c_exprs_item, c_initializer, c_initializers, c_initializers_item,
};
