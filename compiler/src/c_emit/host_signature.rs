use crate::check::ast::Type;
use crate::core::ast::ExternalOperation;

use super::TypeRegistry;

pub(super) struct HostSignature<'a> {
    pub(super) operation_name: &'a str,
    pub(super) result_type: String,
    parameters: Vec<HostParameter>,
}

struct HostParameter {
    c_type: String,
    default_name: String,
    is_context: bool,
}

impl<'a> HostSignature<'a> {
    pub(super) fn new(external: &'a ExternalOperation, types: &TypeRegistry) -> Self {
        let result_type = if external.result == Type::Unit {
            "void".into()
        } else {
            types.header_c_type(&external.result, external.result_alias.as_deref())
        };
        let mut parameters = vec![HostParameter {
            c_type: "MalContext *".into(),
            default_name: "context".into(),
            is_context: true,
        }];

        match &external.parameter {
            Type::Unit => {
                debug_assert!(external.parameter_aliases.is_empty());
            }
            Type::Product(elements) => {
                debug_assert_eq!(external.parameter_aliases.len(), elements.len());
                for (index, (element, alias)) in
                    elements.iter().zip(&external.parameter_aliases).enumerate()
                {
                    parameters.push(HostParameter {
                        c_type: types.header_c_type(element, alias.as_deref()),
                        default_name: format!("argument_{index}"),
                        is_context: false,
                    });
                }
            }
            parameter => {
                debug_assert_eq!(external.parameter_aliases.len(), 1);
                parameters.push(HostParameter {
                    c_type: types
                        .header_c_type(parameter, external.parameter_aliases[0].as_deref()),
                    default_name: "value".into(),
                    is_context: false,
                });
            }
        }

        Self {
            operation_name: &external.name,
            result_type,
            parameters,
        }
    }

    pub(super) fn parameter_names(&self) -> Vec<&str> {
        self.parameters
            .iter()
            .map(|parameter| parameter.default_name.as_str())
            .collect()
    }

    pub(super) fn parameter_declarations(&self) -> Vec<String> {
        self.render_parameter_declarations(false)
    }

    pub(super) fn definition_parameter_declarations(&self) -> Vec<String> {
        self.render_parameter_declarations(true)
    }

    fn render_parameter_declarations(&self, definition: bool) -> Vec<String> {
        self.parameters
            .iter()
            .map(|parameter| {
                let separator = if parameter.is_context { "" } else { " " };
                let unused = if definition && parameter.is_context {
                    " MAL_DETAIL_MAYBE_UNUSED"
                } else {
                    ""
                };
                format!(
                    "{}{separator}{}{unused}",
                    parameter.c_type, parameter.default_name
                )
            })
            .collect()
    }
}
