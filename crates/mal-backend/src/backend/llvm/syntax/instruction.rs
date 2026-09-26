use super::Type;
use super::function::{is_single_line, is_valid_name};

#[derive(Clone)]
pub(in crate::backend::llvm) enum Instruction {
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
    Raw(String),
}

#[derive(Clone)]
pub(in crate::backend::llvm) enum Callee {
    Direct(String),
    Indirect(String),
}

#[derive(Clone)]
pub(in crate::backend::llvm) struct TypedValue {
    ty: Type,
    value: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::backend::llvm) enum MetadataAttachment {
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
        local_name(&result)
            .is_some_and(is_valid_name)
            .then_some(Self::Alloca {
                result,
                ty,
                alignment,
            })
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
        (local_name(&result).is_some_and(is_valid_name) && is_value(&pointer)).then_some(
            Self::Load {
                result,
                ty,
                pointer,
                alignment,
                metadata: metadata.into_iter().collect(),
            },
        )
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
        (is_value(&value) && is_value(&pointer)).then_some(Self::Store {
            ty,
            value,
            pointer,
            alignment,
            metadata: metadata.into_iter().collect(),
        })
    }

    pub(in crate::backend::llvm) fn call(
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
            && !matches!(result_type, Type::Void)
            || result.is_none())
        .then_some(Self::Call {
            result,
            tail,
            result_type,
            callee,
            arguments: arguments.into_iter().collect(),
        })
    }

    pub(super) fn raw(text: impl Into<String>) -> Option<Self> {
        let text = text.into();
        is_single_line(&text).then_some(Self::Raw(text))
    }

    pub(super) fn render_into(&self, output: &mut String) {
        match self {
            Self::Alloca {
                result,
                ty,
                alignment,
            } => output.push_str(&format!("{result} = alloca {ty}, align {alignment}")),
            Self::Load {
                result,
                ty,
                pointer,
                alignment,
                metadata,
            } => {
                output.push_str(&format!(
                    "{result} = load {ty}, ptr {pointer}, align {alignment}"
                ));
                render_metadata(output, metadata);
            }
            Self::Store {
                ty,
                value,
                pointer,
                alignment,
                metadata,
            } => {
                output.push_str(&format!(
                    "store {ty} {value}, ptr {pointer}, align {alignment}"
                ));
                render_metadata(output, metadata);
            }
            Self::Call {
                result,
                tail,
                result_type,
                callee,
                arguments,
            } => {
                if let Some(result) = result {
                    output.push_str(result);
                    output.push_str(" = ");
                }
                if *tail {
                    output.push_str("tail ");
                }
                output.push_str("call ");
                output.push_str(&result_type.to_string());
                output.push(' ');
                callee.render_into(output);
                output.push('(');
                for (index, argument) in arguments.iter().enumerate() {
                    if index != 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&argument.ty.to_string());
                    output.push(' ');
                    output.push_str(&argument.value);
                }
                output.push(')');
            }
            Self::Raw(text) => output.push_str(text),
        }
    }

    pub(super) fn uses_byte_runtime(&self) -> bool {
        match self {
            Self::Alloca { .. } | Self::Load { .. } | Self::Store { .. } => false,
            Self::Call { callee, .. } => callee.uses_byte_runtime(),
            Self::Raw(text) => [
                "@mal_runtime_bytes_",
                "@mal_runtime_buffer_",
                "@mal_runtime_symbol_",
            ]
            .iter()
            .any(|prefix| text.contains(prefix)),
        }
    }
}

impl Callee {
    pub(in crate::backend::llvm) fn direct(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        is_valid_name(&name).then_some(Self::Direct(name))
    }

    pub(in crate::backend::llvm) fn indirect(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        is_value(&value).then_some(Self::Indirect(value))
    }

    fn render_into(&self, output: &mut String) {
        match self {
            Self::Direct(name) => output.push_str(&format!("@{name}")),
            Self::Indirect(value) => output.push_str(value),
        }
    }

    fn uses_byte_runtime(&self) -> bool {
        matches!(self, Self::Direct(name) if name.starts_with("mal_runtime_bytes_")
            || name.starts_with("mal_runtime_buffer_")
            || name.starts_with("mal_runtime_symbol_"))
    }
}

impl TypedValue {
    pub(in crate::backend::llvm) fn new(ty: Type, value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        is_value(&value).then_some(Self { ty, value })
    }
}

fn local_name(value: &str) -> Option<&str> {
    value.strip_prefix('%')
}

fn is_value(value: &str) -> bool {
    is_single_line(value)
}

fn render_metadata(output: &mut String, metadata: &[MetadataAttachment]) {
    for attachment in metadata {
        let (name, node) = match attachment {
            MetadataAttachment::Tbaa(node) => ("tbaa", node),
            MetadataAttachment::AliasScope(node) => ("alias.scope", node),
            MetadataAttachment::NoAlias(node) => ("noalias", node),
        };
        output.push_str(&format!(", !{name} !{node}"));
    }
}

#[cfg(test)]
mod tests {
    use super::{Instruction, MetadataAttachment, Type};

    #[test]
    fn renders_alloca_from_typed_fields() {
        let instruction = Instruction::alloca(
            "%storage",
            Type::structure([Type::integer(32_u16), Type::Pointer]),
            8,
        )
        .unwrap();
        let mut output = String::new();
        instruction.render_into(&mut output);

        assert_eq!(output, "%storage = alloca { i32, ptr }, align 8");
    }

    #[test]
    fn renders_memory_operations_and_metadata_from_fields() {
        let mut output = String::new();
        Instruction::load(
            "%value",
            Type::integer(32_u16),
            "%address",
            4,
            [MetadataAttachment::Tbaa(3), MetadataAttachment::NoAlias(6)],
        )
        .unwrap()
        .render_into(&mut output);
        assert_eq!(
            output,
            "%value = load i32, ptr %address, align 4, !tbaa !3, !noalias !6"
        );

        output.clear();
        Instruction::store(Type::Pointer, "null", "%address", 8, std::iter::empty())
            .unwrap()
            .render_into(&mut output);
        assert_eq!(output, "store ptr null, ptr %address, align 8");
    }
}
