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
    (typed $ty:tt => $constant:tt) => {{
        let ty = $crate::backend::llvm::syntax::llvm_instruction_type!($ty);
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
    (atom $value:tt) => {
        $crate::backend::llvm::syntax::Constant::atom(
            $crate::backend::llvm::syntax::llvm_instruction_atom!($value).to_string(),
        )
    };
    (zero) => {
        Some($crate::backend::llvm::syntax::Constant::ZeroInitializer)
    };
    (structure [$($field:tt),* $(,)?]) => {
        $crate::backend::llvm::syntax::llvm_typed_constants!($($field),*)
            .map($crate::backend::llvm::syntax::Constant::structure)
    };
    (get_element_ptr $element_type:tt; $pointer:tt; [$($index:tt),* $(,)?]) => {{
        $crate::backend::llvm::syntax::llvm_typed_constant_child!($pointer).and_then(|pointer| {
            $crate::backend::llvm::syntax::llvm_typed_constants!($($index),*).map(|indices| {
                $crate::backend::llvm::syntax::Constant::get_element_ptr(
                    $crate::backend::llvm::syntax::llvm_instruction_type!($element_type),
                    pointer,
                    indices,
                )
            })
        })
    }};
    (unary $operator:tt; $operand:tt) => {
        $crate::backend::llvm::syntax::llvm_typed_constant_child!($operand)
            .map(|operand| $crate::backend::llvm::syntax::Constant::unary(
                $crate::backend::llvm::syntax::llvm_instruction_atom!($operator),
                operand,
            ))
    };
    (binary $operator:tt; $left:tt; $right:tt) => {{
        let left = $crate::backend::llvm::syntax::llvm_typed_constant_child!($left);
        let right = $crate::backend::llvm::syntax::llvm_typed_constant_child!($right);
        left.zip(right).and_then(|(left, right)| {
            $crate::backend::llvm::syntax::Constant::binary(
                $crate::backend::llvm::syntax::llvm_instruction_atom!($operator),
                left,
                right,
            )
        })
    }};
    (cast $operator:tt; $operand:tt; $target:tt) => {
        $crate::backend::llvm::syntax::llvm_typed_constant_child!($operand).map(|operand| {
            $crate::backend::llvm::syntax::Constant::cast(
                $crate::backend::llvm::syntax::llvm_instruction_atom!($operator),
                operand,
                $crate::backend::llvm::syntax::llvm_instruction_type!($target),
            )
        })
    };
}

pub(in crate::backend) use {
    llvm_constant, llvm_constant_child, llvm_typed_constant, llvm_typed_constant_child,
    llvm_typed_constants, llvm_typed_constants_item,
};
