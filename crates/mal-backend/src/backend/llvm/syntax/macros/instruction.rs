macro_rules! llvm_instruction_atom {
    ($value:tt) => {
        $crate::backend::llvm::syntax::llvm_scalar!($value)
    };
}

macro_rules! llvm_instruction_type {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (($($ty:tt)*)) => {
        $crate::backend::llvm::syntax::llvm_type!($($ty)*)
    };
}

macro_rules! llvm_instruction_sequence {
    ([]) => { [] };
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
}

macro_rules! llvm_instruction_indices {
    ([$($index:tt),* $(,)?]) => {
        [$(
            $crate::backend::llvm::syntax::llvm_instruction_atom!($index)
        ),*]
    };
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
}

macro_rules! llvm_instruction {
    ({ $($rust:tt)* }) => {
        Some({ $($rust)* })
    };
    (call None, $($rest:tt)*) => {
        $crate::backend::llvm::syntax::llvm_instruction!(
            call { Option::<String>::None }, $($rest)*
        )
    };
    (alloca $result:tt, $ty:tt, $alignment:tt $(,)?) => {
        $crate::backend::llvm::syntax::Instruction::alloca(
            $crate::backend::llvm::syntax::llvm_instruction_atom!($result),
            $crate::backend::llvm::syntax::llvm_instruction_type!($ty),
            $crate::backend::llvm::syntax::llvm_instruction_atom!($alignment),
        )
    };
    (load $result:tt, $ty:tt, $pointer:tt, $alignment:tt, $metadata:tt $(,)?) => {
        $crate::backend::llvm::syntax::Instruction::load(
            $crate::backend::llvm::syntax::llvm_instruction_atom!($result),
            $crate::backend::llvm::syntax::llvm_instruction_type!($ty),
            $crate::backend::llvm::syntax::llvm_instruction_atom!($pointer),
            $crate::backend::llvm::syntax::llvm_instruction_atom!($alignment),
            $crate::backend::llvm::syntax::llvm_instruction_sequence!($metadata),
        )
    };
    (store $ty:tt, $value:tt, $pointer:tt, $alignment:tt, $metadata:tt $(,)?) => {
        $crate::backend::llvm::syntax::Instruction::store(
            $crate::backend::llvm::syntax::llvm_instruction_type!($ty),
            $crate::backend::llvm::syntax::llvm_instruction_atom!($value),
            $crate::backend::llvm::syntax::llvm_instruction_atom!($pointer),
            $crate::backend::llvm::syntax::llvm_instruction_atom!($alignment),
            $crate::backend::llvm::syntax::llvm_instruction_sequence!($metadata),
        )
    };
    (call $result:tt, $tail:tt, $result_type:tt, direct $callee:tt; [$($argument:tt),* $(,)?]) => {{
        let result: Option<String> =
            $crate::backend::llvm::syntax::llvm_instruction_atom!($result);
        $crate::backend::llvm::syntax::Callee::direct(
            $crate::backend::llvm::syntax::llvm_instruction_atom!($callee)
        ).and_then(|callee| {
            $crate::backend::llvm::syntax::llvm_values!($($argument),*).and_then(
                |arguments| {
                    $crate::backend::llvm::syntax::Instruction::call(
                        result,
                        $crate::backend::llvm::syntax::llvm_instruction_atom!($tail),
                        $crate::backend::llvm::syntax::llvm_instruction_type!($result_type),
                        callee,
                        arguments,
                    )
                },
            )
        })
    }};
    (call $result:tt, $tail:tt, $result_type:tt, direct $callee:tt, $arguments:tt) => {{
        let arguments: Vec<($crate::backend::llvm::syntax::Type, String)> =
            $crate::backend::llvm::syntax::llvm_instruction_sequence!($arguments)
                .into_iter()
                .collect();
        arguments
            .into_iter()
            .map(|(ty, value)| $crate::backend::llvm::syntax::TypedValue::new(ty, value))
            .collect::<Option<Vec<_>>>()
            .and_then(|arguments| $crate::backend::llvm::syntax::llvm_instruction!(
                call $result, $tail, $result_type, direct $callee; [{{ arguments }}]
            ))
    }};
    (call $result:tt, $tail:tt, $result_type:tt, indirect $callee:tt; [$($argument:tt),* $(,)?]) => {{
        let result: Option<String> =
            $crate::backend::llvm::syntax::llvm_instruction_atom!($result);
        $crate::backend::llvm::syntax::Callee::indirect(
            $crate::backend::llvm::syntax::llvm_instruction_atom!($callee)
        ).and_then(|callee| {
            $crate::backend::llvm::syntax::llvm_values!($($argument),*).and_then(
                |arguments| {
                    $crate::backend::llvm::syntax::Instruction::call(
                        result,
                        $crate::backend::llvm::syntax::llvm_instruction_atom!($tail),
                        $crate::backend::llvm::syntax::llvm_instruction_type!($result_type),
                        callee,
                        arguments,
                    )
                },
            )
        })
    }};
    (call $result:tt, $tail:tt, $result_type:tt, indirect $callee:tt, $arguments:tt) => {{
        let arguments: Vec<($crate::backend::llvm::syntax::Type, String)> =
            $crate::backend::llvm::syntax::llvm_instruction_sequence!($arguments)
                .into_iter()
                .collect();
        arguments
            .into_iter()
            .map(|(ty, value)| $crate::backend::llvm::syntax::TypedValue::new(ty, value))
            .collect::<Option<Vec<_>>>()
            .and_then(|arguments| $crate::backend::llvm::syntax::llvm_instruction!(
                call $result, $tail, $result_type, indirect $callee; [{{ arguments }}]
            ))
    }};
    (unary $result:tt, $operator:tt; $ty:tt => $value:tt $(,)?) => {{
        $crate::backend::llvm::syntax::llvm_value!(typed $ty => $value).and_then(|value| {
            $crate::backend::llvm::syntax::Instruction::unary(
                $crate::backend::llvm::syntax::llvm_instruction_atom!($result),
                $crate::backend::llvm::syntax::llvm_instruction_atom!($operator),
                value,
            )
        })
    }};
    (cast $result:tt, $operator:tt; $ty:tt => $value:tt, $target:tt $(,)?) => {{
        $crate::backend::llvm::syntax::llvm_value!(typed $ty => $value).and_then(|value| {
            $crate::backend::llvm::syntax::Instruction::cast(
                $crate::backend::llvm::syntax::llvm_instruction_atom!($result),
                $crate::backend::llvm::syntax::llvm_instruction_atom!($operator),
                value,
                $crate::backend::llvm::syntax::llvm_instruction_type!($target),
            )
        })
    }};
    (extract_value $result:tt; $ty:tt => $value:tt, $indices:tt $(,)?) => {{
        $crate::backend::llvm::syntax::llvm_value!(typed $ty => $value).and_then(|value| {
            $crate::backend::llvm::syntax::Instruction::extract_value(
                $crate::backend::llvm::syntax::llvm_instruction_atom!($result),
                value,
                $crate::backend::llvm::syntax::llvm_instruction_indices!($indices),
            )
        })
    }};
    (binary $result:tt, $operator:tt, $ty:tt, $left:tt, $right:tt $(,)?) => {
        $crate::backend::llvm::syntax::Instruction::binary(
            $crate::backend::llvm::syntax::llvm_instruction_atom!($result),
            $crate::backend::llvm::syntax::llvm_instruction_atom!($operator),
            $crate::backend::llvm::syntax::llvm_instruction_type!($ty),
            $crate::backend::llvm::syntax::llvm_instruction_atom!($left),
            $crate::backend::llvm::syntax::llvm_instruction_atom!($right),
        )
    };
    (compare $result:tt, $kind:tt, $predicate:tt, $ty:tt, $left:tt, $right:tt $(,)?) => {
        $crate::backend::llvm::syntax::Instruction::compare(
            $crate::backend::llvm::syntax::llvm_instruction_atom!($result),
            $crate::backend::llvm::syntax::llvm_instruction_atom!($kind),
            $crate::backend::llvm::syntax::llvm_instruction_atom!($predicate),
            $crate::backend::llvm::syntax::llvm_instruction_type!($ty),
            $crate::backend::llvm::syntax::llvm_instruction_atom!($left),
            $crate::backend::llvm::syntax::llvm_instruction_atom!($right),
        )
    };
    (get_element_ptr $result:tt, $inbounds:tt, $element_type:tt, $pointer:tt; [$($index:tt),* $(,)?]) => {{
        $crate::backend::llvm::syntax::llvm_values!($($index),*).and_then(|indices| {
            $crate::backend::llvm::syntax::Instruction::get_element_ptr(
                $crate::backend::llvm::syntax::llvm_instruction_atom!($result),
                $crate::backend::llvm::syntax::llvm_instruction_atom!($inbounds),
                $crate::backend::llvm::syntax::llvm_instruction_type!($element_type),
                $crate::backend::llvm::syntax::llvm_instruction_atom!($pointer),
                indices,
            )
        })
    }};
    (get_element_ptr $result:tt, $inbounds:tt, $element_type:tt, $pointer:tt, $indices:tt $(,)?) => {{
        let indices: Vec<($crate::backend::llvm::syntax::Type, String)> =
            $crate::backend::llvm::syntax::llvm_instruction_sequence!($indices)
                .into_iter()
                .collect();
        indices
            .into_iter()
            .map(|(ty, value)| $crate::backend::llvm::syntax::TypedValue::new(ty, value))
            .collect::<Option<Vec<_>>>()
            .and_then(|indices| $crate::backend::llvm::syntax::llvm_instruction!(
                get_element_ptr $result, $inbounds, $element_type, $pointer; [{{ indices }}]
            ))
    }};
    (insert_value $result:tt; $aggregate_type:tt => $aggregate:tt, $element_type:tt => $element:tt, $indices:tt $(,)?) => {{
        let aggregate =
            $crate::backend::llvm::syntax::llvm_value!(typed $aggregate_type => $aggregate);
        let element = $crate::backend::llvm::syntax::llvm_value!(typed $element_type => $element);
        aggregate.zip(element).and_then(|(aggregate, element)| {
            $crate::backend::llvm::syntax::Instruction::insert_value(
                $crate::backend::llvm::syntax::llvm_instruction_atom!($result),
                aggregate,
                element,
                $crate::backend::llvm::syntax::llvm_instruction_indices!($indices),
            )
        })
    }};
    (phi $result:tt, $ty:tt, $incoming:tt $(,)?) => {
        $crate::backend::llvm::syntax::Instruction::phi(
            $crate::backend::llvm::syntax::llvm_instruction_atom!($result),
            $crate::backend::llvm::syntax::llvm_instruction_type!($ty),
            $crate::backend::llvm::syntax::llvm_instruction_sequence!($incoming),
        )
    };
}

pub(in crate::backend) use {
    llvm_instruction, llvm_instruction_atom, llvm_instruction_indices, llvm_instruction_sequence,
    llvm_instruction_type,
};
