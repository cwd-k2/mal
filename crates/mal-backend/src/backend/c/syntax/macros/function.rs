macro_rules! c_function_build {
    ([$($signature:tt)*]; body { $($body:tt)* }) => {
        $crate::backend::c::syntax::FunctionDefinition::from_signature(
            $crate::backend::c::syntax::c_signature!($($signature)*),
            { $($body)* },
        )
    };
    ([$($signature:tt)*]; block $body:tt) => {
        $crate::backend::c::syntax::FunctionDefinition::from_signature(
            $crate::backend::c::syntax::c_signature!($($signature)*),
            $crate::backend::c::syntax::c_block! $body,
        )
    };
}

macro_rules! c_function {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*) $body_kind:ident $body:tt) => {
        $crate::backend::c::syntax::c_function_build!(
            [fn $name($($parameter)*) -> $kind($($result)*)]; $body_kind $body
        )
    };
    (fn $name:tt($($parameter:tt)*) -> { $($result:tt)* } $body_kind:ident $body:tt) => {
        $crate::backend::c::syntax::c_function_build!(
            [fn $name($($parameter)*) -> { $($result)* }]; $body_kind $body
        )
    };
    (static fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*) $body_kind:ident $body:tt) => {
        $crate::backend::c::syntax::c_function_build!(
            [static fn $name($($parameter)*) -> $kind($($result)*)]; $body_kind $body
        )
    };
    (static fn $name:tt($($parameter:tt)*) -> { $($result:tt)* } $body_kind:ident $body:tt) => {
        $crate::backend::c::syntax::c_function_build!(
            [static fn $name($($parameter)*) -> { $($result)* }]; $body_kind $body
        )
    };
    (static inline fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*) $body_kind:ident $body:tt) => {
        $crate::backend::c::syntax::c_function_build!(
            [static inline fn $name($($parameter)*) -> $kind($($result)*)]; $body_kind $body
        )
    };
    (static inline fn $name:tt($($parameter:tt)*) -> { $($result:tt)* } $body_kind:ident $body:tt) => {
        $crate::backend::c::syntax::c_function_build!(
            [static inline fn $name($($parameter)*) -> { $($result)* }]; $body_kind $body
        )
    };
    (static inline noreturn fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*) $body_kind:ident $body:tt) => {
        $crate::backend::c::syntax::c_function_build!(
            [static inline noreturn fn $name($($parameter)*) -> $kind($($result)*)]; $body_kind $body
        )
    };
    (static inline noreturn fn $name:tt($($parameter:tt)*) -> { $($result:tt)* } $body_kind:ident $body:tt) => {
        $crate::backend::c::syntax::c_function_build!(
            [static inline noreturn fn $name($($parameter)*) -> { $($result)* }]; $body_kind $body
        )
    };
    (signature { $($signature:tt)* }; body { $($body:tt)* }) => {
        $crate::backend::c::syntax::FunctionDefinition::from_signature(
            { $($signature)* },
            { $($body)* },
        )
    };
    (signature { $($signature:tt)* }; block $body:tt) => {
        $crate::backend::c::syntax::FunctionDefinition::from_signature(
            { $($signature)* },
            $crate::backend::c::syntax::c_block! $body,
        )
    };
    (macro { $($invocation:tt)* }; body { $($body:tt)* }) => {
        $crate::backend::c::syntax::FunctionDefinition::from_macro(
            { $($invocation)* },
            { $($body)* },
        )
    };
    (macro { $($invocation:tt)* }; block $body:tt) => {
        $crate::backend::c::syntax::FunctionDefinition::from_macro(
            { $($invocation)* },
            $crate::backend::c::syntax::c_block! $body,
        )
    };
}

pub(in crate::backend) use {c_function, c_function_build};
