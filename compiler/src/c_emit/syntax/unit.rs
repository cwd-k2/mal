#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct Declaration(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct Comment(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct RawTranslationUnit(&'static str);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) struct AggregateDefinition {
    header: String,
    fields: Vec<AggregateField>,
    declarator: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::c_emit) enum AggregateField {
    Declaration(Declaration),
    Aggregate {
        keyword: &'static str,
        fields: Vec<Self>,
        declarator: String,
    },
}

impl Declaration {
    pub(in crate::c_emit) fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    pub(in crate::c_emit) fn render(&self) -> String {
        format!("{};\n", self.0)
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
    pub(in crate::c_emit) fn new(
        header: impl Into<String>,
        fields: impl IntoIterator<Item = AggregateField>,
        declarator: Option<String>,
    ) -> Self {
        Self {
            header: header.into(),
            fields: fields.into_iter().collect(),
            declarator,
        }
    }

    pub(in crate::c_emit) fn render(&self) -> String {
        let mut output = format!("{} {{\n", self.header);
        render_fields(&mut output, &self.fields, 1);
        output.push('}');
        if let Some(declarator) = &self.declarator {
            output.push(' ');
            output.push_str(declarator);
        }
        output.push_str(";\n");
        output
    }
}

impl AggregateField {
    pub(in crate::c_emit) fn declaration(text: impl Into<String>) -> Self {
        Self::Declaration(Declaration::new(text))
    }

    pub(in crate::c_emit) fn aggregate(
        keyword: &'static str,
        fields: impl IntoIterator<Item = Self>,
        declarator: impl Into<String>,
    ) -> Self {
        Self::Aggregate {
            keyword,
            fields: fields.into_iter().collect(),
            declarator: declarator.into(),
        }
    }
}

fn render_fields(output: &mut String, fields: &[AggregateField], depth: usize) {
    for field in fields {
        for _ in 0..depth {
            output.push_str("    ");
        }
        match field {
            AggregateField::Declaration(declaration) => output.push_str(&declaration.render()),
            AggregateField::Aggregate {
                keyword,
                fields,
                declarator,
            } => {
                output.push_str(keyword);
                output.push_str(" {\n");
                render_fields(output, fields, depth + 1);
                for _ in 0..depth {
                    output.push_str("    ");
                }
                output.push_str("} ");
                output.push_str(declarator);
                output.push_str(";\n");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AggregateDefinition, AggregateField};

    #[test]
    fn renders_nested_aggregate_definitions() {
        let definition = AggregateDefinition::new(
            "struct Value",
            [
                AggregateField::declaration("uint32_t tag"),
                AggregateField::aggregate(
                    "union",
                    [AggregateField::declaration("int32_t integer")],
                    "payload",
                ),
            ],
            None,
        );

        assert_eq!(
            definition.render(),
            "struct Value {\n    uint32_t tag;\n    union {\n        int32_t integer;\n    } payload;\n};\n"
        );
    }
}
