macro_rules! llvm_typed_constant {
    (rust $constant:expr) => {
        Some($constant)
    };
    (typed $ty:expr => $constant:tt) => {{
        let ty = $ty;
        $crate::backend::llvm::syntax::llvm_constant! $constant
            .map(|constant| $crate::backend::llvm::syntax::TypedConstant::new(ty, constant))
    }};
}

macro_rules! llvm_typed_constants_item {
    ($constants:ident; (extend $more:expr)) => {
        $constants.extend($more)
    };
    ($constants:ident; $constant:tt) => {
        $constants.push($crate::backend::llvm::syntax::llvm_typed_constant! $constant?)
    };
}

macro_rules! llvm_typed_constants {
    ($($constant:tt),* $(,)?) => {{
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
    (rust $constant:expr) => {
        Some($constant)
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
        $crate::backend::llvm::syntax::llvm_typed_constant! $pointer .and_then(|pointer| {
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
        $crate::backend::llvm::syntax::llvm_typed_constant! $operand
            .map(|operand| $crate::backend::llvm::syntax::Constant::unary($operator, operand))
    };
    (binary $operator:expr; $left:tt; $right:tt) => {{
        let left = $crate::backend::llvm::syntax::llvm_typed_constant! $left;
        let right = $crate::backend::llvm::syntax::llvm_typed_constant! $right;
        left.zip(right).and_then(|(left, right)| {
            $crate::backend::llvm::syntax::Constant::binary($operator, left, right)
        })
    }};
    (cast $operator:expr; $operand:tt; $target:expr) => {
        $crate::backend::llvm::syntax::llvm_typed_constant! $operand.map(|operand| {
            $crate::backend::llvm::syntax::Constant::cast($operator, operand, $target)
        })
    };
}

pub(in crate::backend::llvm) use {
    llvm_constant, llvm_typed_constant, llvm_typed_constants, llvm_typed_constants_item,
};
