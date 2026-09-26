macro_rules! llvm_value {
    ({ $($rust:tt)* }) => {
        Some({ $($rust)* })
    };
    (typed $ty:expr => $value:expr) => {
        $crate::backend::llvm::syntax::TypedValue::new($ty, $value)
    };
}

macro_rules! llvm_values_item {
    ($values:ident; {{ $($rust:tt)* }}) => {
        $values.extend({ $($rust)* })
    };
    ($values:ident; { $($rust:tt)* }) => {
        $values.push({ $($rust)* })
    };
    ($values:ident; (typed $ty:expr => $value:expr)) => {
        $values.push($crate::backend::llvm::syntax::llvm_value!(typed $ty => $value)?)
    };
}

macro_rules! llvm_values {
    ($($value:tt),* $(,)?) => {{
        (|| {
            let mut values = Vec::new();
            $(
                $crate::backend::llvm::syntax::llvm_values_item!(values; $value);
            )*
            Some(values)
        })()
    }};
}

pub(in crate::backend::llvm) use {llvm_value, llvm_values, llvm_values_item};
