use super::*;

impl Declaration {
    pub(in crate::backend::c) fn render(&self) -> String {
        let declaration = match self {
            Self::Function(signature) => signature.render(),
            Self::TypeAlias { source, alias } => {
                format!("typedef {}", source.render_declarator(alias))
            }
        };
        format!("{declaration};\n")
    }
}

impl Comment {
    pub(in crate::backend::c) fn render(&self) -> String {
        format!("/* {} */\n", self.0.replace("*/", "* /"))
    }
}

impl AggregateDefinition {
    pub(in crate::backend::c) fn render(&self) -> String {
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
