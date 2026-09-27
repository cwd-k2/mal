macro_rules! llvm_declaration_result_normalized {
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; $kind:ident($($result:tt)*)) => {
        $crate::backend::llvm::syntax::llvm_declaration_build_normalized! {
            $name; [$($parameter)*]; [$($attribute)*];
            $crate::backend::llvm::syntax::llvm_type_normalized!($kind($($result)*))
        }
    };
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; $primitive:ident) => {
        $crate::backend::llvm::syntax::llvm_declaration_build_normalized! {
            $name; [$($parameter)*]; [$($attribute)*];
            $crate::backend::llvm::syntax::llvm_type_normalized!($primitive)
        }
    };
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; (@rust $($result:tt)*)) => {
        $crate::backend::llvm::syntax::llvm_declaration_build_normalized! {
            $name; [$($parameter)*]; [$($attribute)*]; { $($result)* }
        }
    };
}

macro_rules! llvm_declaration_build_normalized {
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; $result:expr) => {
        $crate::backend::llvm::syntax::FunctionDeclaration::new(
            $result,
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($name),
            $crate::backend::llvm::syntax::llvm_parameters_normalized!($($parameter)*),
        )
        .with_attributes(
            $crate::backend::llvm::syntax::llvm_function_attributes_normalized!($($attribute)*)
        )
    };
}

macro_rules! llvm_declaration_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (#[attributes($($attribute:tt)*)] fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*);) => {
        $crate::backend::llvm::syntax::llvm_declaration_result_normalized! {
            $name; [$($parameter)*]; [$($attribute)*]; $kind($($result)*)
        }
    };
    (#[attributes($($attribute:tt)*)] fn $name:tt($($parameter:tt)*) -> $result:ident;) => {
        $crate::backend::llvm::syntax::llvm_declaration_result_normalized! {
            $name; [$($parameter)*]; [$($attribute)*]; $result
        }
    };
    (#[attributes($($attribute:tt)*)] fn $name:tt($($parameter:tt)*) -> (@rust $($result:tt)*);) => {
        $crate::backend::llvm::syntax::llvm_declaration_result_normalized! {
            $name; [$($parameter)*]; [$($attribute)*]; (@rust $($result)*)
        }
    };
    (fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*);) => {
        $crate::backend::llvm::syntax::llvm_declaration_result_normalized! {
            $name; [$($parameter)*]; []; $kind($($result)*)
        }
    };
    (fn $name:tt($($parameter:tt)*) -> $result:ident;) => {
        $crate::backend::llvm::syntax::llvm_declaration_result_normalized! {
            $name; [$($parameter)*]; []; $result
        }
    };
    (fn $name:tt($($parameter:tt)*) -> (@rust $($result:tt)*);) => {
        $crate::backend::llvm::syntax::llvm_declaration_result_normalized! {
            $name; [$($parameter)*]; []; (@rust $($result)*)
        }
    };
}

macro_rules! llvm_global_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (byte_owner {
        name: $name:tt,
        bytes: (@rust $($bytes:tt)*),
        alignment: $alignment:tt $(,)?
    }) => {
        $crate::backend::llvm::syntax::GlobalDefinition::byte_owner(
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($name),
            { $($bytes)* },
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($alignment),
        )
    };
}

macro_rules! llvm_metadata_operand_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    (node($id:tt)) => {
        $crate::backend::llvm::syntax::MetadataOperand::Node(
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($id),
        )
    };
    (text($text:tt)) => {
        $crate::backend::llvm::syntax::MetadataOperand::Text(
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($text).into(),
        )
    };
    (integer($kind:ident($($ty:tt)*), $value:tt)) => {
        $crate::backend::llvm::syntax::MetadataOperand::Integer {
            ty: $crate::backend::llvm::syntax::llvm_type_normalized!($kind($($ty)*)),
            value: $crate::backend::llvm::syntax::llvm_scalar_normalized!($value),
        }
    };
    (integer($primitive:ident, $value:tt)) => {
        $crate::backend::llvm::syntax::MetadataOperand::Integer {
            ty: $crate::backend::llvm::syntax::llvm_type_normalized!($primitive),
            value: $crate::backend::llvm::syntax::llvm_scalar_normalized!($value),
        }
    };
    (integer((@rust $($ty:tt)*), $value:tt)) => {
        $crate::backend::llvm::syntax::MetadataOperand::Integer {
            ty: { $($ty)* },
            value: $crate::backend::llvm::syntax::llvm_scalar_normalized!($value),
        }
    };
}

macro_rules! llvm_metadata_operands_normalized {
    ($($operand:tt)*) => {{
        #[allow(unused_mut, clippy::vec_init_then_push)]
        let mut operands = Vec::from([]);
        $crate::backend::llvm::syntax::llvm_metadata_operands_items_normalized!(operands; $($operand)*);
        operands
    }};
}

macro_rules! llvm_metadata_operands_items_normalized {
    ($operands:ident;) => {};
    ($operands:ident; ...(@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $operands.extend({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_metadata_operands_items_normalized!($operands; $($($rest)*)?);
    };
    ($operands:ident; (@rust $($rust:tt)*) $(, $($rest:tt)*)?) => {
        $operands.push({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_metadata_operands_items_normalized!($operands; $($($rest)*)?);
    };
    ($operands:ident; $kind:ident($($argument:tt)*) $(, $($rest:tt)*)?) => {
        $operands.push($crate::backend::llvm::syntax::llvm_metadata_operand_normalized!($kind($($argument)*)));
        $crate::backend::llvm::syntax::llvm_metadata_operands_items_normalized!($operands; $($($rest)*)?);
    };
}

macro_rules! llvm_metadata_normalized {
    ((@rust $($rust:tt)*)) => {{ $($rust)* }};
    ({ id: $id:tt, distinct: $distinct:literal, operands: [$($operand:tt)*] $(,)? }) => {
        $crate::backend::llvm::syntax::MetadataDefinition::new(
            $crate::backend::llvm::syntax::llvm_scalar_normalized!($id),
            $distinct,
            $crate::backend::llvm::syntax::llvm_metadata_operands_normalized!($($operand)*),
        )
    };
}

macro_rules! llvm_declaration {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::llvm::syntax::llvm_declaration_normalized]; $($syntax)*
        )
    };
}

macro_rules! llvm_global {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::llvm::syntax::llvm_global_normalized]; $($syntax)*
        )
    };
}

#[cfg(test)]
macro_rules! llvm_metadata_operand {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::llvm::syntax::llvm_metadata_operand_normalized]; $($syntax)*
        )
    };
}

macro_rules! llvm_metadata {
    ($($syntax:tt)*) => {
        $crate::backend::normalize_syntax_interpolation!(
            [$crate::backend::llvm::syntax::llvm_metadata_normalized]; $($syntax)*
        )
    };
}

pub(in crate::backend) use {
    llvm_declaration_build_normalized, llvm_declaration_normalized,
    llvm_declaration_result_normalized, llvm_global_normalized, llvm_metadata_normalized,
    llvm_metadata_operand_normalized, llvm_metadata_operands_items_normalized,
    llvm_metadata_operands_normalized,
};

#[cfg(test)]
pub(in crate::backend) use llvm_metadata_operand;
pub(in crate::backend) use {llvm_declaration, llvm_global, llvm_metadata};
