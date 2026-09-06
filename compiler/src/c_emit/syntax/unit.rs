use super::{Expr, FunctionSignature, TypeName, VariableDeclaration};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum Declaration {
    Variable(VariableDeclaration),
    Function(FunctionSignature),
    TypeAlias { source: TypeName, alias: String },
    StaticAssert { condition: Expr, message: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct Comment(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct RawTranslationUnit(&'static str);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct AggregateDefinition {
    kind: AggregateKind,
    tag: Option<String>,
    fields: Vec<AggregateField>,
    is_typedef: bool,
    alias: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum AggregateKind {
    Struct,
    Union,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum AggregateField {
    Declaration(VariableDeclaration),
    Aggregate {
        kind: AggregateKind,
        fields: Vec<Self>,
        name: String,
    },
}

impl Declaration {
    pub(in crate::c_emit) fn variable(declaration: VariableDeclaration) -> Self {
        Self::Variable(declaration)
    }

    pub(in crate::c_emit) fn function(signature: FunctionSignature) -> Self {
        Self::Function(signature)
    }

    pub(in crate::c_emit) fn type_alias(
        source: impl Into<TypeName>,
        alias: impl Into<String>,
    ) -> Self {
        Self::TypeAlias {
            source: source.into(),
            alias: alias.into(),
        }
    }

    pub(in crate::c_emit) fn static_assert(condition: Expr, message: impl Into<String>) -> Self {
        Self::StaticAssert {
            condition,
            message: message.into(),
        }
    }

    pub(in crate::c_emit) fn render(&self) -> String {
        let declaration = match self {
            Self::Variable(variable) => variable.render(),
            Self::Function(signature) => signature.render(),
            Self::TypeAlias { source, alias } => {
                format!("typedef {}", source.render_declarator(alias))
            }
            Self::StaticAssert { condition, message } => {
                format!("_Static_assert({condition}, \"{message}\")")
            }
        };
        format!("{declaration};\n")
    }
}

impl Comment {
    pub(in crate::c_emit) fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    pub(in crate::c_emit) fn render(&self) -> String {
        format!("/* {} */\n", self.0)
    }
}

impl RawTranslationUnit {
    pub(in crate::c_emit) fn new(source: &'static str) -> Self {
        Self(source)
    }

    pub(in crate::c_emit) fn render(self) -> &'static str {
        self.0
    }
}

impl AggregateDefinition {
    pub(in crate::c_emit) fn structure(
        tag: impl Into<String>,
        fields: impl IntoIterator<Item = AggregateField>,
    ) -> Self {
        Self {
            kind: AggregateKind::Struct,
            tag: Some(tag.into()),
            fields: fields.into_iter().collect(),
            is_typedef: false,
            alias: None,
        }
    }

    pub(in crate::c_emit) fn typedef_structure(
        tag: Option<String>,
        fields: impl IntoIterator<Item = AggregateField>,
        alias: impl Into<String>,
    ) -> Self {
        Self {
            kind: AggregateKind::Struct,
            tag,
            fields: fields.into_iter().collect(),
            is_typedef: true,
            alias: Some(alias.into()),
        }
    }

    pub(in crate::c_emit) fn render(&self) -> String {
        let mut output = String::new();
        if self.is_typedef {
            output.push_str("typedef ");
        }
        output.push_str(self.kind.keyword());
        if let Some(tag) = &self.tag {
            output.push(' ');
            output.push_str(tag);
        }
        if self.tag.is_none()
            && self.is_typedef
            && self
                .fields
                .iter()
                .all(|field| matches!(field, AggregateField::Declaration(_)))
        {
            output.push_str(" { ");
            for field in &self.fields {
                let AggregateField::Declaration(declaration) = field else {
                    unreachable!()
                };
                output.push_str(&declaration.render());
                output.push_str("; ");
            }
            output.push('}');
            if let Some(alias) = &self.alias {
                output.push(' ');
                output.push_str(alias);
            }
            output.push_str(";\n");
            return output;
        }
        output.push_str(" {\n");
        render_fields(&mut output, &self.fields, 1);
        output.push('}');
        if let Some(alias) = &self.alias {
            output.push(' ');
            output.push_str(alias);
        }
        output.push_str(";\n");
        output
    }
}

impl AggregateField {
    pub(in crate::c_emit) fn variable(ty: impl Into<TypeName>, name: impl Into<String>) -> Self {
        Self::Declaration(VariableDeclaration::new(ty, name))
    }

    pub(in crate::c_emit) fn function_pointer(
        result: impl Into<TypeName>,
        name: impl Into<String>,
        parameters: impl IntoIterator<Item = super::Parameter>,
    ) -> Self {
        Self::Declaration(VariableDeclaration::function_pointer(
            result, name, parameters,
        ))
    }

    pub(in crate::c_emit) fn aggregate(
        kind: AggregateKind,
        fields: impl IntoIterator<Item = Self>,
        name: impl Into<String>,
    ) -> Self {
        Self::Aggregate {
            kind,
            fields: fields.into_iter().collect(),
            name: name.into(),
        }
    }
}

fn render_fields(output: &mut String, fields: &[AggregateField], depth: usize) {
    for field in fields {
        for _ in 0..depth {
            output.push_str("    ");
        }
        match field {
            AggregateField::Declaration(declaration) => {
                output.push_str(&declaration.render());
                output.push_str(";\n");
            }
            AggregateField::Aggregate { kind, fields, name } => {
                output.push_str(kind.keyword());
                output.push_str(" {\n");
                render_fields(output, fields, depth + 1);
                for _ in 0..depth {
                    output.push_str("    ");
                }
                output.push_str("} ");
                output.push_str(name);
                output.push_str(";\n");
            }
        }
    }
}

impl AggregateKind {
    fn keyword(self) -> &'static str {
        match self {
            Self::Struct => "struct",
            Self::Union => "union",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AggregateDefinition, AggregateField, AggregateKind};

    #[test]
    fn renders_nested_aggregate_definitions() {
        let definition = AggregateDefinition::structure(
            "Value",
            [
                AggregateField::variable("uint32_t", "tag"),
                AggregateField::aggregate(
                    AggregateKind::Union,
                    [AggregateField::variable("int32_t", "integer")],
                    "payload",
                ),
            ],
        );

        assert_eq!(
            definition.render(),
            "struct Value {\n    uint32_t tag;\n    union {\n        int32_t integer;\n    } payload;\n};\n"
        );
    }
}
