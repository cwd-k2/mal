macro_rules! llvm_typed_constant_normalized {
    ((@rust $($rust:tt)*)) => {
        Some({ $($rust)* })
    };
    (typed($ty:tt, $kind:ident($($constant:tt)*) $(,)?)) => {{
        let ty = $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($ty);
        $crate::backend::llvm::syntax::llvm_constant_normalized!($kind($($constant)*))
            .map(|constant| $crate::backend::llvm::syntax::TypedConstant::new(ty, constant))
    }};
    (typed($ty:tt, $kind:ident { $($constant:tt)* } $(,)?)) => {{
        let ty = $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($ty);
        $crate::backend::llvm::syntax::llvm_constant_normalized!($kind { $($constant)* })
            .map(|constant| $crate::backend::llvm::syntax::TypedConstant::new(ty, constant))
    }};
    (typed($ty:tt, (@rust $($constant:tt)*) $(,)?)) => {{
        let ty = $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($ty);
        Some($crate::backend::llvm::syntax::TypedConstant::new(
            ty,
            { $($constant)* },
        ))
    }};
}
macro_rules! llvm_typed_constants_item_normalized {
    ($constants:ident; ...(@rust $($rust:tt)*)) => {
        $constants.extend({ $($rust)* })
    };
    ($constants:ident; (@rust $($rust:tt)*)) => {
        $constants.push({ $($rust)* })
    };
    ($constants:ident; typed($($argument:tt)*)) => {
        $constants.push($crate::backend::llvm::syntax::llvm_typed_constant_normalized!(typed($($argument)*))?)
    };
}
macro_rules! llvm_typed_constants_normalized {
    ($($constant:tt)*) => {{
        #[allow(clippy::redundant_closure_call)]
        (|| {
            #[allow(unused_mut)]
            let mut constants = Vec::from([]);
            $crate::backend::llvm::syntax::llvm_typed_constants_items_normalized!(constants; $($constant)*);
            Some(constants)
        })()
    }};
}
macro_rules! llvm_typed_constants_items_normalized {
    ($constants:ident;) => {};
    ($constants:ident; ...(@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_typed_constants_item_normalized!($constants; ...(@rust $($rust)*));
        $crate::backend::llvm::syntax::llvm_typed_constants_items_normalized!($constants; $($($rest)*)?);
    };
    ($constants:ident; (@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_typed_constants_item_normalized!($constants; (@rust $($rust)*));
        $crate::backend::llvm::syntax::llvm_typed_constants_items_normalized!($constants; $($($rest)*)?);
    };
    ($constants:ident; typed($($argument:tt)*) $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_typed_constants_item_normalized!($constants; typed($($argument)*));
        $crate::backend::llvm::syntax::llvm_typed_constants_items_normalized!($constants; $($($rest)*)?);
    };
}

macro_rules! llvm_constant_normalized {
    ((@rust $($rust:tt)*)) => {
        Some({ $($rust)* })
    };
    (atom($value:tt)) => {
        $crate::backend::llvm::syntax::Constant::atom(
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($value).to_string(),
        )
    };
    (structure([$($field:tt)*])) => {
        $crate::backend::llvm::syntax::llvm_typed_constants_normalized!($($field)*)
            .map($crate::backend::llvm::syntax::Constant::structure)
    };
    (get_element_ptr {
        element_type: $element_type:tt,
        pointer: $pointer_kind:ident($($pointer:tt)*),
        indices: [$($index:tt)*] $(,)?
    }) => {{
        $crate::backend::llvm::syntax::llvm_typed_constant_normalized!($pointer_kind($($pointer)*)).and_then(
            |pointer| $crate::backend::llvm::syntax::llvm_typed_constants_normalized!($($index)*)
                .map(|indices| $crate::backend::llvm::syntax::Constant::get_element_ptr(
                    $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($element_type),
                    pointer,
                    indices,
                )),
        )
    }};
    (get_element_ptr {
        element_type: $element_type:tt,
        pointer: (@rust $($pointer:tt)*),
        indices: [$($index:tt)*] $(,)?
    }) => {{
        Some({ $($pointer)* }).and_then(|pointer| {
            $crate::backend::llvm::syntax::llvm_typed_constants_normalized!($($index)*)
                .map(|indices| $crate::backend::llvm::syntax::Constant::get_element_ptr(
                    $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($element_type),
                    pointer,
                    indices,
                ))
        })
    }};
    (unary {
        operator: $operator:tt,
        operand: $operand_kind:ident($($operand:tt)*) $(,)?
    }) => {
        $crate::backend::llvm::syntax::llvm_typed_constant_normalized!($operand_kind($($operand)*)).map(
            |operand| $crate::backend::llvm::syntax::Constant::unary(
                $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($operator),
                operand,
            ),
        )
    };
    (unary { operator: $operator:tt, operand: (@rust $($operand:tt)*) $(,)? }) => {
        Some($crate::backend::llvm::syntax::Constant::unary(
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($operator),
            { $($operand)* },
        ))
    };
    (binary {
        operator: $operator:tt,
        left: $left_kind:ident($($left:tt)*),
        right: $right_kind:ident($($right:tt)*) $(,)?
    }) => {{
        let left = $crate::backend::llvm::syntax::llvm_typed_constant_normalized!($left_kind($($left)*));
        let right = $crate::backend::llvm::syntax::llvm_typed_constant_normalized!($right_kind($($right)*));
        left.zip(right).and_then(|(left, right)| {
            $crate::backend::llvm::syntax::Constant::binary(
                $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($operator), left, right,
            )
        })
    }};
    (binary { operator: $operator:tt, left: (@rust $($left:tt)*), right: (@rust $($right:tt)*) $(,)? }) => {
        $crate::backend::llvm::syntax::Constant::binary(
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($operator),
            { $($left)* },
            { $($right)* },
        )
    };
    (cast {
        operator: $operator:tt,
        operand: (@rust $($operand:tt)*),
        to: $target:tt $(,)?
    }) => {
        Some($crate::backend::llvm::syntax::Constant::cast(
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($operator),
            { $($operand)* },
            $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($target),
        ))
    };
    (zero) => {
        Some($crate::backend::llvm::syntax::Constant::ZeroInitializer)
    };
}

macro_rules! llvm_typed_constant {
    (@normalized $($syntax:tt)*) => {
        $crate::backend::llvm::syntax::llvm_typed_constant_normalized!($($syntax)*)
    };
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::llvm::syntax::llvm_typed_constant]; $($syntax)*
        )
    };
}

macro_rules! llvm_constant {
    (@normalized $($syntax:tt)*) => {
        $crate::backend::llvm::syntax::llvm_constant_normalized!($($syntax)*)
    };
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::llvm::syntax::llvm_constant]; $($syntax)*
        )
    };
}

pub(in crate::backend) use {
    llvm_constant_normalized, llvm_typed_constant_normalized, llvm_typed_constants_item_normalized,
    llvm_typed_constants_items_normalized, llvm_typed_constants_normalized,
};

pub(in crate::backend) use {llvm_constant, llvm_typed_constant};
