//! The rendering of each instruction to LLVM assembly text.

use super::*;

impl Instruction {
    pub(in crate::backend::llvm::syntax) fn render_into(&self, output: &mut String) {
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
            Self::Unary {
                result,
                operator,
                operand,
            } => output.push_str(&format!(
                "{result} = {} {} {}",
                operator.mnemonic(),
                operand.ty,
                operand.value
            )),
            Self::Binary {
                result,
                operator,
                ty,
                left,
                right,
            } => output.push_str(&format!(
                "{result} = {} {ty} {left}, {right}",
                operator.mnemonic()
            )),
            Self::Compare {
                result,
                kind,
                predicate,
                ty,
                left,
                right,
            } => output.push_str(&format!(
                "{result} = {} {} {ty} {left}, {right}",
                kind.mnemonic(),
                predicate.mnemonic()
            )),
            Self::Cast {
                result,
                operator,
                operand,
                target,
            } => output.push_str(&format!(
                "{result} = {} {} {} to {target}",
                operator.mnemonic(),
                operand.ty,
                operand.value
            )),
            Self::GetElementPtr {
                result,
                inbounds,
                element_type,
                pointer,
                indices,
            } => {
                output.push_str(&format!("{result} = getelementptr"));
                if *inbounds {
                    output.push_str(" inbounds");
                }
                output.push_str(&format!(" {element_type}, ptr {pointer}"));
                for index in indices {
                    output.push_str(&format!(", {} {}", index.ty, index.value));
                }
            }
            Self::ExtractValue {
                result,
                aggregate,
                indices,
            } => output.push_str(&format!(
                "{result} = extractvalue {} {}, {}",
                aggregate.ty,
                aggregate.value,
                render_indices(indices)
            )),
            Self::InsertValue {
                result,
                aggregate,
                element,
                indices,
            } => output.push_str(&format!(
                "{result} = insertvalue {} {}, {} {}, {}",
                aggregate.ty,
                aggregate.value,
                element.ty,
                element.value,
                render_indices(indices)
            )),
            Self::Phi {
                result,
                ty,
                incoming,
            } => {
                output.push_str(&format!("{result} = phi {ty} "));
                for (index, (value, block)) in incoming.iter().enumerate() {
                    if index != 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&format!("[{value}, %{block}]"));
                }
            }
        }
    }
}

fn render_indices(indices: &[usize]) -> String {
    indices
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(", ")
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
