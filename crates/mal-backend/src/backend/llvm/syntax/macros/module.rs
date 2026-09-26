macro_rules! llvm_declaration_build {
    ($name:tt; [$($parameter:tt)*]; $result:expr; [$($attribute:tt)*]) => {
        $crate::backend::llvm::syntax::FunctionDeclaration::new(
            $result,
            $name,
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
            $crate::backend::llvm::syntax::llvm_type!($kind($($result)*));
            [$($attribute)*]
        )
    };
    (fn $name:tt($($parameter:tt)*) -> $primitive:ident; attributes [$($attribute:tt)*]) => {
        $crate::backend::llvm::syntax::llvm_declaration_build!(
            $name;
            [$($parameter)*];
            $crate::backend::llvm::syntax::llvm_type!($primitive);
            [$($attribute)*]
        )
    };
    (fn $name:tt($($parameter:tt)*) -> { $($result:tt)* }; attributes [$($attribute:tt)*]) => {
        $crate::backend::llvm::syntax::llvm_declaration_build!(
            $name; [$($parameter)*]; { $($result)* }; [$($attribute)*]
        )
    };
}

macro_rules! llvm_global {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (byte_owner $name:expr; bytes { $($bytes:tt)* }; align $alignment:expr) => {
        $crate::backend::llvm::syntax::GlobalDefinition::byte_owner(
            $name,
            { $($bytes)* },
            $alignment,
        )
    };
}

macro_rules! llvm_metadata_operand {
    ({ $($rust:tt)* }) => {{ $($rust)* }};
    (node $id:expr) => {
        $crate::backend::llvm::syntax::MetadataOperand::Node($id)
    };
    (text $text:expr) => {
        $crate::backend::llvm::syntax::MetadataOperand::Text($text.into())
    };
    (integer $kind:ident($($ty:tt)*) => $value:expr) => {
        $crate::backend::llvm::syntax::MetadataOperand::Integer {
            ty: $crate::backend::llvm::syntax::llvm_type!($kind($($ty)*)),
            value: $value,
        }
    };
    (integer $primitive:ident => $value:expr) => {
        $crate::backend::llvm::syntax::MetadataOperand::Integer {
            ty: $crate::backend::llvm::syntax::llvm_type!($primitive),
            value: $value,
        }
    };
    (integer { $($ty:tt)* } => $value:expr) => {
        $crate::backend::llvm::syntax::MetadataOperand::Integer {
            ty: { $($ty)* },
            value: $value,
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
    ($id:expr => [$($operand:tt),* $(,)?]) => {
        $crate::backend::llvm::syntax::MetadataDefinition::new(
            $id,
            false,
            $crate::backend::llvm::syntax::llvm_metadata_operands!($($operand),*),
        )
    };
    (distinct $id:expr => [$($operand:tt),* $(,)?]) => {
        $crate::backend::llvm::syntax::MetadataDefinition::new(
            $id,
            true,
            $crate::backend::llvm::syntax::llvm_metadata_operands!($($operand),*),
        )
    };
}

pub(in crate::backend::llvm) use {
    llvm_declaration, llvm_declaration_build, llvm_global, llvm_metadata, llvm_metadata_operand,
    llvm_metadata_operands, llvm_metadata_operands_item,
};
