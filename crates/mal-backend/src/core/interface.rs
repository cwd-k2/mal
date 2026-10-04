use mal_frontend::check::ast as checked;

use super::ast::{ExternalOperation, ExternalType, ProgramInterface, TypeAlias};

pub(crate) fn lower_interface(program: &checked::Program) -> ProgramInterface {
    let mut interface = ProgramInterface {
        type_aliases: Vec::new(),
        external_types: Vec::new(),
        externals: Vec::new(),
    };
    for item in &program.items {
        match &item.kind {
            checked::TopItem::TypeAlias {
                binding,
                ty,
                element_aliases,
            } => {
                interface.type_aliases.push(TypeAlias {
                    name: binding.name.text.clone(),
                    ty: ty.clone(),
                    element_aliases: element_aliases.clone(),
                    span: item.span,
                });
            }
            checked::TopItem::ExternalType { binding } => {
                interface.external_types.push(ExternalType {
                    name: binding.name.text.clone(),
                    span: item.span,
                });
            }
            checked::TopItem::ExternalOperation {
                id,
                binding,
                parameter,
                parameter_alias,
                parameter_aliases,
                result,
                result_alias,
                ..
            } => interface.externals.push(ExternalOperation {
                id: *id,
                name: binding.name.text.clone(),
                parameter: parameter.clone(),
                parameter_alias: parameter_alias.clone(),
                parameter_aliases: parameter_aliases.clone(),
                result: result.clone(),
                result_alias: result_alias.clone(),
                span: item.span,
            }),
            checked::TopItem::GenericBinding(_)
            | checked::TopItem::OpaqueType { .. }
            | checked::TopItem::OperationFamily(_)
            | checked::TopItem::OperationImplementation(_)
            | checked::TopItem::Binding(_) => {}
        }
    }
    interface
}

impl ProgramInterface {
    pub(crate) fn for_file(&self, file: mal_syntax::source::FileId) -> Self {
        Self {
            type_aliases: self
                .type_aliases
                .iter()
                .filter(|alias| alias.span.file() == file)
                .cloned()
                .collect(),
            external_types: self
                .external_types
                .iter()
                .filter(|external| external.span.file() == file)
                .cloned()
                .collect(),
            externals: self
                .externals
                .iter()
                .filter(|external| external.span.file() == file)
                .cloned()
                .collect(),
        }
    }
}
