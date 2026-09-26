macro_rules! c_type {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (named($name:expr)) => {
        $crate::backend::c::syntax::TypeName::named($name)
    };
    (struct($name:expr)) => {
        $crate::backend::c::syntax::TypeName::structure($name)
    };
    (const(named($name:expr))) => {
        $crate::backend::c::syntax::TypeName::const_named($name)
    };
    (ptr($($inner:tt)+)) => {
        ($crate::backend::c::syntax::c_type!($($inner)+)).pointer()
    };
}

macro_rules! c_parameter_attributes {
    ($parameter:ident;) => {};
    ($parameter:ident; maybe_unused $(, $rest:ident)*) => {
        $parameter = $parameter.maybe_unused();
        $crate::backend::c::syntax::c_parameter_attributes!($parameter; $($rest),*);
    };
}

macro_rules! c_parameter {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (_ : $kind:ident($($type:tt)*)) => {
        $crate::backend::c::syntax::Parameter::unnamed(
            $crate::backend::c::syntax::c_type!($kind($($type)*)),
        )
    };
    (_ : { $($type:tt)* }) => {
        $crate::backend::c::syntax::Parameter::unnamed({ $($type)* })
    };
    ($name:tt : $kind:ident($($type:tt)*) [$($attribute:ident),* $(,)?]) => {{
        let mut parameter = $crate::backend::c::syntax::Parameter::named(
            $crate::backend::c::syntax::c_type!($kind($($type)*)),
            $name,
        );
        $crate::backend::c::syntax::c_parameter_attributes!(parameter; $($attribute),*);
        parameter
    }};
    ($name:tt : { $($type:tt)* } [$($attribute:ident),* $(,)?]) => {{
        let mut parameter = $crate::backend::c::syntax::Parameter::named({ $($type)* }, $name);
        $crate::backend::c::syntax::c_parameter_attributes!(parameter; $($attribute),*);
        parameter
    }};
    ($name:tt : $kind:ident($($type:tt)*)) => {
        $crate::backend::c::syntax::Parameter::named(
            $crate::backend::c::syntax::c_type!($kind($($type)*)),
            $name,
        )
    };
    ($name:tt : { $($type:tt)* }) => {
        $crate::backend::c::syntax::Parameter::named({ $($type)* }, $name)
    };
}

macro_rules! c_parameters {
    ($($parameter:tt)*) => {{
        #[allow(unused_mut, clippy::vec_init_then_push)]
        let mut parameters = Vec::from([]);
        $crate::backend::c::syntax::c_parameters_items!(parameters; $($parameter)*);
        parameters
    }};
}

macro_rules! c_parameters_items {
    ($parameters:ident;) => {};
    ($parameters:ident; {{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $parameters.extend({ $($rust)* });
        $crate::backend::c::syntax::c_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; { $($rust:tt)* } $(, $($rest:tt)*)?) => {
        $parameters.push({ $($rust)* });
        $crate::backend::c::syntax::c_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; $name:tt : $kind:ident($($type:tt)*) [$($attribute:ident),* $(,)?], $($rest:tt)*) => {
        $parameters.push($crate::backend::c::syntax::c_parameter!(
            $name : $kind($($type)*) [$($attribute),*]
        ));
        $crate::backend::c::syntax::c_parameters_items!($parameters; $($rest)*);
    };
    ($parameters:ident; $name:tt : { $($type:tt)* } [$($attribute:ident),* $(,)?], $($rest:tt)*) => {
        $parameters.push($crate::backend::c::syntax::c_parameter!(
            $name : { $($type)* } [$($attribute),*]
        ));
        $crate::backend::c::syntax::c_parameters_items!($parameters; $($rest)*);
    };
    ($parameters:ident; $name:tt : $kind:ident($($type:tt)*), $($rest:tt)*) => {
        $parameters.push($crate::backend::c::syntax::c_parameter!($name : $kind($($type)*)));
        $crate::backend::c::syntax::c_parameters_items!($parameters; $($rest)*);
    };
    ($parameters:ident; $name:tt : { $($type:tt)* }, $($rest:tt)*) => {
        $parameters.push($crate::backend::c::syntax::c_parameter!($name : { $($type)* }));
        $crate::backend::c::syntax::c_parameters_items!($parameters; $($rest)*);
    };
    ($parameters:ident; $name:tt : $kind:ident($($type:tt)*) [$($attribute:ident),* $(,)?] $(,)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter!(
            $name : $kind($($type)*) [$($attribute),*]
        ));
    };
    ($parameters:ident; $name:tt : { $($type:tt)* } [$($attribute:ident),* $(,)?] $(,)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter!(
            $name : { $($type)* } [$($attribute),*]
        ));
    };
    ($parameters:ident; $name:tt : $kind:ident($($type:tt)*) $(,)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter!($name : $kind($($type)*)));
    };
    ($parameters:ident; $name:tt : { $($type:tt)* } $(,)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter!($name : { $($type)* }));
    };
}

macro_rules! c_signature_from_parts {
    ($constructor:ident; $name:tt; [$($parameter:tt)*]; $kind:ident($($result:tt)*)) => {
        $crate::backend::c::syntax::FunctionSignature::$constructor(
            $crate::backend::c::syntax::c_type!($kind($($result)*)),
            $name,
            $crate::backend::c::syntax::c_parameters!($($parameter)*),
        )
    };
    ($constructor:ident; $name:tt; [$($parameter:tt)*]; { $($result:tt)* }) => {
        $crate::backend::c::syntax::FunctionSignature::$constructor(
            { $($result)* },
            $name,
            $crate::backend::c::syntax::c_parameters!($($parameter)*),
        )
    };
}

macro_rules! c_signature {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::c::syntax::c_signature_from_parts!(
            new; $name; [$($parameter)*]; $($result)+
        )
    };
    (static fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::c::syntax::c_signature_from_parts!(
            static_function; $name; [$($parameter)*]; $($result)+
        )
    };
    (static inline fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::c::syntax::c_signature_from_parts!(
            static_inline; $name; [$($parameter)*]; $($result)+
        )
    };
    (noreturn fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::c::syntax::c_signature_from_parts!(
            no_return; $name; [$($parameter)*]; $($result)+
        )
    };
}

pub(in crate::backend) use {
    c_parameter, c_parameter_attributes, c_parameters, c_parameters_items, c_signature,
    c_signature_from_parts, c_type,
};
