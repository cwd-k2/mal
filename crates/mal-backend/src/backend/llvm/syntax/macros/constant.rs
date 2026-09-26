macro_rules! llvm_constant_child {
    ({ $($rust:tt)* }) => {
        Some({ $($rust)* })
    };
    ($constant:tt) => {
        $crate::backend::llvm::syntax::llvm_constant! $constant
    };
}

macro_rules! llvm_typed_constant_child {
    ({ $($rust:tt)* }) => {
        Some({ $($rust)* })
    };
    ($constant:tt) => {
        $crate::backend::llvm::syntax::llvm_typed_constant! $constant
    };
}

macro_rules! llvm_typed_constant {
    ({ $($rust:tt)* }) => {
        Some({ $($rust)* })
    };
    (typed $ty:expr => $constant:tt) => {{
        let ty = $ty;
        $crate::backend::llvm::syntax::llvm_constant_child!($constant)
            .map(|constant| $crate::backend::llvm::syntax::TypedConstant::new(ty, constant))
    }};
}

macro_rules! llvm_typed_constants_item {
    ($constants:ident; {{ $($rust:tt)* }}) => {
        $constants.extend({ $($rust)* })
    };
    ($constants:ident; { $($rust:tt)* }) => {
        $constants.push({ $($rust)* })
    };
    ($constants:ident; $constant:tt) => {
        $constants.push($crate::backend::llvm::syntax::llvm_typed_constant_child!($constant)?)
    };
}

macro_rules! llvm_typed_constants {
    ($($constant:tt),* $(,)?) => {{
        #[allow(clippy::redundant_closure_call)]
        (|| {
            let mut constants = Vec::new();
            $(
                $crate::backend::llvm::syntax::llvm_typed_constants_item!(constants; $constant);
            )*
            Some(constants)
        })()
    }};
}

macro_rules! llvm_constant {
    ({ $($rust:tt)* }) => {
        Some({ $($rust)* })
    };
    (atom $value:expr) => {
        $crate::backend::llvm::syntax::Constant::atom($value.to_string())
    };
    (zero) => {
        Some($crate::backend::llvm::syntax::Constant::ZeroInitializer)
    };
    (structure [$($field:tt),* $(,)?]) => {
        $crate::backend::llvm::syntax::llvm_typed_constants!($($field),*)
            .map($crate::backend::llvm::syntax::Constant::structure)
    };
    (get_element_ptr $element_type:expr; $pointer:tt; [$($index:tt),* $(,)?]) => {{
        $crate::backend::llvm::syntax::llvm_typed_constant_child!($pointer).and_then(|pointer| {
            $crate::backend::llvm::syntax::llvm_typed_constants!($($index),*).map(|indices| {
                $crate::backend::llvm::syntax::Constant::get_element_ptr(
                    $element_type,
                    pointer,
                    indices,
                )
            })
        })
    }};
    (unary $operator:expr; $operand:tt) => {
        $crate::backend::llvm::syntax::llvm_typed_constant_child!($operand)
            .map(|operand| $crate::backend::llvm::syntax::Constant::unary($operator, operand))
    };
    (binary $operator:expr; $left:tt; $right:tt) => {{
        let left = $crate::backend::llvm::syntax::llvm_typed_constant_child!($left);
        let right = $crate::backend::llvm::syntax::llvm_typed_constant_child!($right);
        left.zip(right).and_then(|(left, right)| {
            $crate::backend::llvm::syntax::Constant::binary($operator, left, right)
        })
    }};
    (cast $operator:expr; $operand:tt; $target:expr) => {
        $crate::backend::llvm::syntax::llvm_typed_constant_child!($operand).map(|operand| {
            $crate::backend::llvm::syntax::Constant::cast($operator, operand, $target)
        })
    };
}

pub(in crate::backend::llvm) use {
    llvm_constant, llvm_constant_child, llvm_typed_constant, llvm_typed_constant_child,
    llvm_typed_constants, llvm_typed_constants_item,
};
