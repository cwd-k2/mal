macro_rules! llvm_declaration_result {
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; $kind:ident($($result:tt)*)) => {
        $crate::backend::llvm::syntax::llvm_declaration_build!(
            $name; [$($parameter)*]; [$($attribute)*];
            $crate::backend::llvm::syntax::llvm_type!($kind($($result)*))
        )
    };
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; $primitive:ident) => {
        $crate::backend::llvm::syntax::llvm_declaration_build!(
            $name; [$($parameter)*]; [$($attribute)*];
            $crate::backend::llvm::syntax::llvm_type!($primitive)
        )
    };
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; {{ $($result:tt)* }}) => {
        $crate::backend::llvm::syntax::llvm_declaration_build!(
            $name; [$($parameter)*]; [$($attribute)*]; { $($result)* }
        )
    };
}

macro_rules! llvm_declaration_build {
    ($name:tt; [$($parameter:tt)*]; [$($attribute:tt)*]; $result:expr) => {
        $crate::backend::llvm::syntax::FunctionDeclaration::new(
            $result,
            $crate::backend::llvm::syntax::llvm_scalar!($name),
            $crate::backend::llvm::syntax::llvm_parameters!($($parameter)*),
        )
        .with_attributes(
            $crate::backend::llvm::syntax::llvm_function_attributes!($($attribute)*)
        )
    };
}

macro_rules! llvm_declaration {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (#[attributes($($attribute:tt)*)] fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*);) => {
        $crate::backend::llvm::syntax::llvm_declaration_result!(
            $name; [$($parameter)*]; [$($attribute)*]; $kind($($result)*)
        )
    };
    (#[attributes($($attribute:tt)*)] fn $name:tt($($parameter:tt)*) -> $result:ident;) => {
        $crate::backend::llvm::syntax::llvm_declaration_result!(
            $name; [$($parameter)*]; [$($attribute)*]; $result
        )
    };
    (#[attributes($($attribute:tt)*)] fn $name:tt($($parameter:tt)*) -> {{ $($result:tt)* }};) => {
        $crate::backend::llvm::syntax::llvm_declaration_result!(
            $name; [$($parameter)*]; [$($attribute)*]; {{ $($result)* }}
        )
    };
    (fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*);) => {
        $crate::backend::llvm::syntax::llvm_declaration_result!(
            $name; [$($parameter)*]; []; $kind($($result)*)
        )
    };
    (fn $name:tt($($parameter:tt)*) -> $result:ident;) => {
        $crate::backend::llvm::syntax::llvm_declaration_result!(
            $name; [$($parameter)*]; []; $result
        )
    };
    (fn $name:tt($($parameter:tt)*) -> {{ $($result:tt)* }};) => {
        $crate::backend::llvm::syntax::llvm_declaration_result!(
            $name; [$($parameter)*]; []; {{ $($result)* }}
        )
    };
}

macro_rules! llvm_global {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (byte_owner {
        name: $name:tt,
        bytes: {{ $($bytes:tt)* }},
        alignment: $alignment:tt $(,)?
    }) => {
        $crate::backend::llvm::syntax::GlobalDefinition::byte_owner(
            $crate::backend::llvm::syntax::llvm_scalar!($name),
            { $($bytes)* },
            $crate::backend::llvm::syntax::llvm_scalar!($alignment),
        )
    };
}

macro_rules! llvm_metadata_operand {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    (node($id:tt)) => {
        $crate::backend::llvm::syntax::MetadataOperand::Node(
            $crate::backend::llvm::syntax::llvm_scalar!($id),
        )
    };
    (text($text:tt)) => {
        $crate::backend::llvm::syntax::MetadataOperand::Text(
            $crate::backend::llvm::syntax::llvm_scalar!($text).into(),
        )
    };
    (integer($kind:ident($($ty:tt)*), $value:tt)) => {
        $crate::backend::llvm::syntax::MetadataOperand::Integer {
            ty: $crate::backend::llvm::syntax::llvm_type!($kind($($ty)*)),
            value: $crate::backend::llvm::syntax::llvm_scalar!($value),
        }
    };
    (integer($primitive:ident, $value:tt)) => {
        $crate::backend::llvm::syntax::MetadataOperand::Integer {
            ty: $crate::backend::llvm::syntax::llvm_type!($primitive),
            value: $crate::backend::llvm::syntax::llvm_scalar!($value),
        }
    };
    (integer({{ $($ty:tt)* }}, $value:tt)) => {
        $crate::backend::llvm::syntax::MetadataOperand::Integer {
            ty: { $($ty)* },
            value: $crate::backend::llvm::syntax::llvm_scalar!($value),
        }
    };
}

macro_rules! llvm_metadata_operands {
    ($($operand:tt)*) => {{
        #[allow(unused_mut, clippy::vec_init_then_push)]
        let mut operands = Vec::from([]);
        $crate::backend::llvm::syntax::llvm_metadata_operands_items!(operands; $($operand)*);
        operands
    }};
}

macro_rules! llvm_metadata_operands_items {
    ($operands:ident;) => {};
    ($operands:ident; ...{{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $operands.extend({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_metadata_operands_items!($operands; $($($rest)*)?);
    };
    ($operands:ident; {{ $($rust:tt)* }} $(, $($rest:tt)*)?) => {
        $operands.push({ $($rust)* });
        $crate::backend::llvm::syntax::llvm_metadata_operands_items!($operands; $($($rest)*)?);
    };
    ($operands:ident; $kind:ident($($argument:tt)*) $(, $($rest:tt)*)?) => {
        $operands.push($crate::backend::llvm::syntax::llvm_metadata_operand!($kind($($argument)*)));
        $crate::backend::llvm::syntax::llvm_metadata_operands_items!($operands; $($($rest)*)?);
    };
}

macro_rules! llvm_metadata {
    ({{ $($rust:tt)* }}) => {{ $($rust)* }};
    ({ id: $id:tt, distinct: $distinct:literal, operands: [$($operand:tt)*] $(,)? }) => {
        $crate::backend::llvm::syntax::MetadataDefinition::new(
            $crate::backend::llvm::syntax::llvm_scalar!($id),
            $distinct,
            $crate::backend::llvm::syntax::llvm_metadata_operands!($($operand)*),
        )
    };
}

pub(in crate::backend) use {
    llvm_declaration, llvm_declaration_build, llvm_declaration_result, llvm_global, llvm_metadata,
    llvm_metadata_operand, llvm_metadata_operands, llvm_metadata_operands_items,
};
