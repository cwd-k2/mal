macro_rules! llvm_declaration_build {
    ($name:tt; [$($parameter:tt)*]; result { $($result:tt)* }; [$($attribute:tt)*]) => {
        $crate::backend::llvm::syntax::FunctionDeclaration::new(
            { $($result)* },
            $crate::backend::llvm::syntax::llvm_scalar!($name),
            $crate::backend::llvm::syntax::llvm_parameters!($($parameter)*),
        )
        .with_attributes(
            $crate::backend::llvm::syntax::llvm_function_attributes!($($attribute)*)
        )
    };
}

macro_rules! llvm_declaration {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (fn $name:tt($($parameter:tt)*) -> $kind:ident($($result:tt)*); attributes [$($attribute:tt)*]) => {
        $crate::backend::llvm::syntax::llvm_declaration_build!(
            $name;
            [$($parameter)*];
            result { $crate::backend::llvm::syntax::llvm_type!($kind($($result)*)) };
            [$($attribute)*]
        )
    };
    (fn $name:tt($($parameter:tt)*) -> $primitive:ident; attributes [$($attribute:tt)*]) => {
        $crate::backend::llvm::syntax::llvm_declaration_build!(
            $name;
            [$($parameter)*];
            result { $crate::backend::llvm::syntax::llvm_type!($primitive) };
            [$($attribute)*]
        )
    };
    (fn $name:tt($($parameter:tt)*) -> { $($result:tt)* }; attributes [$($attribute:tt)*]) => {
        $crate::backend::llvm::syntax::llvm_declaration_build!(
            $name; [$($parameter)*]; result { $($result)* }; [$($attribute)*]
        )
    };
}

macro_rules! llvm_global {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (byte_owner $name:tt; bytes { $($bytes:tt)* }; align $alignment:tt) => {
        $crate::backend::llvm::syntax::GlobalDefinition::byte_owner(
            $crate::backend::llvm::syntax::llvm_scalar!($name),
            { $($bytes)* },
            $crate::backend::llvm::syntax::llvm_scalar!($alignment),
        )
    };
}

macro_rules! llvm_metadata_operand {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (node $id:tt) => {
        $crate::backend::llvm::syntax::MetadataOperand::Node(
            $crate::backend::llvm::syntax::llvm_scalar!($id),
        )
    };
    (text $text:tt) => {
        $crate::backend::llvm::syntax::MetadataOperand::Text(
            $crate::backend::llvm::syntax::llvm_scalar!($text).into(),
        )
    };
    (integer $kind:ident($($ty:tt)*) => $value:tt) => {
        $crate::backend::llvm::syntax::MetadataOperand::Integer {
            ty: $crate::backend::llvm::syntax::llvm_type!($kind($($ty)*)),
            value: $crate::backend::llvm::syntax::llvm_scalar!($value),
        }
    };
    (integer $primitive:ident => $value:tt) => {
        $crate::backend::llvm::syntax::MetadataOperand::Integer {
            ty: $crate::backend::llvm::syntax::llvm_type!($primitive),
            value: $crate::backend::llvm::syntax::llvm_scalar!($value),
        }
    };
    (integer { $($ty:tt)* } => $value:tt) => {
        $crate::backend::llvm::syntax::MetadataOperand::Integer {
            ty: { $($ty)* },
            value: $crate::backend::llvm::syntax::llvm_scalar!($value),
        }
    };
}

macro_rules! llvm_metadata_operands {
    ($($operand:tt),* $(,)?) => {{
        #[allow(unused_mut, clippy::vec_init_then_push)]
        let mut operands = Vec::from([]);
        $(
            $crate::backend::llvm::syntax::llvm_metadata_operands_item!(operands; $operand);
        )*
        operands
    }};
}

macro_rules! llvm_metadata_operands_item {
    ($operands:ident; {{ $($rust:tt)* }}) => {
        $operands.extend({ $($rust)* })
    };
    ($operands:ident; { $($rust:tt)* }) => {
        $operands.push({ $($rust)* })
    };
    ($operands:ident; $operand:tt) => {
        $operands.push($crate::backend::llvm::syntax::llvm_metadata_operand! $operand)
    };
}

macro_rules! llvm_metadata {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    ($id:tt => [$($operand:tt),* $(,)?]) => {
        $crate::backend::llvm::syntax::MetadataDefinition::new(
            $crate::backend::llvm::syntax::llvm_scalar!($id),
            false,
            $crate::backend::llvm::syntax::llvm_metadata_operands!($($operand),*),
        )
    };
    (distinct $id:tt => [$($operand:tt),* $(,)?]) => {
        $crate::backend::llvm::syntax::MetadataDefinition::new(
            $crate::backend::llvm::syntax::llvm_scalar!($id),
            true,
            $crate::backend::llvm::syntax::llvm_metadata_operands!($($operand),*),
        )
    };
}

pub(in crate::backend) use {
    llvm_declaration, llvm_declaration_build, llvm_global, llvm_metadata, llvm_metadata_operand,
    llvm_metadata_operands, llvm_metadata_operands_item,
};
