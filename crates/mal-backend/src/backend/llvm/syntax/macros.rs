mod constant;
mod terminator;

pub(in crate::backend::llvm) use constant::{
    llvm_constant, llvm_typed_constant, llvm_typed_constants, llvm_typed_constants_item,
};
pub(in crate::backend::llvm) use terminator::{
    llvm_switch_cases, llvm_switch_cases_item, llvm_terminator,
};

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
pub(in crate::backend::llvm) use {llvm_value, llvm_values, llvm_values_item};

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::backend::llvm::syntax::{
        BinaryOperator, FunctionBuilder, FunctionSignature, Type, TypedValue,
    };

    #[test]
    fn composes_static_embedded_and_runtime_typed_values_in_order() {
        let evaluations = Cell::new(0);
        let dynamic = || {
            evaluations.set(evaluations.get() + 1);
            TypedValue::new(Type::Pointer, "%dynamic").unwrap()
        };
        let pair_evaluations = Cell::new(0);
        let pairs = || {
            pair_evaluations.set(pair_evaluations.get() + 1);
            [(Type::integer(16_u16), "2")]
        };
        let trailing = [TypedValue::new(Type::integer(8_u16), "7").unwrap()];
        let instruction = super::llvm_instruction!(
            call Some("%result"), false, Type::integer(32_u16), direct "work"; [
                (typed Type::integer(32_u16) => "1"),
                (rust dynamic()),
                (typed_extend pairs()),
                (extend trailing),
            ]
        )
        .unwrap();
        let mut rendered = String::new();
        instruction.render_into(&mut rendered);

        assert_eq!(evaluations.get(), 1);
        assert_eq!(pair_evaluations.get(), 1);
        assert_eq!(
            rendered,
            "%result = call i32 @work(i32 1, ptr %dynamic, i16 2, i8 7)"
        );
    }

    #[test]
    fn composes_nested_constants_and_dynamic_fields() {
        let trailing =
            [super::llvm_typed_constant!(typed Type::integer(8_u16) => (atom 7)).unwrap()];
        let constant = super::llvm_constant!(structure [
            (typed Type::integer(32_u16) => (binary BinaryOperator::Add;
                (typed Type::integer(32_u16) => (atom 1));
                (typed Type::integer(32_u16) => (atom 2))
            )),
            (extend trailing),
        ])
        .unwrap();

        assert_eq!(constant.render(), "{ i32 add (i32 1, i32 2), i8 7 }");
    }

    #[test]
    fn composes_switch_cases_and_finishes_the_function() {
        let trailing = [("1".to_string(), "one".to_string())];
        let mut function = FunctionBuilder::new(FunctionSignature::new(Type::Void, "choose", []));
        assert!(function.start_block("entry"));
        assert!(
            function.terminate(
                super::llvm_terminator!(switch Type::integer(8_u16) => "%tag";
                    default "other";
                    [
                        (case 0 => "zero"),
                        (extend trailing),
                    ]
                )
                .unwrap()
            )
        );
        for block in ["zero", "one", "other"] {
            assert!(function.start_block(block));
            assert!(function.terminate(super::llvm_terminator!(return_void).unwrap()));
        }

        assert!(function.finish().is_some());
    }
}
