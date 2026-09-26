use super::Type;
use super::function::{is_single_line, is_valid_name};

#[derive(Clone)]
pub(in crate::backend::llvm) enum Instruction {
    Alloca {
        result: String,
        ty: Type,
        alignment: usize,
    },
    Raw(String),
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
            Self::Raw(text) => output.push_str(text),
        }
    }

    pub(super) fn uses_byte_runtime(&self) -> bool {
        match self {
            Self::Alloca { .. } => false,
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

fn local_name(value: &str) -> Option<&str> {
    value.strip_prefix('%')
}

#[cfg(test)]
mod tests {
    use super::{Instruction, Type};

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
}
