macro_rules! llvm_instruction_atom_normalized {
    ($value:tt) => {
        $crate::backend::llvm::syntax::llvm_scalar_normalized!($value)
    };
}

macro_rules! llvm_instruction_type_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (($($ty:tt)*)) => {
        $crate::backend::llvm::syntax::llvm_type_normalized!($($ty)*)
    };
}

macro_rules! llvm_instruction_sequence_normalized {
    ([]) => { [] };
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
}

macro_rules! llvm_instruction_indices_normalized {
    ([$($index:tt),* $(,)?]) => {
        [$(
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($index)
        ),*]
    };
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
}

macro_rules! llvm_instruction_callee_normalized {
    (direct($callee:tt)) => {
        $crate::backend::llvm::syntax::Callee::direct(
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($callee),
        )
    };
    (indirect($callee:tt)) => {
        $crate::backend::llvm::syntax::Callee::indirect(
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($callee),
        )
    };
}

macro_rules! llvm_instruction_normalized {
    ((@rust $($rust:tt)*)) => {
        Some({ $($rust)* })
    };
    (let $result:tt = alloca {
        ty: $ty:tt,
        alignment: $alignment:tt $(,)?
    };) => {
        $crate::backend::llvm::syntax::Instruction::alloca(
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($result),
            $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($ty),
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($alignment),
        )
    };
    (let $result:tt = load {
        ty: $ty:tt,
        pointer: $pointer:tt,
        alignment: $alignment:tt,
        metadata: $metadata:tt $(,)?
    };) => {
        $crate::backend::llvm::syntax::Instruction::load(
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($result),
            $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($ty),
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($pointer),
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($alignment),
            $crate::backend::llvm::syntax::llvm_instruction_sequence_normalized!($metadata),
        )
    };
    (store {
        value: typed($ty:tt, $value:tt $(,)?),
        pointer: $pointer:tt,
        alignment: $alignment:tt,
        metadata: $metadata:tt $(,)?
    };) => {
        $crate::backend::llvm::syntax::Instruction::store(
            $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($ty),
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($value),
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($pointer),
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($alignment),
            $crate::backend::llvm::syntax::llvm_instruction_sequence_normalized!($metadata),
        )
    };
    (let $result:tt = call {
        tail: $tail:tt,
        result_type: $result_type:tt,
        callee: $callee_kind:ident($callee:tt),
        arguments: [$($argument:tt)*] $(,)?
    };) => {{
        $crate::backend::llvm::syntax::llvm_instruction_callee_normalized!($callee_kind($callee)).and_then(
            |callee| {
                $crate::backend::llvm::syntax::llvm_values_normalized!($($argument)*).and_then(
                    |arguments| $crate::backend::llvm::syntax::Instruction::call(
                        Some($crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($result)),
                        $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($tail),
                        $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($result_type),
                        callee,
                        arguments,
                    ),
                )
            },
        )
    }};
    (call {
        tail: $tail:tt,
        result_type: $result_type:tt,
        callee: $callee_kind:ident($callee:tt),
        arguments: [$($argument:tt)*] $(,)?
    };) => {{
        $crate::backend::llvm::syntax::llvm_instruction_callee_normalized!($callee_kind($callee)).and_then(
            |callee| {
                $crate::backend::llvm::syntax::llvm_values_normalized!($($argument)*).and_then(
                    |arguments| $crate::backend::llvm::syntax::Instruction::call(
                        Option::<String>::None,
                        $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($tail),
                        $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($result_type),
                        callee,
                        arguments,
                    ),
                )
            },
        )
    }};
    (let $result:tt = unary {
        operator: $operator:tt,
        value: typed($ty:tt, $value:tt $(,)?) $(,)?
    };) => {
        $crate::backend::llvm::syntax::llvm_value_normalized!(typed($ty, $value)).and_then(|value| {
            $crate::backend::llvm::syntax::Instruction::unary(
                $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($result),
                $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($operator),
                value,
            )
        })
    };
    (let $result:tt = cast {
        operator: $operator:tt,
        value: typed($ty:tt, $value:tt $(,)?),
        to: $target:tt $(,)?
    };) => {
        $crate::backend::llvm::syntax::llvm_value_normalized!(typed($ty, $value)).and_then(|value| {
            $crate::backend::llvm::syntax::Instruction::cast(
                $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($result),
                $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($operator),
                value,
                $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($target),
            )
        })
    };
    (let $result:tt = extract_value {
        aggregate: typed($ty:tt, $value:tt $(,)?),
        indices: $indices:tt $(,)?
    };) => {
        $crate::backend::llvm::syntax::llvm_value_normalized!(typed($ty, $value)).and_then(|value| {
            $crate::backend::llvm::syntax::Instruction::extract_value(
                $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($result),
                value,
                $crate::backend::llvm::syntax::llvm_instruction_indices_normalized!($indices),
            )
        })
    };
    (let $result:tt = binary {
        operator: $operator:tt,
        ty: $ty:tt,
        left: $left:tt,
        right: $right:tt $(,)?
    };) => {
        $crate::backend::llvm::syntax::Instruction::binary(
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($result),
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($operator),
            $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($ty),
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($left),
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($right),
        )
    };
    (let $result:tt = compare {
        kind: $kind:tt,
        predicate: $predicate:tt,
        ty: $ty:tt,
        left: $left:tt,
        right: $right:tt $(,)?
    };) => {
        $crate::backend::llvm::syntax::Instruction::compare(
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($result),
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($kind),
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($predicate),
            $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($ty),
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($left),
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($right),
        )
    };
    (let $result:tt = get_element_ptr {
        inbounds: $inbounds:tt,
        element_type: $element_type:tt,
        pointer: $pointer:tt,
        indices: [$($index:tt)*] $(,)?
    };) => {
        $crate::backend::llvm::syntax::llvm_values_normalized!($($index)*).and_then(|indices| {
            $crate::backend::llvm::syntax::Instruction::get_element_ptr(
                $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($result),
                $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($inbounds),
                $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($element_type),
                $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($pointer),
                indices,
            )
        })
    };
    (let $result:tt = insert_value {
        aggregate: typed($aggregate_type:tt, $aggregate:tt $(,)?),
        element: typed($element_type:tt, $element:tt $(,)?),
        indices: $indices:tt $(,)?
    };) => {{
        let aggregate = $crate::backend::llvm::syntax::llvm_value_normalized!(typed($aggregate_type, $aggregate));
        let element = $crate::backend::llvm::syntax::llvm_value_normalized!(typed($element_type, $element));
        aggregate.zip(element).and_then(|(aggregate, element)| {
            $crate::backend::llvm::syntax::Instruction::insert_value(
                $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($result),
                aggregate,
                element,
                $crate::backend::llvm::syntax::llvm_instruction_indices_normalized!($indices),
            )
        })
    }};
    (let $result:tt = phi {
        ty: $ty:tt,
        incoming: $incoming:tt $(,)?
    };) => {
        $crate::backend::llvm::syntax::Instruction::phi(
            $crate::backend::llvm::syntax::llvm_instruction_atom_normalized!($result),
            $crate::backend::llvm::syntax::llvm_instruction_type_normalized!($ty),
            $crate::backend::llvm::syntax::llvm_instruction_sequence_normalized!($incoming),
        )
    };
}

macro_rules! llvm_instruction {
    (@normalized $($syntax:tt)*) => {
        $crate::backend::llvm::syntax::llvm_instruction_normalized!($($syntax)*)
    };
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::llvm::syntax::llvm_instruction]; $($syntax)*
        )
    };
}

pub(in crate::backend) use {
    llvm_instruction_atom_normalized, llvm_instruction_callee_normalized,
    llvm_instruction_indices_normalized, llvm_instruction_normalized,
    llvm_instruction_sequence_normalized, llvm_instruction_type_normalized,
};

pub(in crate::backend) use llvm_instruction;
