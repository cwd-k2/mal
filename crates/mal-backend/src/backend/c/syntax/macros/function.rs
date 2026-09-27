macro_rules! c_function_from_syntax_normalized {
    ([$($signature:tt)*] { $($body:tt)* }) => {
        $crate::backend::c::syntax::FunctionDefinition::from_signature(
            $crate::backend::c::syntax::c_signature_normalized!($($signature)*),
            $crate::backend::c::syntax::c_block_normalized!( $($body)* ),
        )
    };
}

macro_rules! c_function_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*) (@rust $($body:tt)*)) => {
        $crate::backend::c::syntax::FunctionDefinition::from_signature(
            $crate::backend::c::syntax::c_signature_normalized! {
                fn $name($($parameter)*) -> $kind($($result)*)
            },
            { $($body)* },
        )
    };
    (fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*) { $($body:tt)* }) => {
        $crate::backend::c::syntax::c_function_from_syntax_normalized! {
            [fn $name($($parameter)*) -> $kind($($result)*)] { $($body)* }
        }
    };
    (fn $name:tt($($parameter:tt)*) -> (@rust $($result:tt)*) { $($body:tt)* }) => {
        $crate::backend::c::syntax::c_function_from_syntax_normalized! {
            [fn $name($($parameter)*) -> (@rust $($result)*)] { $($body)* }
        }
    };
    (#[static] fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*) { $($body:tt)* }) => {
        $crate::backend::c::syntax::c_function_from_syntax_normalized! {
            [#[static] fn $name($($parameter)*) -> $kind($($result)*)] { $($body)* }
        }
    };
    (#[static] fn $name:tt($($parameter:tt)*) -> (@rust $($result:tt)*) { $($body:tt)* }) => {
        $crate::backend::c::syntax::c_function_from_syntax_normalized! {
            [#[static] fn $name($($parameter)*) -> (@rust $($result)*)] { $($body)* }
        }
    };
    (#[static] #[inline] fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*) { $($body:tt)* }) => {
        $crate::backend::c::syntax::c_function_from_syntax_normalized! {
            [#[static] #[inline] fn $name($($parameter)*) -> $kind($($result)*)] { $($body)* }
        }
    };
    (#[static] #[inline] fn $name:tt($($parameter:tt)*) -> (@rust $($result:tt)*) { $($body:tt)* }) => {
        $crate::backend::c::syntax::c_function_from_syntax_normalized! {
            [#[static] #[inline] fn $name($($parameter)*) -> (@rust $($result)*)] { $($body)* }
        }
    };
    (#[static] #[inline] #[noreturn] fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*) { $($body:tt)* }) => {
        $crate::backend::c::syntax::c_function_from_syntax_normalized! {
            [#[static] #[inline] #[noreturn] fn $name($($parameter)*) -> $kind($($result)*)]
            { $($body)* }
        }
    };
    (#[static] #[inline] #[noreturn] fn $name:tt($($parameter:tt)*) -> (@rust $($result:tt)*) { $($body:tt)* }) => {
        $crate::backend::c::syntax::c_function_from_syntax_normalized! {
            [#[static] #[inline] #[noreturn] fn $name($($parameter)*) -> (@rust $($result)*)]
            { $($body)* }
        }
    };
    (signature (@rust $($signature:tt)*) { $($body:tt)* }) => {
        $crate::backend::c::syntax::FunctionDefinition::from_signature(
            { $($signature)* },
            $crate::backend::c::syntax::c_block_normalized!( $($body)* ),
        )
    };
    (signature (@rust $($signature:tt)*) body (@rust $($body:tt)*)) => {
        $crate::backend::c::syntax::FunctionDefinition::from_signature(
            { $($signature)* },
            { $($body)* },
        )
    };
    (macro (@rust $($invocation:tt)*) { $($body:tt)* }) => {
        $crate::backend::c::syntax::FunctionDefinition::from_macro(
            { $($invocation)* },
            $crate::backend::c::syntax::c_block_normalized!( $($body)* ),
        )
    };
    (macro (@rust $($invocation:tt)*) body (@rust $($body:tt)*)) => {
        $crate::backend::c::syntax::FunctionDefinition::from_macro(
            { $($invocation)* },
            { $($body)* },
        )
    };
}

macro_rules! c_function {
    (@normalized $($syntax:tt)*) => {
        $crate::backend::c::syntax::c_function_normalized!($($syntax)*)
    };
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_function]; $($syntax)*
        )
    };
}

pub(in crate::backend) use {c_function_from_syntax_normalized, c_function_normalized};

pub(in crate::backend) use c_function;
