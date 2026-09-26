macro_rules! llvm_instruction {
    (rust $instruction:expr) => {
        Some($instruction)
    };
    (alloca $result:expr, $ty:expr, $alignment:expr $(,)?) => {
        $crate::backend::llvm::syntax::Instruction::alloca($result, $ty, $alignment)
    };
    (load $result:expr, $ty:expr, $pointer:expr, $alignment:expr, $metadata:expr $(,)?) => {
        $crate::backend::llvm::syntax::Instruction::load(
            $result, $ty, $pointer, $alignment, $metadata,
        )
    };
    (store $ty:expr, $value:expr, $pointer:expr, $alignment:expr, $metadata:expr $(,)?) => {
        $crate::backend::llvm::syntax::Instruction::store(
            $ty, $value, $pointer, $alignment, $metadata,
        )
    };
    (call $result:expr, $tail:expr, $result_type:expr, direct $callee:expr; [$($argument:tt),* $(,)?]) => {{
        $crate::backend::llvm::syntax::Callee::direct($callee).and_then(|callee| {
            $crate::backend::llvm::syntax::llvm_values!($($argument),*).and_then(
                |arguments| {
                    $crate::backend::llvm::syntax::Instruction::call(
                        $result,
                        $tail,
                        $result_type,
                        callee,
                        arguments,
                    )
                },
            )
        })
    }};
    (call $result:expr, $tail:expr, $result_type:expr, direct $callee:expr, $arguments:expr) => {
        $crate::backend::llvm::syntax::llvm_instruction!(
            call $result, $tail, $result_type, direct $callee; [(typed_extend $arguments)]
        )
    };
    (call $result:expr, $tail:expr, $result_type:expr, indirect $callee:expr; [$($argument:tt),* $(,)?]) => {{
        $crate::backend::llvm::syntax::Callee::indirect($callee).and_then(|callee| {
            $crate::backend::llvm::syntax::llvm_values!($($argument),*).and_then(
                |arguments| {
                    $crate::backend::llvm::syntax::Instruction::call(
                        $result,
                        $tail,
                        $result_type,
                        callee,
                        arguments,
                    )
                },
            )
        })
    }};
    (call $result:expr, $tail:expr, $result_type:expr, indirect $callee:expr, $arguments:expr) => {
        $crate::backend::llvm::syntax::llvm_instruction!(
            call $result, $tail, $result_type, indirect $callee; [(typed_extend $arguments)]
        )
    };
    (unary $result:expr, $operator:expr; $ty:expr => $value:expr $(,)?) => {{
        $crate::backend::llvm::syntax::llvm_value!(typed $ty => $value).and_then(|value| {
            $crate::backend::llvm::syntax::Instruction::unary($result, $operator, value)
        })
    }};
    (cast $result:expr, $operator:expr; $ty:expr => $value:expr, $target:expr $(,)?) => {{
        $crate::backend::llvm::syntax::llvm_value!(typed $ty => $value).and_then(|value| {
            $crate::backend::llvm::syntax::Instruction::cast($result, $operator, value, $target)
        })
    }};
    (extract_value $result:expr; $ty:expr => $value:expr, $indices:expr $(,)?) => {{
        $crate::backend::llvm::syntax::llvm_value!(typed $ty => $value).and_then(|value| {
            $crate::backend::llvm::syntax::Instruction::extract_value($result, value, $indices)
        })
    }};
    (binary $result:expr, $operator:expr, $ty:expr, $left:expr, $right:expr $(,)?) => {
        $crate::backend::llvm::syntax::Instruction::binary(
            $result, $operator, $ty, $left, $right,
        )
    };
    (compare $result:expr, $kind:expr, $predicate:expr, $ty:expr, $left:expr, $right:expr $(,)?) => {
        $crate::backend::llvm::syntax::Instruction::compare(
            $result, $kind, $predicate, $ty, $left, $right,
        )
    };
    (get_element_ptr $result:expr, $inbounds:expr, $element_type:expr, $pointer:expr; [$($index:tt),* $(,)?]) => {{
        $crate::backend::llvm::syntax::llvm_values!($($index),*).and_then(|indices| {
            $crate::backend::llvm::syntax::Instruction::get_element_ptr(
                $result,
                $inbounds,
                $element_type,
                $pointer,
                indices,
            )
        })
    }};
    (get_element_ptr $result:expr, $inbounds:expr, $element_type:expr, $pointer:expr, $indices:expr $(,)?) => {
        $crate::backend::llvm::syntax::llvm_instruction!(
            get_element_ptr $result, $inbounds, $element_type, $pointer; [(typed_extend $indices)]
        )
    };
    (insert_value $result:expr; $aggregate_type:expr => $aggregate:expr, $element_type:expr => $element:expr, $indices:expr $(,)?) => {{
        let aggregate =
            $crate::backend::llvm::syntax::llvm_value!(typed $aggregate_type => $aggregate);
        let element = $crate::backend::llvm::syntax::llvm_value!(typed $element_type => $element);
        aggregate.zip(element).and_then(|(aggregate, element)| {
            $crate::backend::llvm::syntax::Instruction::insert_value(
                $result, aggregate, element, $indices,
            )
        })
    }};
    (phi $result:expr, $ty:expr, $incoming:expr $(,)?) => {
        $crate::backend::llvm::syntax::Instruction::phi($result, $ty, $incoming)
    };
}

pub(in crate::backend::llvm) use llvm_instruction;
