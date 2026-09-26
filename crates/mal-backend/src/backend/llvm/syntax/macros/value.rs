macro_rules! llvm_value {
    (rust $value:expr) => {
        Some($value)
    };
    (typed $ty:expr => $value:expr) => {
        $crate::backend::llvm::syntax::TypedValue::new($ty, $value)
    };
}

macro_rules! llvm_values_item {
    ($values:ident; (rust $value:expr)) => {
        $values.push($value)
    };
    ($values:ident; (typed $ty:expr => $value:expr)) => {
        $values.push($crate::backend::llvm::syntax::llvm_value!(typed $ty => $value)?)
    };
    ($values:ident; (extend $more:expr)) => {
        $values.extend($more)
    };
    ($values:ident; (typed_extend $more:expr)) => {
        $values.extend(
            $more
                .into_iter()
                .map(|(ty, value)| $crate::backend::llvm::syntax::llvm_value!(typed ty => value))
                .collect::<Option<Vec<_>>>()?,
        )
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
