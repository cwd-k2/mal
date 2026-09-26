mod constant;
mod function;
mod instruction;
mod module;
mod ty;

macro_rules! llvm_instruction {
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
