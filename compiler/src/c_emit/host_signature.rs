use crate::check::ast::Type;
use crate::core::ast::ExternalOperation;

use super::{
    TypeRegistry,
    syntax::{Parameter, TypeName},
};

pub(super) struct HostSignature<'a> {
    pub(super) operation_name: &'a str,
    pub(super) result_type: TypeName,
    parameters: Vec<HostParameter>,
}

struct HostParameter {
    c_type: TypeName,
    default_name: String,
    is_context: bool,
}

impl<'a> HostSignature<'a> {
    pub(super) fn new(external: &'a ExternalOperation, types: &TypeRegistry) -> Self {
        let result_type = if external.result == Type::Unit {
            TypeName::named("void")
        } else {
            types.header_c_type(&external.result, external.result_alias.as_deref())
        };
        let mut parameters = vec![HostParameter {
            c_type: TypeName::named("MalContext").pointer(),
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

    pub(super) fn parameters(&self) -> Vec<Parameter> {
        self.build_parameters(false)
    }

    pub(super) fn definition_parameters(&self) -> Vec<Parameter> {
        self.build_parameters(true)
    }

    fn build_parameters(&self, definition: bool) -> Vec<Parameter> {
        self.parameters
            .iter()
            .map(|parameter| {
                let declaration =
                    Parameter::named(parameter.c_type.clone(), parameter.default_name.clone());
                if definition && parameter.is_context {
                    declaration.maybe_unused()
                } else {
                    declaration
                }
            })
            .collect()
    }
}
