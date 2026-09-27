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

macro_rules! c_aggregate_field_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    ($name:tt : $kind:ident($($ty:tt)*)) => {
        $crate::backend::c::syntax::AggregateField::variable(
            $crate::backend::c::syntax::c_type_normalized!($kind($($ty)*)),
            $crate::backend::c::syntax::c_scalar_normalized!($name),
        )
    };
    ($name:tt : (@rust $($ty:tt)*)) => {
        $crate::backend::c::syntax::AggregateField::variable(
            { $($ty)* },
            $crate::backend::c::syntax::c_scalar_normalized!($name),
        )
    };
    (fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*)) => {
        $crate::backend::c::syntax::AggregateField::function_pointer(
            $crate::backend::c::syntax::c_type_normalized!($kind($($result)*)),
            $crate::backend::c::syntax::c_scalar_normalized!($name),
            $crate::backend::c::syntax::c_parameters_normalized!($($parameter)*),
        )
    };
    (fn $name:tt($($parameter:tt)*) -> (@rust $($result:tt)*)) => {
        $crate::backend::c::syntax::AggregateField::function_pointer(
            { $($result)* },
            $crate::backend::c::syntax::c_scalar_normalized!($name),
            $crate::backend::c::syntax::c_parameters_normalized!($($parameter)*),
        )
    };
    (struct $name:tt { $($field:tt)* }) => {
        $crate::backend::c::syntax::AggregateField::aggregate(
            $crate::backend::c::syntax::AggregateKind::Struct,
            $crate::backend::c::syntax::c_aggregate_fields_normalized!($($field)*),
            $crate::backend::c::syntax::c_scalar_normalized!($name),
        )
    };
    (union $name:tt { $($field:tt)* }) => {
        $crate::backend::c::syntax::AggregateField::aggregate(
            $crate::backend::c::syntax::AggregateKind::Union,
            $crate::backend::c::syntax::c_aggregate_fields_normalized!($($field)*),
            $crate::backend::c::syntax::c_scalar_normalized!($name),
        )
    };
}

macro_rules! c_aggregate_fields_normalized {
    ($($field:tt)*) => {{
        #[allow(unused_mut, clippy::vec_init_then_push)]
        let mut fields = Vec::from([]);
        $crate::backend::c::syntax::c_aggregate_fields_items_normalized!(fields; $($field)*);
        fields
    }};
}

macro_rules! c_aggregate_fields_items_normalized {
    ($fields:ident;) => {};
    ($fields:ident; ...(@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $fields.extend({ $($rust)* });
        $crate::backend::c::syntax::c_aggregate_fields_items_normalized!($fields; $($($rest)*)?);
    };
    ($fields:ident; (@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $fields.push({ $($rust)* });
        $crate::backend::c::syntax::c_aggregate_fields_items_normalized!($fields; $($($rest)*)?);
    };
    ($fields:ident; $name:tt : $kind:ident($($ty:tt)*) $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::c::syntax::c_aggregate_field_normalized!($name : $kind($($ty)*)));
        $crate::backend::c::syntax::c_aggregate_fields_items_normalized!($fields; $($($rest)*)?);
    };
    ($fields:ident; $name:tt : (@rust $($ty:tt)*) $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::c::syntax::c_aggregate_field_normalized!($name : (@rust $($ty)*)));
        $crate::backend::c::syntax::c_aggregate_fields_items_normalized!($fields; $($($rest)*)?);
    };
    ($fields:ident; fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*) $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::c::syntax::c_aggregate_field_normalized! {
            fn $name($($parameter)*) -> $kind($($result)*)
        });
        $crate::backend::c::syntax::c_aggregate_fields_items_normalized!($fields; $($($rest)*)?);
    };
    ($fields:ident; fn $name:tt($($parameter:tt)*) -> (@rust $($result:tt)*) $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::c::syntax::c_aggregate_field_normalized! {
            fn $name($($parameter)*) -> (@rust $($result)*)
        });
        $crate::backend::c::syntax::c_aggregate_fields_items_normalized!($fields; $($($rest)*)?);
    };
    ($fields:ident; struct $name:tt { $($field:tt)* } $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::c::syntax::c_aggregate_field_normalized! {
            struct $name { $($field)* }
        });
        $crate::backend::c::syntax::c_aggregate_fields_items_normalized!($fields; $($($rest)*)?);
    };
    ($fields:ident; union $name:tt { $($field:tt)* } $(, $($rest:tt)*)?) => {
        $fields.push($crate::backend::c::syntax::c_aggregate_field_normalized! {
            union $name { $($field)* }
        });
        $crate::backend::c::syntax::c_aggregate_fields_items_normalized!($fields; $($($rest)*)?);
    };
}

macro_rules! c_aggregate_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (struct $tag:tt { $($field:tt)* }) => {
        $crate::backend::c::syntax::AggregateDefinition::structure(
            $crate::backend::c::syntax::c_scalar_normalized!($tag),
            $crate::backend::c::syntax::c_aggregate_fields_normalized!($($field)*),
        )
    };
    (type $alias:tt = struct $tag:tt { $($field:tt)* }) => {
        $crate::backend::c::syntax::AggregateDefinition::typedef_structure(
            Some($crate::backend::c::syntax::c_scalar_normalized!($tag).to_string()),
            $crate::backend::c::syntax::c_aggregate_fields_normalized!($($field)*),
            $crate::backend::c::syntax::c_scalar_normalized!($alias),
        )
    };
    (type $alias:tt = struct { $($field:tt)* }) => {
        $crate::backend::c::syntax::AggregateDefinition::typedef_structure(
            None,
            $crate::backend::c::syntax::c_aggregate_fields_normalized!($($field)*),
            $crate::backend::c::syntax::c_scalar_normalized!($alias),
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

macro_rules! c_aggregate_field {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_aggregate_field_normalized]; $($syntax)*
        )
    };
}

macro_rules! c_aggregate_fields {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_aggregate_fields_normalized]; $($syntax)*
        )
    };
}

macro_rules! c_aggregate {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::c::syntax::c_aggregate_normalized]; $($syntax)*
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
    c_aggregate_field_normalized, c_aggregate_fields_items_normalized,
    c_aggregate_fields_normalized, c_aggregate_normalized, c_comment_normalized,
    c_declaration_normalized, c_parameter_attributes_normalized, c_parameter_normalized,
    c_parameters_items_normalized, c_parameters_normalized, c_scalar_normalized,
    c_signature_from_parts_normalized, c_signature_normalized, c_type_normalized,
    c_variable_normalized,
};

pub(in crate::backend) use {
    c_aggregate, c_aggregate_field, c_aggregate_fields, c_comment, c_declaration, c_parameter,
    c_parameters, c_signature, c_type, c_variable,
};
