use crate::core::ast::ExternalOperation;
use mal_frontend::check::ast::Type;

use super::{
    TypeRegistry,
    syntax::{FunctionSignature, TypeName, c_parameter, c_parameters, c_signature},
};

pub(super) struct ExternalSignature<'a> {
    pub(super) operation_name: &'a str,
    result: HostValue<'a>,
    parameter: Option<HostValue<'a>>,
}

struct HostValue<'a> {
    ty: &'a Type,
    alias: Option<&'a str>,
    c_type: TypeName,
}

impl<'a> ExternalSignature<'a> {
    pub(super) fn new(external: &'a ExternalOperation, types: &TypeRegistry) -> Self {
        let signature = Self {
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
        };
        debug_assert!(signature.represents(external, types));
        signature
    }

    pub(super) fn parameter_names(&self) -> Vec<&str> {
        let mut names = vec!["call"];
        if self.parameter.is_some() {
            names.push("value");
        }
        names
    }

    pub(super) fn signature(&self, definition: bool) -> FunctionSignature {
        let mut parameters = c_parameters!(call: *mut mal_call_t);
        if definition {
            parameters[0] = parameters[0].clone().maybe_unused();
        }
        if let Some(parameter) = &self.parameter {
            parameters.push(c_parameter!(value: { parameter.c_type.clone() }));
        }
        c_signature! {
            fn { format!("mal_ext_{}", self.operation_name) }(
                ..{ parameters },
            ) -> { self.result.c_type.clone() }
        }
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
    fn preserves_one_source_carrier_at_the_public_boundary() {
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

        let signature = ExternalSignature::new(&external, &TypeRegistry::default());

        assert_eq!(signature.parameter_names(), ["call", "value"]);
        let parameter = signature.parameter.expect("one host value parameter");
        assert_eq!(parameter.ty, &external.parameter);
        assert_eq!(parameter.alias, Some("Request"));
        assert_eq!(parameter.c_type, TypeName::named("mal_Request_t"));
        assert_eq!(signature.result.alias, Some("Count"));
        assert_eq!(signature.result.c_type, TypeName::named("mal_Count_t"));
    }
}
