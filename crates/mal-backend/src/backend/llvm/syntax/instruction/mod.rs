//! Typed LLVM instructions, admitted field by field when they are built.

use super::Type;
use super::function::{is_single_line, is_valid_name};

mod operator;
mod render;
#[cfg(test)]
mod tests;
mod value;

use operator::comparison_is_valid;
pub(in crate::backend) use operator::{
    BinaryOperator, CastOperator, ComparisonKind, ComparisonPredicate, UnaryOperator,
};
use value::local_name;
pub(in crate::backend) use value::{Callee, TypedValue};
pub(in crate::backend::llvm::syntax) use value::{is_atom, is_value};

#[derive(Clone)]
pub(in crate::backend) enum Instruction {
    Alloca {
        result: String,
        ty: Type,
        alignment: usize,
    },
    Load {
        result: String,
        ty: Type,
        pointer: String,
        alignment: usize,
        metadata: Vec<MetadataAttachment>,
    },
    Store {
        ty: Type,
        value: String,
        pointer: String,
        alignment: usize,
        metadata: Vec<MetadataAttachment>,
    },
    Call {
        result: Option<String>,
        tail: bool,
        result_type: Type,
        callee: Callee,
        arguments: Vec<TypedValue>,
    },
    Unary {
        result: String,
        operator: UnaryOperator,
        operand: TypedValue,
    },
    Binary {
        result: String,
        operator: BinaryOperator,
        ty: Type,
        left: String,
        right: String,
    },
    Compare {
        result: String,
        kind: ComparisonKind,
        predicate: ComparisonPredicate,
        ty: Type,
        left: String,
        right: String,
    },
    Cast {
        result: String,
        operator: CastOperator,
        operand: TypedValue,
        target: Type,
    },
    GetElementPtr {
        result: String,
        inbounds: bool,
        element_type: Type,
        pointer: String,
        indices: Vec<TypedValue>,
    },
    ExtractValue {
        result: String,
        aggregate: TypedValue,
        indices: Vec<usize>,
    },
    InsertValue {
        result: String,
        aggregate: TypedValue,
        element: TypedValue,
        indices: Vec<usize>,
    },
    Phi {
        result: String,
        ty: Type,
        incoming: Vec<(String, String)>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::backend) enum MetadataAttachment {
    Tbaa(usize),
    AliasScope(usize),
    NoAlias(usize),
}

impl Instruction {
    pub(in crate::backend::llvm) fn alloca(
        result: impl Into<String>,
        ty: Type,
        alignment: usize,
    ) -> Option<Self> {
        let result = result.into();
        (local_name(&result).is_some_and(is_valid_name) && alignment.is_power_of_two()).then_some(
            Self::Alloca {
                result,
                ty,
                alignment,
            },
        )
    }

    pub(in crate::backend::llvm) fn load(
        result: impl Into<String>,
        ty: Type,
        pointer: impl Into<String>,
        alignment: usize,
        metadata: impl IntoIterator<Item = MetadataAttachment>,
    ) -> Option<Self> {
        let result = result.into();
        let pointer = pointer.into();
        (local_name(&result).is_some_and(is_valid_name)
            && is_value(&pointer)
            && alignment.is_power_of_two())
        .then_some(Self::Load {
            result,
            ty,
            pointer,
            alignment,
            metadata: metadata.into_iter().collect(),
        })
    }

    pub(in crate::backend::llvm) fn store(
        ty: Type,
        value: impl Into<String>,
        pointer: impl Into<String>,
        alignment: usize,
        metadata: impl IntoIterator<Item = MetadataAttachment>,
    ) -> Option<Self> {
        let value = value.into();
        let pointer = pointer.into();
        (is_value(&value) && is_value(&pointer) && alignment.is_power_of_two()).then_some(
            Self::Store {
                ty,
                value,
                pointer,
                alignment,
                metadata: metadata.into_iter().collect(),
            },
        )
    }

    pub(in crate::backend) fn call(
        result: Option<impl Into<String>>,
        tail: bool,
        result_type: Type,
        callee: Callee,
        arguments: impl IntoIterator<Item = TypedValue>,
    ) -> Option<Self> {
        let result = result.map(Into::into);
        (result
            .as_deref()
            .is_none_or(|result| local_name(result).is_some_and(is_valid_name))
            && !(result.is_some() && matches!(result_type, Type::Void)))
        .then_some(Self::Call {
            result,
            tail,
            result_type,
            callee,
            arguments: arguments.into_iter().collect(),
        })
    }

    pub(in crate::backend::llvm) fn unary(
        result: impl Into<String>,
        operator: UnaryOperator,
        operand: TypedValue,
    ) -> Option<Self> {
        let result = result.into();
        local_name(&result)
            .is_some_and(is_valid_name)
            .then_some(Self::Unary {
                result,
                operator,
                operand,
            })
    }

    pub(in crate::backend::llvm) fn binary(
        result: impl Into<String>,
        operator: BinaryOperator,
        ty: Type,
        left: impl Into<String>,
        right: impl Into<String>,
    ) -> Option<Self> {
        let result = result.into();
        let left = left.into();
        let right = right.into();
        (local_name(&result).is_some_and(is_valid_name) && is_value(&left) && is_value(&right))
            .then_some(Self::Binary {
                result,
                operator,
                ty,
                left,
                right,
            })
    }

    pub(in crate::backend::llvm) fn compare(
        result: impl Into<String>,
        kind: ComparisonKind,
        predicate: ComparisonPredicate,
        ty: Type,
        left: impl Into<String>,
        right: impl Into<String>,
    ) -> Option<Self> {
        let result = result.into();
        let left = left.into();
        let right = right.into();
        (local_name(&result).is_some_and(is_valid_name)
            && is_value(&left)
            && is_value(&right)
            && comparison_is_valid(kind, predicate))
        .then_some(Self::Compare {
            result,
            kind,
            predicate,
            ty,
            left,
            right,
        })
    }

    pub(in crate::backend::llvm) fn cast(
        result: impl Into<String>,
        operator: CastOperator,
        operand: TypedValue,
        target: Type,
    ) -> Option<Self> {
        let result = result.into();
        local_name(&result)
            .is_some_and(is_valid_name)
            .then_some(Self::Cast {
                result,
                operator,
                operand,
                target,
            })
    }

    pub(in crate::backend::llvm) fn get_element_ptr(
        result: impl Into<String>,
        inbounds: bool,
        element_type: Type,
        pointer: impl Into<String>,
        indices: impl IntoIterator<Item = TypedValue>,
    ) -> Option<Self> {
        let result = result.into();
        let pointer = pointer.into();
        (local_name(&result).is_some_and(is_valid_name) && is_value(&pointer)).then_some(
            Self::GetElementPtr {
                result,
                inbounds,
                element_type,
                pointer,
                indices: indices.into_iter().collect(),
            },
        )
    }

    pub(in crate::backend::llvm) fn extract_value(
        result: impl Into<String>,
        aggregate: TypedValue,
        indices: impl IntoIterator<Item = usize>,
    ) -> Option<Self> {
        let result = result.into();
        let indices = indices.into_iter().collect::<Vec<_>>();
        (local_name(&result).is_some_and(is_valid_name) && !indices.is_empty()).then_some(
            Self::ExtractValue {
                result,
                aggregate,
                indices,
            },
        )
    }

    pub(in crate::backend::llvm) fn insert_value(
        result: impl Into<String>,
        aggregate: TypedValue,
        element: TypedValue,
        indices: impl IntoIterator<Item = usize>,
    ) -> Option<Self> {
        let result = result.into();
        let indices = indices.into_iter().collect::<Vec<_>>();
        (local_name(&result).is_some_and(is_valid_name) && !indices.is_empty()).then_some(
            Self::InsertValue {
                result,
                aggregate,
                element,
                indices,
            },
        )
    }

    pub(in crate::backend::llvm) fn phi(
        result: impl Into<String>,
        ty: Type,
        incoming: impl IntoIterator<Item = (String, String)>,
    ) -> Option<Self> {
        let result = result.into();
        let incoming = incoming.into_iter().collect::<Vec<_>>();
        (local_name(&result).is_some_and(is_valid_name)
            && !incoming.is_empty()
            && incoming
                .iter()
                .all(|(value, block)| is_value(value) && is_valid_name(block)))
        .then_some(Self::Phi {
            result,
            ty,
            incoming,
        })
    }

    pub(in crate::backend::llvm::syntax) fn uses_byte_runtime(&self) -> bool {
        match self {
            Self::Alloca { .. }
            | Self::Load { .. }
            | Self::Store { .. }
            | Self::Unary { .. }
            | Self::Binary { .. }
            | Self::Compare { .. }
            | Self::Cast { .. }
            | Self::GetElementPtr { .. }
            | Self::ExtractValue { .. }
            | Self::InsertValue { .. }
            | Self::Phi { .. } => false,
            Self::Call { callee, .. } => callee.uses_byte_runtime(),
        }
    }
}
