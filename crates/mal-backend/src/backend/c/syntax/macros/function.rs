macro_rules! c_function {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
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

pub(in crate::backend) use c_function;
