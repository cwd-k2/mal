use crate::core::ast::ExternalOperation;
use mal_frontend::check::ast::Type;

use super::{
    TypeRegistry,
    syntax::{FunctionSignature, Parameter, TypeName},
};

pub(super) struct ExternalSignatures<'a> {
    pub(super) compiler: CompilerSignature<'a>,
    pub(super) host_body: HostBodySignature<'a>,
}

pub(super) struct CompilerSignature<'a> {
    pub(super) operation_name: &'a str,
    pub(super) result_type: TypeName,
    parameters: Vec<CompilerParameter>,
}

struct CompilerParameter {
    c_type: TypeName,
    default_name: String,
    is_context: bool,
}

pub(super) struct HostBodySignature<'a> {
    pub(super) operation_name: &'a str,
    result: HostValue<'a>,
    parameter: Option<HostValue<'a>>,
    raw_result_type: TypeName,
}

struct HostValue<'a> {
    ty: &'a Type,
    alias: Option<&'a str>,
    c_type: TypeName,
}

impl<'a> ExternalSignatures<'a> {
    pub(super) fn new(external: &'a ExternalOperation, types: &TypeRegistry) -> Self {
        let signatures = Self {
            compiler: CompilerSignature::new(external, types),
            host_body: HostBodySignature::new(external, types),
        };
        debug_assert!(signatures.host_body.represents(external, types));
        signatures
    }
}

impl<'a> CompilerSignature<'a> {
    fn new(external: &'a ExternalOperation, types: &TypeRegistry) -> Self {
        let result_type = if external.result == Type::Unit {
            TypeName::named("void")
        } else {
            types.header_c_type(&external.result, external.result_alias.as_deref())
        };
        let mut parameters = vec![CompilerParameter {
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
                    parameters.push(CompilerParameter {
                        c_type: types.header_c_type(element, alias.as_deref()),
                        default_name: format!("argument_{index}"),
                        is_context: false,
                    });
                }
            }
            parameter => {
                debug_assert_eq!(external.parameter_aliases.len(), 1);
                parameters.push(CompilerParameter {
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

impl<'a> HostBodySignature<'a> {
    fn new(external: &'a ExternalOperation, types: &TypeRegistry) -> Self {
        Self {
            operation_name: &external.name,
            result: HostValue {
                ty: &external.result,
                alias: external.result_alias.as_deref(),
                c_type: types.host_value_c_type(&external.result, external.result_alias.as_deref()),
            },
            parameter: (external.parameter != Type::Unit).then_some(HostValue {
                ty: &external.parameter,
                alias: external.parameter_alias.as_deref(),
                c_type: types
                    .host_value_c_type(&external.parameter, external.parameter_alias.as_deref()),
            }),
            raw_result_type: if external.result == Type::Unit {
                TypeName::named("MalType_Unit")
            } else {
                types.header_c_type(&external.result, external.result_alias.as_deref())
            },
        }
    }

    pub(super) fn parameter_names(&self) -> Vec<&str> {
        let mut names = vec!["call"];
        if self.parameter.is_some() {
            names.push("value");
        }
        names
    }

    pub(super) fn signature(&self) -> FunctionSignature {
        let mut parameters = vec![Parameter::named(
            TypeName::named("mal_call_t").pointer(),
            "call",
        )];
        if let Some(parameter) = &self.parameter {
            parameters.push(Parameter::named(parameter.c_type.clone(), "value"));
        }
        FunctionSignature::static_function(
            self.raw_result_type.clone(),
            format!("mal_detail_{}", self.operation_name),
            parameters,
        )
    }

    fn represents(&self, external: &ExternalOperation, types: &TypeRegistry) -> bool {
        self.operation_name == external.name
            && self.result.ty == &external.result
            && self.result.alias == external.result_alias.as_deref()
            && self.result.c_type
                == types.host_value_c_type(&external.result, external.result_alias.as_deref())
            && match &self.parameter {
                Some(parameter) => {
                    parameter.ty == &external.parameter
                        && parameter.alias == external.parameter_alias.as_deref()
                        && parameter.c_type
                            == types.host_value_c_type(
                                &external.parameter,
                                external.parameter_alias.as_deref(),
                            )
                }
                None => external.parameter == Type::Unit && external.parameter_alias.is_none(),
            }
    }
}

#[cfg(test)]
mod tests {
    use mal_frontend::resolve::ast::ExternalOperationId;
    use mal_syntax::source::{FileId, Span};

    use super::*;

    #[test]
    fn separates_flattened_compiler_parameters_from_the_source_host_value() {
        let external = ExternalOperation {
            id: ExternalOperationId(0),
            name: "inspect".into(),
            parameter: Type::Product(vec![Type::UInt64, Type::Int32].into()),
            parameter_alias: Some("Request".into()),
            parameter_aliases: vec![Some("Count".into()), None],
            result: Type::UInt64,
            result_alias: Some("Count".into()),
            span: Span::new(FileId::new(0), 0, 0),
        };

        let signatures = ExternalSignatures::new(&external, &TypeRegistry::default());

        assert_eq!(
            signatures
                .compiler
                .parameters
                .iter()
                .map(|parameter| parameter.default_name.as_str())
                .collect::<Vec<_>>(),
            ["context", "argument_0", "argument_1"]
        );
        let parameter = signatures
            .host_body
            .parameter
            .expect("one host value parameter");
        assert_eq!(parameter.ty, &external.parameter);
        assert_eq!(parameter.alias, Some("Request"));
        assert_eq!(parameter.c_type, TypeName::named("mal_Request_t"));
        assert_eq!(signatures.host_body.result.alias, Some("Count"));
        assert_eq!(
            signatures.host_body.result.c_type,
            TypeName::named("mal_Count_t")
        );
    }
}
