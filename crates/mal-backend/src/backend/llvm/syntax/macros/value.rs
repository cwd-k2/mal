macro_rules! llvm_value {
    ({{ $($rust:tt)* }}) => {
        Some({ $($rust)* })
    };
    (typed($ty:tt, $value:tt)) => {
        $crate::backend::llvm::syntax::TypedValue::new(
            $crate::backend::llvm::syntax::llvm_instruction_type!($ty),
            $crate::backend::llvm::syntax::llvm_instruction_atom!($value),
        )
    };
    ({ $($rust:tt)* }) => {
        Some({ $($rust)* })
    };
    (typed $ty:tt => $value:tt) => {
        $crate::backend::llvm::syntax::TypedValue::new(
            $crate::backend::llvm::syntax::llvm_instruction_type!($ty),
            $crate::backend::llvm::syntax::llvm_instruction_atom!($value),
        )
    };
}

macro_rules! llvm_values_item {
    ($values:ident; ...{{ $($rust:tt)* }}) => {
        $values.extend({ $($rust)* })
    };
    ($values:ident; {{ $($rust:tt)* }}) => {
        $values.push({ $($rust)* })
    };
    ($values:ident; typed($ty:tt, $value:tt)) => {
        $values.push($crate::backend::llvm::syntax::llvm_value!(typed($ty, $value))?)
    };
    ($values:ident; {{ $($rust:tt)* }}) => {
        $values.extend({ $($rust)* })
    };
    ($values:ident; { $($rust:tt)* }) => {
        $values.push({ $($rust)* })
    };
    ($values:ident; (typed $ty:tt => $value:tt)) => {
        $values.push($crate::backend::llvm::syntax::llvm_value!(typed $ty => $value)?)
    };
}

macro_rules! llvm_values {
    (@new $($value:tt)*) => {{
        #[allow(clippy::redundant_closure_call)]
        (|| {
            #[allow(unused_mut)]
            let mut values = Vec::from([]);
            $crate::backend::llvm::syntax::llvm_values_items!(values; $($value)*);
            Some(values)
        })()
    }};
    ($($value:tt),* $(,)?) => {{
        #[allow(clippy::redundant_closure_call)]
        (|| {
            #[allow(unused_mut)]
            let mut values = Vec::from([]);
            $(
                $crate::backend::llvm::syntax::llvm_values_item!(values; $value);
            )*
            Some(values)
        })()
    }};
}

macro_rules! llvm_values_items {
    ($values:ident;) => {};
    ($values:ident; ...{{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_values_item!($values; ...{{ $($rust)* }});
        $crate::backend::llvm::syntax::llvm_values_items!($values; $($($rest)*)?);
    };
    ($values:ident; {{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_values_item!($values; {{ $($rust)* }});
        $crate::backend::llvm::syntax::llvm_values_items!($values; $($($rest)*)?);
    };
    ($values:ident; typed($ty:tt, $value:tt) $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_values_item!($values; typed($ty, $value));
        $crate::backend::llvm::syntax::llvm_values_items!($values; $($($rest)*)?);
    };
}

pub(in crate::backend) use {llvm_value, llvm_values, llvm_values_item, llvm_values_items};
