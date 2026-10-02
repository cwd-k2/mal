//! C syntax macros for aggregates and their fields.

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

pub(in crate::backend) use {
    c_aggregate, c_aggregate_field, c_aggregate_field_normalized, c_aggregate_fields,
    c_aggregate_fields_items_normalized, c_aggregate_fields_normalized, c_aggregate_normalized,
};
