macro_rules! llvm_typed_constant {
    ({{ $($rust:tt)* }}) => {
        Some({ $($rust)* })
    };
    (typed($ty:tt, $kind:ident($($constant:tt)*))) => {{
        let ty = $crate::backend::llvm::syntax::llvm_instruction_type!($ty);
        $crate::backend::llvm::syntax::llvm_constant!($kind($($constant)*))
            .map(|constant| $crate::backend::llvm::syntax::TypedConstant::new(ty, constant))
    }};
    (typed($ty:tt, $kind:ident { $($constant:tt)* })) => {{
        let ty = $crate::backend::llvm::syntax::llvm_instruction_type!($ty);
        $crate::backend::llvm::syntax::llvm_constant!($kind { $($constant)* })
            .map(|constant| $crate::backend::llvm::syntax::TypedConstant::new(ty, constant))
    }};
    (typed($ty:tt, {{ $($constant:tt)* }})) => {{
        let ty = $crate::backend::llvm::syntax::llvm_instruction_type!($ty);
        Some({ $($constant)* })
            .map(|constant| $crate::backend::llvm::syntax::TypedConstant::new(ty, constant))
    }};
}
macro_rules! llvm_typed_constants_item {
    ($constants:ident; ...{{ $($rust:tt)* }}) => {
        $constants.extend({ $($rust)* })
    };
    ($constants:ident; {{ $($rust:tt)* }}) => {
        $constants.push({ $($rust)* })
    };
    ($constants:ident; typed($($argument:tt)*)) => {
        $constants.push($crate::backend::llvm::syntax::llvm_typed_constant!(typed($($argument)*))?)
    };
}
macro_rules! llvm_typed_constants {
    (@new $($constant:tt)*) => {{
        #[allow(clippy::redundant_closure_call)]
        (|| {
            #[allow(unused_mut)]
            let mut constants = Vec::from([]);
            $crate::backend::llvm::syntax::llvm_typed_constants_items!(constants; $($constant)*);
            Some(constants)
        })()
    }};
}
macro_rules! llvm_typed_constants_items {
    ($constants:ident;) => {};
    ($constants:ident; ...{{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_typed_constants_item!($constants; ...{{ $($rust)* }});
        $crate::backend::llvm::syntax::llvm_typed_constants_items!($constants; $($($rest)*)?);
    };
    ($constants:ident; {{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_typed_constants_item!($constants; {{ $($rust)* }});
        $crate::backend::llvm::syntax::llvm_typed_constants_items!($constants; $($($rest)*)?);
    };
    ($constants:ident; typed($($argument:tt)*) $(, $($rest:tt)*)?) => {
        $crate::backend::llvm::syntax::llvm_typed_constants_item!($constants; typed($($argument)*));
        $crate::backend::llvm::syntax::llvm_typed_constants_items!($constants; $($($rest)*)?);
    };
}

macro_rules! llvm_constant {
    ({{ $($rust:tt)* }}) => {
        Some({ $($rust)* })
    };
    (atom($value:tt)) => {
        $crate::backend::llvm::syntax::Constant::atom(
            $crate::backend::llvm::syntax::llvm_instruction_atom!($value).to_string(),
        )
    };
    (structure([$($field:tt)*])) => {
        $crate::backend::llvm::syntax::llvm_typed_constants!(@new $($field)*)
            .map($crate::backend::llvm::syntax::Constant::structure)
    };
    (get_element_ptr {
        element_type: $element_type:tt,
        pointer: $pointer_kind:ident($($pointer:tt)*),
        indices: [$($index:tt)*] $(,)?
    }) => {{
        $crate::backend::llvm::syntax::llvm_typed_constant!($pointer_kind($($pointer)*)).and_then(
            |pointer| $crate::backend::llvm::syntax::llvm_typed_constants!(@new $($index)*)
                .map(|indices| $crate::backend::llvm::syntax::Constant::get_element_ptr(
                    $crate::backend::llvm::syntax::llvm_instruction_type!($element_type),
                    pointer,
                    indices,
                )),
        )
    }};
    (get_element_ptr {
        element_type: $element_type:tt,
        pointer: {{ $($pointer:tt)* }},
        indices: [$($index:tt)*] $(,)?
    }) => {{
        Some({ $($pointer)* }).and_then(|pointer| {
            $crate::backend::llvm::syntax::llvm_typed_constants!(@new $($index)*)
                .map(|indices| $crate::backend::llvm::syntax::Constant::get_element_ptr(
                    $crate::backend::llvm::syntax::llvm_instruction_type!($element_type),
                    pointer,
                    indices,
                ))
        })
    }};
    (unary {
        operator: $operator:tt,
        operand: $operand_kind:ident($($operand:tt)*) $(,)?
    }) => {
        $crate::backend::llvm::syntax::llvm_typed_constant!($operand_kind($($operand)*)).map(
            |operand| $crate::backend::llvm::syntax::Constant::unary(
                $crate::backend::llvm::syntax::llvm_instruction_atom!($operator),
                operand,
            ),
        )
    };
    (unary { operator: $operator:tt, operand: {{ $($operand:tt)* }} $(,)? }) => {
        Some({ $($operand)* }).map(|operand| $crate::backend::llvm::syntax::Constant::unary(
            $crate::backend::llvm::syntax::llvm_instruction_atom!($operator), operand,
        ))
    };
    (binary {
        operator: $operator:tt,
        left: $left_kind:ident($($left:tt)*),
        right: $right_kind:ident($($right:tt)*) $(,)?
    }) => {{
        let left = $crate::backend::llvm::syntax::llvm_typed_constant!($left_kind($($left)*));
        let right = $crate::backend::llvm::syntax::llvm_typed_constant!($right_kind($($right)*));
        left.zip(right).and_then(|(left, right)| {
            $crate::backend::llvm::syntax::Constant::binary(
                $crate::backend::llvm::syntax::llvm_instruction_atom!($operator), left, right,
            )
        })
    }};
    (binary { operator: $operator:tt, left: {{ $($left:tt)* }}, right: {{ $($right:tt)* }} $(,)? }) => {
        Some(({ $($left)* }, { $($right)* })).and_then(|(left, right)| {
            $crate::backend::llvm::syntax::Constant::binary(
                $crate::backend::llvm::syntax::llvm_instruction_atom!($operator), left, right,
            )
        })
    };
    (cast {
        operator: $operator:tt,
        operand: {{ $($operand:tt)* }},
        to: $target:tt $(,)?
    }) => {
        Some({ $($operand)* }).map(|operand| {
            $crate::backend::llvm::syntax::Constant::cast(
                $crate::backend::llvm::syntax::llvm_instruction_atom!($operator), operand,
                $crate::backend::llvm::syntax::llvm_instruction_type!($target),
            )
        })
    };
    (zero) => {
        Some($crate::backend::llvm::syntax::Constant::ZeroInitializer)
    };
}

pub(in crate::backend) use {
    llvm_constant, llvm_typed_constant, llvm_typed_constants, llvm_typed_constants_item,
    llvm_typed_constants_items,
};
