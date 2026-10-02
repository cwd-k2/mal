//! C syntax macros for parameters and function signatures.

macro_rules! c_parameter_attributes_normalized {
    ($parameter:ident;) => {};
    ($parameter:ident; maybe_unused $(, $rest:ident)*) => {
        $parameter = $parameter.maybe_unused();
        $crate::backend::c::syntax::c_parameter_attributes_normalized!($parameter; $($rest),*);
    };
}

macro_rules! c_parameter_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (_ : $kind:ident($($type:tt)*)) => {
        $crate::backend::c::syntax::Parameter::unnamed(
            $crate::backend::c::syntax::c_type_normalized!($kind($($type)*)),
        )
    };
    (_ : (@rust $($type:tt)*)) => {
        $crate::backend::c::syntax::Parameter::unnamed({ $($type)* })
    };
    (#[maybe_unused] $name:tt : $kind:ident($($type:tt)*)) => {{
        let mut parameter = $crate::backend::c::syntax::Parameter::named(
            $crate::backend::c::syntax::c_type_normalized!($kind($($type)*)),
            $crate::backend::c::syntax::c_scalar_normalized!($name),
        );
        $crate::backend::c::syntax::c_parameter_attributes_normalized!(parameter; maybe_unused);
        parameter
    }};
    (#[maybe_unused] $name:tt : (@rust $($type:tt)*)) => {{
        let mut parameter = $crate::backend::c::syntax::Parameter::named(
            { $($type)* },
            $crate::backend::c::syntax::c_scalar_normalized!($name),
        );
        $crate::backend::c::syntax::c_parameter_attributes_normalized!(parameter; maybe_unused);
        parameter
    }};
    ($name:tt : $kind:ident($($type:tt)*)) => {
        $crate::backend::c::syntax::Parameter::named(
            $crate::backend::c::syntax::c_type_normalized!($kind($($type)*)),
            $crate::backend::c::syntax::c_scalar_normalized!($name),
        )
    };
    ($name:tt : (@rust $($type:tt)*)) => {
        $crate::backend::c::syntax::Parameter::named(
            { $($type)* },
            $crate::backend::c::syntax::c_scalar_normalized!($name),
        )
    };
}

macro_rules! c_parameters_normalized {
    ($($parameter:tt)*) => {{
        #[allow(unused_mut, clippy::vec_init_then_push)]
        let mut parameters = Vec::from([]);
        $crate::backend::c::syntax::c_parameters_items_normalized!(parameters; $($parameter)*);
        parameters
    }};
}

macro_rules! c_parameters_items_normalized {
    ($parameters:ident;) => {};
    ($parameters:ident; ...(@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.extend({ $($rust)* });
        $crate::backend::c::syntax::c_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; (@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push({ $($rust)* });
        $crate::backend::c::syntax::c_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; _ : $kind:ident($($type:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter_normalized!(_ : $kind($($type)*)));
        $crate::backend::c::syntax::c_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; _ : (@rust $($type:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter_normalized!(_ : (@rust $($type)*)));
        $crate::backend::c::syntax::c_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; #[maybe_unused] $name:tt : $kind:ident($($type:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter_normalized! {
            #[maybe_unused] $name : $kind($($type)*)
        });
        $crate::backend::c::syntax::c_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; #[maybe_unused] $name:tt : (@rust $($type:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter_normalized! {
            #[maybe_unused] $name : (@rust $($type)*)
        });
        $crate::backend::c::syntax::c_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; $name:tt : $kind:ident($($type:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter_normalized!($name : $kind($($type)*)));
        $crate::backend::c::syntax::c_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; $name:tt : (@rust $($type:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter_normalized!($name : (@rust $($type)*)));
        $crate::backend::c::syntax::c_parameters_items_normalized!($parameters; $($($rest)*)?);
    };
}

macro_rules! c_signature_from_parts_normalized {
    ($constructor:ident; $name:tt; [$($parameter:tt)*]; $kind:ident($($result:tt)*)) => {
        $crate::backend::c::syntax::FunctionSignature::$constructor(
            $crate::backend::c::syntax::c_type_normalized!($kind($($result)*)),
            $crate::backend::c::syntax::c_scalar_normalized!($name),
            $crate::backend::c::syntax::c_parameters_normalized!($($parameter)*),
        )
    };
    ($constructor:ident; $name:tt; [$($parameter:tt)*]; (@rust $($result:tt)*)) => {
        $crate::backend::c::syntax::FunctionSignature::$constructor(
            { $($result)* },
            $crate::backend::c::syntax::c_scalar_normalized!($name),
            $crate::backend::c::syntax::c_parameters_normalized!($($parameter)*),
        )
    };
}

macro_rules! c_signature_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::c::syntax::c_signature_from_parts_normalized! {
            new; $name; [$($parameter)*]; $($result)+
        }
    };
    (#[static] fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::c::syntax::c_signature_from_parts_normalized! {
            static_function; $name; [$($parameter)*]; $($result)+
        }
    };
    (#[static] #[inline] fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::c::syntax::c_signature_from_parts_normalized! {
            static_inline; $name; [$($parameter)*]; $($result)+
        }
    };
    (#[noreturn] fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::c::syntax::c_signature_from_parts_normalized! {
            no_return; $name; [$($parameter)*]; $($result)+
        }
    };
    (#[static] #[inline] #[noreturn] fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {{
        let signature = $crate::backend::c::syntax::c_signature_normalized! {
            fn $name($($parameter)*) -> $($result)+
        };
        signature.with_specifiers([
            $crate::backend::c::syntax::FunctionSpecifier::Static,
            $crate::backend::c::syntax::FunctionSpecifier::Inline,
            $crate::backend::c::syntax::FunctionSpecifier::NoReturn,
        ])
    }};
}

macro_rules! c_parameter {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_parameter_normalized]; $($syntax)*
        )
    };
}

macro_rules! c_parameters {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_parameters_normalized]; $($syntax)*
        )
    };
}

macro_rules! c_signature {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_signature_normalized]; $($syntax)*
        )
    };
}

pub(in crate::backend) use {
    c_parameter, c_parameter_attributes_normalized, c_parameter_normalized, c_parameters,
    c_parameters_items_normalized, c_parameters_normalized, c_signature,
    c_signature_from_parts_normalized, c_signature_normalized,
};
