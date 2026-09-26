mod constant;
mod function;
mod instruction;
mod module;
mod ty;

macro_rules! llvm_instruction {
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
    (call $result:expr, $tail:expr, $result_type:expr, direct $callee:expr, $arguments:expr) => {{
        $crate::backend::llvm::syntax::Callee::direct($callee).and_then(|callee| {
            $arguments
                .into_iter()
                .map(|(ty, value)| $crate::backend::llvm::syntax::TypedValue::new(ty, value))
                .collect::<Option<Vec<_>>>()
                .and_then(|arguments| {
                $crate::backend::llvm::syntax::Instruction::call(
                    $result,
                    $tail,
                    $result_type,
                    callee,
                    arguments,
                )
            })
        })
    }};
    (call $result:expr, $tail:expr, $result_type:expr, indirect $callee:expr, $arguments:expr) => {{
        $crate::backend::llvm::syntax::Callee::indirect($callee).and_then(|callee| {
            $arguments
                .into_iter()
                .map(|(ty, value)| $crate::backend::llvm::syntax::TypedValue::new(ty, value))
                .collect::<Option<Vec<_>>>()
                .and_then(|arguments| {
                $crate::backend::llvm::syntax::Instruction::call(
                    $result,
                    $tail,
                    $result_type,
                    callee,
                    arguments,
                )
            })
        })
    }};
    (typed $constructor:ident($($leading:expr),*); $ty:expr => $value:expr $(, $trailing:expr)* $(,)? ) => {{
        $crate::backend::llvm::syntax::TypedValue::new($ty, $value).and_then(|value| {
            $crate::backend::llvm::syntax::Instruction::$constructor(
                $($leading,)* value $(, $trailing)*
            )
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
    (get_element_ptr $result:expr, $inbounds:expr, $element_type:expr, $pointer:expr, $indices:expr $(,)?) => {{
        $indices
            .into_iter()
            .map(|(ty, value)| $crate::backend::llvm::syntax::TypedValue::new(ty, value))
            .collect::<Option<Vec<_>>>()
            .and_then(|indices| {
                $crate::backend::llvm::syntax::Instruction::get_element_ptr(
                    $result,
                    $inbounds,
                    $element_type,
                    $pointer,
                    indices,
                )
            })
    }};
    (insert_value $result:expr; $aggregate_type:expr => $aggregate:expr, $element_type:expr => $element:expr, $indices:expr $(,)?) => {{
        let aggregate = $crate::backend::llvm::syntax::TypedValue::new(
            $aggregate_type,
            $aggregate,
        );
        let element = $crate::backend::llvm::syntax::TypedValue::new($element_type, $element);
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

pub(super) use llvm_instruction;

pub(super) use constant::{Constant, TypedConstant};
#[cfg(test)]
pub(super) use function::BasicBlock;
pub(super) use function::{
    FunctionAttribute, FunctionBuilder, FunctionDefinition, Linkage, ParameterAttribute, Terminator,
};
pub(in crate::backend) use function::{FunctionSignature, Parameter};
pub(super) use instruction::MetadataAttachment;
pub(super) use instruction::{
    BinaryOperator, CastOperator, ComparisonKind, ComparisonPredicate, UnaryOperator,
};
pub(in crate::backend) use instruction::{Callee, Instruction, TypedValue};
pub(in crate::backend) use module::FunctionDeclaration;
pub(super) use module::Module;
pub(super) use module::{GlobalDefinition, MetadataDefinition, MetadataOperand};
pub(in crate::backend) use ty::Type;
