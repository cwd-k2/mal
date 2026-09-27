macro_rules! llvm_value_normalized {
    ((@rust $($rust:tt)*)) => {
        Some({ $($rust)* })
    };
    (typed($ty:tt, $value:tt $(,)?)) => {
        $crate::backend::llvm::syntax::TypedValue::new(
            $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($ty),
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($value),
        )
    };
}

macro_rules! llvm_values_item_normalized {
    ($values:ident; ...(@rust $($rust:tt)*)) => {
        $values.extend({ $($rust)* })
    };
    ($values:ident; (@rust $($rust:tt)*)) => {
        $values.push({ $($rust)* })
    };
    ($values:ident; typed($ty:tt, $value:tt $(,)?)) => {
        $values.push($crate::backend::llvm::syntax::llvm_value_normalized!(typed($ty, $value))?)
    };
}

macro_rules! llvm_values_normalized {
    ($($value:tt)*) => {{
        #[allow(clippy::redundant_closure_call)]
        (|| {
            #[allow(unused_mut)]
            let mut values = Vec::from([]);
            $crate::backend::llvm::syntax::llvm_values_items_normalized!(values; $($value)*);
            Some(values)
        })()
    }};
}

macro_rules! llvm_values_items_normalized {
    ($values:ident;) => {};
    ($values:ident; ...(@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_values_item_normalized!($values; ...(@rust $($rust)*));
        $crate::backend::llvm::syntax::llvm_values_items_normalized!($values; $($($rest)*)?);
    };
    ($values:ident; (@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_values_item_normalized!($values; (@rust $($rust)*));
        $crate::backend::llvm::syntax::llvm_values_items_normalized!($values; $($($rest)*)?);
    };
    ($values:ident; typed($ty:tt, $value:tt $(,)?) $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_values_item_normalized!($values; typed($ty, $value));
        $crate::backend::llvm::syntax::llvm_values_items_normalized!($values; $($($rest)*)?);
    };
}

pub(in crate::backend) use {
    llvm_value_normalized, llvm_values_item_normalized, llvm_values_items_normalized,
    llvm_values_normalized,
};
