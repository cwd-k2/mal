macro_rules! c_scalar {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    ($literal:literal) => { $literal };
}

macro_rules! c_comment {
    ($text:tt) => {
        $crate::backend::c::syntax::Comment::new($crate::backend::c::syntax::c_scalar!($text))
    };
}

macro_rules! c_type {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (named($name:tt)) => {
        $crate::backend::c::syntax::TypeName::named(
            $crate::backend::c::syntax::c_scalar!($name),
        )
    };
    (struct($name:tt)) => {
        $crate::backend::c::syntax::TypeName::structure(
            $crate::backend::c::syntax::c_scalar!($name),
        )
    };
    (const(named($name:tt))) => {
        $crate::backend::c::syntax::TypeName::const_named(
            $crate::backend::c::syntax::c_scalar!($name),
        )
    };
    (ptr($($inner:tt)+)) => {
        ($crate::backend::c::syntax::c_type!($($inner)+)).pointer()
    };
}

macro_rules! c_variable {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    ($name:tt : $kind:ident($($ty:tt)*)) => {
        $crate::backend::c::syntax::VariableDeclaration::new(
            $crate::backend::c::syntax::c_type!($kind($($ty)*)),
            $crate::backend::c::syntax::c_scalar!($name),
        )
    };
    ($name:tt : {{ $($ty:tt)* }}) => {
        $crate::backend::c::syntax::VariableDeclaration::new(
            { $($ty)* },
            $crate::backend::c::syntax::c_scalar!($name),
        )
    };
    (array $name:tt : $kind:ident($($ty:tt)*); size $size:tt) => {
        $crate::backend::c::syntax::VariableDeclaration::array(
            $crate::backend::c::syntax::c_type!($kind($($ty)*)),
            $crate::backend::c::syntax::c_scalar!($name),
            $crate::backend::c::syntax::c_expr_child!($size),
        )
    };
    (array $name:tt : {{ $($ty:tt)* }}; size $size:tt) => {
        $crate::backend::c::syntax::VariableDeclaration::array(
            { $($ty)* },
            $crate::backend::c::syntax::c_scalar!($name),
            $crate::backend::c::syntax::c_expr_child!($size),
        )
    };
}

macro_rules! c_aggregate_field {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    ($name:tt : $kind:ident($($ty:tt)*)) => {
        $crate::backend::c::syntax::AggregateField::variable(
            $crate::backend::c::syntax::c_type!($kind($($ty)*)),
            $crate::backend::c::syntax::c_scalar!($name),
        )
    };
    ($name:tt : {{ $($ty:tt)* }}) => {
        $crate::backend::c::syntax::AggregateField::variable(
            { $($ty)* },
            $crate::backend::c::syntax::c_scalar!($name),
        )
    };
    (fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*)) => {
        $crate::backend::c::syntax::AggregateField::function_pointer(
            $crate::backend::c::syntax::c_type!($kind($($result)*)),
            $crate::backend::c::syntax::c_scalar!($name),
            $crate::backend::c::syntax::c_parameters!($($parameter)*),
        )
    };
    (fn $name:tt($($parameter:tt)*) -> {{ $($result:tt)* }}) => {
        $crate::backend::c::syntax::AggregateField::function_pointer(
            { $($result)* },
            $crate::backend::c::syntax::c_scalar!($name),
            $crate::backend::c::syntax::c_parameters!($($parameter)*),
        )
    };
    (struct $name:tt { $($field:tt)* }) => {
        $crate::backend::c::syntax::AggregateField::aggregate(
            $crate::backend::c::syntax::AggregateKind::Struct,
            $crate::backend::c::syntax::c_aggregate_fields!($($field)*),
            $crate::backend::c::syntax::c_scalar!($name),
        )
    };
    (union $name:tt { $($field:tt)* }) => {
        $crate::backend::c::syntax::AggregateField::aggregate(
            $crate::backend::c::syntax::AggregateKind::Union,
            $crate::backend::c::syntax::c_aggregate_fields!($($field)*),
            $crate::backend::c::syntax::c_scalar!($name),
        )
    };
}

macro_rules! c_aggregate_fields {
    ($($field:tt)*) => {{
        #[allow(unused_mut, clippy::vec_init_then_push)]
        let mut fields = Vec::from([]);
        $crate::backend::c::syntax::c_aggregate_fields_items!(fields; $($field)*);
        fields
    }};
}

macro_rules! c_aggregate_fields_items {
    ($fields:ident;) => {};
    ($fields:ident; ...{{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $fields.extend({ $($rust)* });
        $crate::backend::c::syntax::c_aggregate_fields_items!($fields; $($($rest)*)?);
    };
    ($fields:ident; {{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $fields.push({ $($rust)* });
        $crate::backend::c::syntax::c_aggregate_fields_items!($fields; $($($rest)*)?);
    };
    ($fields:ident; $name:tt : $kind:ident($($ty:tt)*) $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::c::syntax::c_aggregate_field!($name : $kind($($ty)*)));
        $crate::backend::c::syntax::c_aggregate_fields_items!($fields; $($($rest)*)?);
    };
    ($fields:ident; $name:tt : {{ $($ty:tt)* }} $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::c::syntax::c_aggregate_field!($name : {{ $($ty)* }}));
        $crate::backend::c::syntax::c_aggregate_fields_items!($fields; $($($rest)*)?);
    };
    ($fields:ident; fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*) $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::c::syntax::c_aggregate_field!(
            fn $name($($parameter)*) -> $kind($($result)*)
        ));
        $crate::backend::c::syntax::c_aggregate_fields_items!($fields; $($($rest)*)?);
    };
    ($fields:ident; fn $name:tt($($parameter:tt)*) -> {{ $($result:tt)* }} $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::c::syntax::c_aggregate_field!(
            fn $name($($parameter)*) -> {{ $($result)* }}
        ));
        $crate::backend::c::syntax::c_aggregate_fields_items!($fields; $($($rest)*)?);
    };
    ($fields:ident; struct $name:tt { $($field:tt)* } $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::c::syntax::c_aggregate_field!(
            struct $name { $($field)* }
        ));
        $crate::backend::c::syntax::c_aggregate_fields_items!($fields; $($($rest)*)?);
    };
    ($fields:ident; union $name:tt { $($field:tt)* } $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::c::syntax::c_aggregate_field!(
            union $name { $($field)* }
        ));
        $crate::backend::c::syntax::c_aggregate_fields_items!($fields; $($($rest)*)?);
    };
}

macro_rules! c_aggregate {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (struct $tag:tt { $($field:tt)* }) => {
        $crate::backend::c::syntax::AggregateDefinition::structure(
            $crate::backend::c::syntax::c_scalar!($tag),
            $crate::backend::c::syntax::c_aggregate_fields!($($field)*),
        )
    };
    (type $alias:tt = struct $tag:tt { $($field:tt)* }) => {
        $crate::backend::c::syntax::AggregateDefinition::typedef_structure(
            Some($crate::backend::c::syntax::c_scalar!($tag).to_string()),
            $crate::backend::c::syntax::c_aggregate_fields!($($field)*),
            $crate::backend::c::syntax::c_scalar!($alias),
        )
    };
    (type $alias:tt = struct { $($field:tt)* }) => {
        $crate::backend::c::syntax::AggregateDefinition::typedef_structure(
            None,
            $crate::backend::c::syntax::c_aggregate_fields!($($field)*),
            $crate::backend::c::syntax::c_scalar!($alias),
        )
    };
}

macro_rules! c_declaration {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (fn {{ $($signature:tt)* }};) => {
        $crate::backend::c::syntax::Declaration::function({ $($signature)* })
    };
    (type $alias:tt = $kind:ident($($source:tt)*)) => {
        $crate::backend::c::syntax::Declaration::type_alias(
            $crate::backend::c::syntax::c_type!($kind($($source)*)),
            $crate::backend::c::syntax::c_scalar!($alias),
        )
    };
    (type $alias:tt = {{ $($source:tt)* }}) => {
        $crate::backend::c::syntax::Declaration::type_alias(
            { $($source)* },
            $crate::backend::c::syntax::c_scalar!($alias),
        )
    };
    (static_assert($condition:tt, $message:tt);) => {
        $crate::backend::c::syntax::Declaration::static_assert(
            $crate::backend::c::syntax::c_expr_child!($condition),
            $crate::backend::c::syntax::c_scalar!($message),
        )
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
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (_ : $kind:ident($($type:tt)*)) => {
        $crate::backend::c::syntax::Parameter::unnamed(
            $crate::backend::c::syntax::c_type!($kind($($type)*)),
        )
    };
    (_ : {{ $($type:tt)* }}) => {
        $crate::backend::c::syntax::Parameter::unnamed({ $($type)* })
    };
    (#[maybe_unused] $name:tt : $kind:ident($($type:tt)*)) => {{
        let mut parameter = $crate::backend::c::syntax::Parameter::named(
            $crate::backend::c::syntax::c_type!($kind($($type)*)),
            $crate::backend::c::syntax::c_scalar!($name),
        );
        $crate::backend::c::syntax::c_parameter_attributes!(parameter; maybe_unused);
        parameter
    }};
    (#[maybe_unused] $name:tt : {{ $($type:tt)* }}) => {{
        let mut parameter = $crate::backend::c::syntax::Parameter::named(
            { $($type)* },
            $crate::backend::c::syntax::c_scalar!($name),
        );
        $crate::backend::c::syntax::c_parameter_attributes!(parameter; maybe_unused);
        parameter
    }};
    ($name:tt : $kind:ident($($type:tt)*)) => {
        $crate::backend::c::syntax::Parameter::named(
            $crate::backend::c::syntax::c_type!($kind($($type)*)),
            $crate::backend::c::syntax::c_scalar!($name),
        )
    };
    ($name:tt : {{ $($type:tt)* }}) => {
        $crate::backend::c::syntax::Parameter::named(
            { $($type)* },
            $crate::backend::c::syntax::c_scalar!($name),
        )
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
    ($parameters:ident; ...{{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $parameters.extend({ $($rust)* });
        $crate::backend::c::syntax::c_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; {{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $parameters.push({ $($rust)* });
        $crate::backend::c::syntax::c_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; _ : $kind:ident($($type:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter!(_ : $kind($($type)*)));
        $crate::backend::c::syntax::c_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; _ : {{ $($type:tt)* }} $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter!(_ : {{ $($type)* }}));
        $crate::backend::c::syntax::c_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; #[maybe_unused] $name:tt : $kind:ident($($type:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter!(
            #[maybe_unused] $name : $kind($($type)*)
        ));
        $crate::backend::c::syntax::c_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; #[maybe_unused] $name:tt : {{ $($type:tt)* }} $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter!(
            #[maybe_unused] $name : {{ $($type)* }}
        ));
        $crate::backend::c::syntax::c_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; $name:tt : $kind:ident($($type:tt)*) $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter!($name : $kind($($type)*)));
        $crate::backend::c::syntax::c_parameters_items!($parameters; $($($rest)*)?);
    };
    ($parameters:ident; $name:tt : {{ $($type:tt)* }} $(, $($rest:tt)*)?) => {
        $parameters.push($crate::backend::c::syntax::c_parameter!($name : {{ $($type)* }}));
        $crate::backend::c::syntax::c_parameters_items!($parameters; $($($rest)*)?);
    };
}

macro_rules! c_signature_from_parts {
    ($constructor:ident; $name:tt; [$($parameter:tt)*]; $kind:ident($($result:tt)*)) => {
        $crate::backend::c::syntax::FunctionSignature::$constructor(
            $crate::backend::c::syntax::c_type!($kind($($result)*)),
            $crate::backend::c::syntax::c_scalar!($name),
            $crate::backend::c::syntax::c_parameters!($($parameter)*),
        )
    };
    ($constructor:ident; $name:tt; [$($parameter:tt)*]; {{ $($result:tt)* }}) => {
        $crate::backend::c::syntax::FunctionSignature::$constructor(
            { $($result)* },
            $crate::backend::c::syntax::c_scalar!($name),
            $crate::backend::c::syntax::c_parameters!($($parameter)*),
        )
    };
}

macro_rules! c_signature {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::c::syntax::c_signature_from_parts!(
            new; $name; [$($parameter)*]; $($result)+
        )
    };
    (#[static] fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::c::syntax::c_signature_from_parts!(
            static_function; $name; [$($parameter)*]; $($result)+
        )
    };
    (#[static] #[inline] fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::c::syntax::c_signature_from_parts!(
            static_inline; $name; [$($parameter)*]; $($result)+
        )
    };
    (#[noreturn] fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {
        $crate::backend::c::syntax::c_signature_from_parts!(
            no_return; $name; [$($parameter)*]; $($result)+
        )
    };
    (#[static] #[inline] #[noreturn] fn $name:tt($($parameter:tt)*) -> $($result:tt)+) => {{
        let signature = $crate::backend::c::syntax::c_signature!(
            fn $name($($parameter)*) -> $($result)+
        );
        signature.with_specifiers([
            $crate::backend::c::syntax::FunctionSpecifier::Static,
            $crate::backend::c::syntax::FunctionSpecifier::Inline,
            $crate::backend::c::syntax::FunctionSpecifier::NoReturn,
        ])
    }};
}

pub(in crate::backend) use {
    c_aggregate, c_aggregate_field, c_aggregate_fields, c_aggregate_fields_items, c_comment,
    c_declaration, c_parameter, c_parameter_attributes, c_parameters, c_parameters_items, c_scalar,
    c_signature, c_signature_from_parts, c_type, c_variable,
};
